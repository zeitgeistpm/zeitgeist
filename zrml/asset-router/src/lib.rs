// Copyright 2023 Forecasting Technologies LTD.
//
// This file is part of Zeitgeist.
//
// Zeitgeist is free software: you can redistribute it and/or modify it
// under the terms of the GNU General Public License as published by the
// Free Software Foundation, either version 3 of the License, or (at
// your option) any later version.
//
// Zeitgeist is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Zeitgeist. If not, see <https://www.gnu.org/licenses/>.

#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use core::marker::PhantomData;
    use frame_support::{
        pallet_prelude::{DispatchError, DispatchResult},
        traits::{
            tokens::{
                fungibles::{Create, Destroy, Inspect, Mutate, Transfer},
                DepositConsequence, WithdrawConsequence,
            },
            BalanceStatus as Status,
        },
        transactional, Parameter,
    };
    use orml_traits::{
        arithmetic::Signed,
        currency::{
            MultiCurrency, MultiCurrencyExtended, MultiLockableCurrency, MultiReservableCurrency,
            NamedMultiReservableCurrency, TransferAll,
        },
        BalanceStatus, LockIdentifier,
    };
    use parity_scale_codec::MaxEncodedLen;
    use sp_runtime::{
        traits::{
            AtLeast32BitUnsigned, Bounded, MaybeSerializeDeserialize, Member, Saturating, Zero,
        },
        FixedPointOperand,
    };
    use zeitgeist_primitives::types::{
        Assets, CampaignAsset, Currencies, CustomAsset, MarketAsset,
    };

    pub trait AssetTraits<T: Config, A>:
        Create<T::AccountId, AssetId = A, Balance = T::Balance>
        + Destroy<T::AccountId, AssetId = A, Balance = T::Balance>
        + Inspect<T::AccountId, AssetId = A, Balance = T::Balance>
        + Transfer<T::AccountId, AssetId = A, Balance = T::Balance>
        + Mutate<T::AccountId, AssetId = A, Balance = T::Balance>
    {
    }

    impl<G, T, A> AssetTraits<T, A> for G
    where
        G: Create<T::AccountId, AssetId = A, Balance = T::Balance>
            + Destroy<T::AccountId, AssetId = A, Balance = T::Balance>
            + Inspect<T::AccountId, AssetId = A, Balance = T::Balance>
            + Transfer<T::AccountId, AssetId = A, Balance = T::Balance>
            + Mutate<T::AccountId, AssetId = A, Balance = T::Balance>,
        T: Config,
    {
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Balance: Parameter
            + Member
            + AtLeast32BitUnsigned
            + Default
            + Copy
            + MaybeSerializeDeserialize
            + MaxEncodedLen
            + FixedPointOperand;
        type Currencies: TransferAll<Self::AccountId>
            + MultiCurrencyExtended<Self::AccountId, CurrencyId = Currencies, Balance = Self::Balance>
            + MultiLockableCurrency<Self::AccountId>
            + MultiReservableCurrency<Self::AccountId>
            + NamedMultiReservableCurrency<Self::AccountId>;
        type CampaignAsset: AssetTraits<Self, CampaignAsset>;
        type CustomAsset: AssetTraits<Self, CustomAsset>;
        type MarketAssets: AssetTraits<Self, MarketAsset>;
    }

    use frame_support::pallet_prelude::Hooks;

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::call]
    impl<T: Config> Pallet<T> {}

    impl<T: Config> TransferAll<T::AccountId> for Pallet<T> {
        #[transactional]
        fn transfer_all(source: &T::AccountId, dest: &T::AccountId) -> DispatchResult {
            // Only transfers assets maintained in orml-tokens, not implementable for pallet-assets
            <T::Currencies as TransferAll<T::AccountId>>::transfer_all(source, dest)
        }
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Cannot convert Amount (MultiCurrencyExtended implementation) into Balance type.
        AmountIntoBalanceFailed,
        /// Asset conversion failed.
        UnknownAsset,
        /// Operation is not supported for given asset
        Unsupported,
    }

    /// This macro converts the invoked asset type into the respective
    /// implementation that handles it and finally calls the $method on it.
    macro_rules! route_call {
        ($currency_id:expr, $currency_method:ident, $asset_method:ident, $($args:expr),*) => {
            if let Ok(currency) = Currencies::try_from($currency_id) {
                Ok(<T::Currencies as MultiCurrency<T::AccountId>>::$currency_method(currency, $($args),*))
            } else if let Ok(asset) = MarketAsset::try_from($currency_id) {
                Ok(T::MarketAssets::$asset_method(asset, $($args),*))
            } else if let Ok(asset) = CampaignAsset::try_from($currency_id) {
                Ok(T::CampaignAsset::$asset_method(asset, $($args),*))
            } else if let Ok(asset) = CustomAsset::try_from($currency_id)  {
                Ok(T::CustomAsset::$asset_method(asset, $($args),*))
            } else {
                Err(Error::<T>::UnknownAsset)
            }
        };
    }

    /// This macro delegates a call to Currencies if the asset represents a currency, otherwise
    /// it returns an error.
    macro_rules! only_currency {
        ($currency_id:expr, $error:expr, $currency_trait:ident, $currency_method:ident, $($args:expr),+) => {
            if let Ok(currency) = Currencies::try_from($currency_id) {
                <T::Currencies as $currency_trait<T::AccountId>>::$currency_method(currency, $($args),+)
            } else {
                $error
            }
        };
    }

    /// This macro delegates a call to one *Asset instance if the asset does not represent a currency, otherwise
    /// it returns an error.
    macro_rules! only_asset {
        ($asset_id:expr, $error:expr, $asset_trait:ident, $asset_method:ident, $($args:expr),*) => {
            if let Ok(asset) = MarketAsset::try_from($asset_id) {
                <T::MarketAssets as $asset_trait<T::AccountId>>::$asset_method(asset, $($args),*)
            } else if let Ok(asset) = CampaignAsset::try_from($asset_id) {
                T::CampaignAsset::$asset_method(asset, $($args),*)
            } else if let Ok(asset) = CustomAsset::try_from($asset_id)  {
                T::CustomAsset::$asset_method(asset, $($args),*)
            } else {
                $error
            }
        };
    }

    impl<T: Config> MultiCurrency<T::AccountId> for Pallet<T> {
        type CurrencyId = Assets;
        type Balance = T::Balance;

        fn minimum_balance(currency_id: Self::CurrencyId) -> Self::Balance {
            let min_balance = route_call!(currency_id, minimum_balance, minimum_balance,);
            min_balance.unwrap_or(0u8.into())
        }

        fn total_issuance(currency_id: Self::CurrencyId) -> Self::Balance {
            let total_issuance = route_call!(currency_id, total_issuance, total_issuance,);
            total_issuance.unwrap_or(0u8.into())
        }

        fn total_balance(currency_id: Self::CurrencyId, who: &T::AccountId) -> Self::Balance {
            let total_balance = route_call!(currency_id, total_balance, balance, who);
            total_balance.unwrap_or(0u8.into())
        }

        fn free_balance(currency_id: Self::CurrencyId, who: &T::AccountId) -> Self::Balance {
            if let Ok(currency) = Currencies::try_from(currency_id) {
                <T::Currencies as MultiCurrency<T::AccountId>>::free_balance(currency, who)
            } else if let Ok(asset) = MarketAsset::try_from(currency_id) {
                T::MarketAssets::reducible_balance(asset, who, false)
            } else if let Ok(asset) = CampaignAsset::try_from(currency_id) {
                T::CampaignAsset::reducible_balance(asset, who, false)
            } else if let Ok(asset) = CustomAsset::try_from(currency_id) {
                T::CustomAsset::reducible_balance(asset, who, false)
            } else {
                0u8.into()
            }
        }

        fn ensure_can_withdraw(
            currency_id: Self::CurrencyId,
            who: &T::AccountId,
            amount: Self::Balance,
        ) -> DispatchResult {
            if let Ok(currency) = Currencies::try_from(currency_id) {
                return <T::Currencies as MultiCurrency<T::AccountId>>::ensure_can_withdraw(
                    currency, who, amount,
                );
            }

            let withdraw_reason = if let Ok(asset) = MarketAsset::try_from(currency_id) {
                T::MarketAssets::can_withdraw(asset, who, amount)
            } else if let Ok(asset) = CampaignAsset::try_from(currency_id) {
                T::CampaignAsset::can_withdraw(asset, who, amount)
            } else if let Ok(asset) = CustomAsset::try_from(currency_id) {
                T::CustomAsset::can_withdraw(asset, who, amount)
            } else {
                return Err(Error::<T>::UnknownAsset.into());
            };

            withdraw_reason.into_result().map(|_| ())
        }

        fn transfer(
            currency_id: Self::CurrencyId,
            from: &T::AccountId,
            to: &T::AccountId,
            amount: Self::Balance,
        ) -> DispatchResult {
            if let Ok(currency) = Currencies::try_from(currency_id) {
                <T::Currencies as MultiCurrency<T::AccountId>>::transfer(currency, from, to, amount)
            } else if let Ok(asset) = MarketAsset::try_from(currency_id) {
                T::MarketAssets::transfer(asset, from, to, amount, false).map(|_| ())
            } else if let Ok(asset) = CampaignAsset::try_from(currency_id) {
                T::CampaignAsset::transfer(asset, from, to, amount, false).map(|_| ())
            } else if let Ok(asset) = CustomAsset::try_from(currency_id) {
                T::CustomAsset::transfer(asset, from, to, amount, false).map(|_| ())
            } else {
                Err(Error::<T>::UnknownAsset.into())
            }
        }

        fn deposit(
            currency_id: Self::CurrencyId,
            who: &T::AccountId,
            amount: Self::Balance,
        ) -> DispatchResult {
            route_call!(currency_id, deposit, mint_into, who, amount)?
        }

        fn withdraw(
            currency_id: Self::CurrencyId,
            who: &T::AccountId,
            amount: Self::Balance,
        ) -> DispatchResult {
            if let Ok(currency) = Currencies::try_from(currency_id) {
                <T::Currencies as MultiCurrency<T::AccountId>>::withdraw(currency, who, amount)
            } else if let Ok(asset) = MarketAsset::try_from(currency_id) {
                // Resulting balance can be ignored as `burn_from` ensures that the
                // requested amount can be burned.
                T::MarketAssets::burn_from(asset, who, amount).map(|_| ())
            } else if let Ok(asset) = CampaignAsset::try_from(currency_id) {
                T::CampaignAsset::burn_from(asset, who, amount).map(|_| ())
            } else if let Ok(asset) = CustomAsset::try_from(currency_id) {
                T::CustomAsset::burn_from(asset, who, amount).map(|_| ())
            } else {
                Err(Error::<T>::UnknownAsset.into())
            }
        }

        fn can_slash(
            currency_id: Self::CurrencyId,
            who: &T::AccountId,
            value: Self::Balance,
        ) -> bool {
            // TODO
            if let Ok(currency) = Currencies::try_from(currency_id) {
                <T::Currencies as MultiCurrency<T::AccountId>>::can_slash(currency, who, value)
            } else if let Ok(asset) = MarketAsset::try_from(currency_id) {
                T::MarketAssets::reducible_balance(asset, who, false) >= value
            } else if let Ok(asset) = CampaignAsset::try_from(currency_id) {
                T::CampaignAsset::reducible_balance(asset, who, false) >= value
            } else if let Ok(asset) = CustomAsset::try_from(currency_id) {
                T::CustomAsset::reducible_balance(asset, who, false) >= value
            } else {
                false
            }
        }

        fn slash(
            currency_id: Self::CurrencyId,
            who: &T::AccountId,
            amount: Self::Balance,
        ) -> Self::Balance {
            if let Ok(currency) = Currencies::try_from(currency_id) {
                <T::Currencies as MultiCurrency<T::AccountId>>::slash(currency, who, amount)
            } else if let Ok(asset) = MarketAsset::try_from(currency_id) {
                T::MarketAssets::slash(asset, who, amount)
                    .map(|b| amount.saturating_sub(b))
                    .unwrap_or_else(|_| amount)
            } else if let Ok(asset) = CampaignAsset::try_from(currency_id) {
                T::CampaignAsset::slash(asset, who, amount)
                    .map(|b| amount.saturating_sub(b))
                    .unwrap_or_else(|_| amount)
            } else if let Ok(asset) = CustomAsset::try_from(currency_id) {
                T::CustomAsset::slash(asset, who, amount)
                    .map(|b| amount.saturating_sub(b))
                    .unwrap_or_else(|_| amount)
            } else {
                amount
            }
        }
    }

    impl<T: Config> MultiCurrencyExtended<T::AccountId> for Pallet<T> {
        type Amount = <T::Currencies as MultiCurrencyExtended<T::AccountId>>::Amount;

        fn update_balance(
            currency_id: Self::CurrencyId,
            who: &T::AccountId,
            by_amount: Self::Amount,
        ) -> DispatchResult {
            if let Ok(currency) = Currencies::try_from(currency_id) {
                return <T::Currencies as MultiCurrencyExtended<T::AccountId>>::update_balance(
                    currency, who, by_amount,
                );
            }

            if by_amount.is_zero() {
                return Ok(());
            }

            // Ensure this doesn't overflow. There isn't any traits that exposes
            // `saturating_abs` so we need to do it manually.
            let by_amount_abs = if by_amount == Self::Amount::min_value() {
                Self::Amount::max_value()
            } else {
                by_amount.abs()
            };

            let by_balance = TryInto::<Self::Balance>::try_into(by_amount_abs)
                .map_err(|_| Error::<T>::AmountIntoBalanceFailed)?;
            if by_amount.is_positive() {
                Self::deposit(currency_id, who, by_balance)
            } else {
                Self::withdraw(currency_id, who, by_balance).map(|_| ())
            }
        }
    }

    impl<T: Config> MultiLockableCurrency<T::AccountId> for Pallet<T> {
        type Moment = T::BlockNumber;

        fn set_lock(
            lock_id: LockIdentifier,
            currency_id: Self::CurrencyId,
            who: &T::AccountId,
            amount: Self::Balance,
        ) -> DispatchResult {
            if let Ok(currency) = Currencies::try_from(currency_id) {
                return <T::Currencies as MultiLockableCurrency<T::AccountId>>::set_lock(
                    lock_id, currency, who, amount,
                );
            }

            Err(Error::<T>::Unsupported.into())
        }

        fn extend_lock(
            lock_id: LockIdentifier,
            currency_id: Self::CurrencyId,
            who: &T::AccountId,
            amount: Self::Balance,
        ) -> DispatchResult {
            if let Ok(currency) = Currencies::try_from(currency_id) {
                return <T::Currencies as MultiLockableCurrency<T::AccountId>>::extend_lock(
                    lock_id, currency, who, amount,
                );
            }

            Err(Error::<T>::Unsupported.into())
        }

        fn remove_lock(
            lock_id: LockIdentifier,
            currency_id: Self::CurrencyId,
            who: &T::AccountId,
        ) -> DispatchResult {
            if let Ok(currency) = Currencies::try_from(currency_id) {
                return <T::Currencies as MultiLockableCurrency<T::AccountId>>::remove_lock(
                    lock_id, currency, who,
                );
            }

            Err(Error::<T>::Unsupported.into())
        }
    }

    impl<T: Config> MultiReservableCurrency<T::AccountId> for Pallet<T> {
        fn can_reserve(
            currency_id: Self::CurrencyId,
            who: &T::AccountId,
            value: Self::Balance,
        ) -> bool {
            only_currency!(currency_id, false, MultiReservableCurrency, can_reserve, who, value)
        }

        fn slash_reserved(
            currency_id: Self::CurrencyId,
            who: &T::AccountId,
            value: Self::Balance,
        ) -> Self::Balance {
            only_currency!(currency_id, value, MultiReservableCurrency, slash_reserved, who, value)
        }

        fn reserved_balance(currency_id: Self::CurrencyId, who: &T::AccountId) -> Self::Balance {
            only_currency!(
                currency_id,
                Zero::zero(),
                MultiReservableCurrency,
                reserved_balance,
                who
            )
        }

        fn reserve(
            currency_id: Self::CurrencyId,
            who: &T::AccountId,
            value: Self::Balance,
        ) -> DispatchResult {
            only_currency!(
                currency_id,
                Err(Error::<T>::Unsupported.into()),
                MultiReservableCurrency,
                reserve,
                who,
                value
            )
        }

        fn unreserve(
            currency_id: Self::CurrencyId,
            who: &T::AccountId,
            value: Self::Balance,
        ) -> Self::Balance {
            only_currency!(currency_id, value, MultiReservableCurrency, unreserve, who, value)
        }

        fn repatriate_reserved(
            currency_id: Self::CurrencyId,
            slashed: &T::AccountId,
            beneficiary: &T::AccountId,
            value: Self::Balance,
            status: BalanceStatus,
        ) -> Result<Self::Balance, DispatchError> {
            only_currency!(
                currency_id,
                Err(Error::<T>::Unsupported.into()),
                MultiReservableCurrency,
                repatriate_reserved,
                slashed,
                beneficiary,
                value,
                status
            )
        }
    }

    impl<T: Config> NamedMultiReservableCurrency<T::AccountId> for Pallet<T> {
        type ReserveIdentifier =
            <T::Currencies as NamedMultiReservableCurrency<T::AccountId>>::ReserveIdentifier;

        fn reserved_balance_named(
            id: &Self::ReserveIdentifier,
            currency_id: Self::CurrencyId,
            who: &T::AccountId,
        ) -> Self::Balance {
            if let Ok(currency) = Currencies::try_from(currency_id) {
                return <T::Currencies as NamedMultiReservableCurrency<T::AccountId>>::reserved_balance_named(
                    id, currency, who,
                );
            }

            Zero::zero()
        }

        fn reserve_named(
            id: &Self::ReserveIdentifier,
            currency_id: Self::CurrencyId,
            who: &T::AccountId,
            value: Self::Balance,
        ) -> DispatchResult {
            if let Ok(currency) = Currencies::try_from(currency_id) {
                return <T::Currencies as NamedMultiReservableCurrency<T::AccountId>>::reserve_named(
                    id, currency, who, value
                );
            }

            Err(Error::<T>::Unsupported.into())
        }

        fn unreserve_named(
            id: &Self::ReserveIdentifier,
            currency_id: Self::CurrencyId,
            who: &T::AccountId,
            value: Self::Balance,
        ) -> Self::Balance {
            if let Ok(currency) = Currencies::try_from(currency_id) {
                return <T::Currencies as NamedMultiReservableCurrency<T::AccountId>>::unreserve_named(
                    id, currency, who, value
                );
            }

            value
        }

        fn slash_reserved_named(
            id: &Self::ReserveIdentifier,
            currency_id: Self::CurrencyId,
            who: &T::AccountId,
            value: Self::Balance,
        ) -> Self::Balance {
            if let Ok(currency) = Currencies::try_from(currency_id) {
                return <T::Currencies as NamedMultiReservableCurrency<T::AccountId>>::slash_reserved_named(
                    id, currency, who, value
                );
            }

            value
        }

        fn repatriate_reserved_named(
            id: &Self::ReserveIdentifier,
            currency_id: Self::CurrencyId,
            slashed: &T::AccountId,
            beneficiary: &T::AccountId,
            value: Self::Balance,
            status: Status,
        ) -> Result<Self::Balance, DispatchError> {
            if let Ok(currency) = Currencies::try_from(currency_id) {
                return <T::Currencies as NamedMultiReservableCurrency<T::AccountId>>::repatriate_reserved_named(
                    id, currency, slashed, beneficiary, value, status
                );
            }

            Err(Error::<T>::Unsupported.into())
        }
    }

    // Supertrait of Create and Destroy
    impl<T: Config> Inspect<T::AccountId> for Pallet<T> {
        type AssetId = Assets;
        type Balance = T::Balance;

        fn total_issuance(asset: Self::AssetId) -> Self::Balance {
            only_asset!(asset, Zero::zero(), Inspect, total_issuance,)
        }

        fn minimum_balance(asset: Self::AssetId) -> Self::Balance {
            only_asset!(asset, Zero::zero(), Inspect, minimum_balance,)
        }

        fn balance(asset: Self::AssetId, who: &T::AccountId) -> Self::Balance {
            only_asset!(asset, Zero::zero(), Inspect, balance, who)
        }

        fn reducible_balance(
            asset: Self::AssetId,
            who: &T::AccountId,
            keep_alive: bool,
        ) -> Self::Balance {
            only_asset!(asset, Zero::zero(), Inspect, reducible_balance, who, keep_alive)
        }

        fn can_deposit(
            asset: Self::AssetId,
            who: &T::AccountId,
            amount: Self::Balance,
            mint: bool,
        ) -> DepositConsequence {
            only_asset!(
                asset,
                DepositConsequence::UnknownAsset,
                Inspect,
                can_deposit,
                who,
                amount,
                mint
            )
        }

        fn can_withdraw(
            asset: Self::AssetId,
            who: &T::AccountId,
            amount: Self::Balance,
        ) -> WithdrawConsequence<Self::Balance> {
            only_asset!(
                asset,
                WithdrawConsequence::UnknownAsset,
                Inspect,
                can_withdraw,
                who,
                amount
            )
        }

        fn asset_exists(asset: Self::AssetId) -> bool {
            only_asset!(asset, false, Inspect, asset_exists,)
        }
    }

    impl<T: Config> Create<T::AccountId> for Pallet<T> {
        fn create(
            id: Self::AssetId,
            admin: T::AccountId,
            is_sufficient: bool,
            min_balance: Self::Balance,
        ) -> DispatchResult {
            only_asset!(
                id,
                Err(Error::<T>::Unsupported.into()),
                Create,
                create,
                admin,
                is_sufficient,
                min_balance
            )
        }
    }

    impl<T: Config> Destroy<T::AccountId> for Pallet<T> {
        fn start_destroy(
            id: Self::AssetId,
            maybe_check_owner: Option<T::AccountId>,
        ) -> DispatchResult {
            only_asset!(
                id,
                Err(Error::<T>::Unsupported.into()),
                Destroy,
                start_destroy,
                maybe_check_owner
            )
        }

        fn destroy_accounts(id: Self::AssetId, max_items: u32) -> Result<u32, DispatchError> {
            only_asset!(
                id,
                Err(Error::<T>::Unsupported.into()),
                Destroy,
                destroy_accounts,
                max_items
            )
        }

        fn destroy_approvals(id: Self::AssetId, max_items: u32) -> Result<u32, DispatchError> {
            only_asset!(
                id,
                Err(Error::<T>::Unsupported.into()),
                Destroy,
                destroy_approvals,
                max_items
            )
        }

        fn finish_destroy(id: Self::AssetId) -> DispatchResult {
            only_asset!(id, Err(Error::<T>::Unsupported.into()), Destroy, finish_destroy,)
        }
    }
}
