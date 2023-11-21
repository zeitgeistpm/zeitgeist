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

mod mock;
mod tests;

#[frame_support::pallet]
mod pallet {
    use core::marker::PhantomData;
    use frame_support::{
        pallet_prelude::{DispatchError, DispatchResult},
        traits::{
            tokens::{
                fungibles::{Inspect, Mutate, Transfer},
                
            },
            BalanceStatus as Status,
        },
        transactional,
    };
    use sp_runtime::{traits::{Bounded, Zero}, Saturating};
    use orml_traits::{
        arithmetic::Signed,
        currency::{
            MultiCurrency, MultiCurrencyExtended, MultiLockableCurrency, MultiReservableCurrency,
            NamedMultiReservableCurrency, TransferAll,
        },
        BalanceStatus, LockIdentifier,
    };
    use zeitgeist_primitives::types::{
        Assets, CampaignAsset, Currencies, CustomAsset, MarketAsset,
    };

    trait AssetTraits<T: Config>:
        From<pallet_assets::Call<T>>
        + frame_support::dispatch::Dispatchable
        + Inspect<T::AccountId, Balance = T::Balance>
        + Transfer<T::AccountId, Balance = T::Balance>
        + Mutate<T::AccountId, Balance = T::Balance>
    {
    }

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_assets::Config {
        type Currencies: TransferAll<Self::AccountId>
            + MultiCurrencyExtended<Self::AccountId, CurrencyId = Currencies, Balance = Self::Balance>
            + MultiLockableCurrency<Self::AccountId>
            + MultiReservableCurrency<Self::AccountId>
            + NamedMultiReservableCurrency<Self::AccountId>;
        type CampaignAsset: AssetTraits<Self, AssetId = CampaignAsset>;
        type CustomAsset: AssetTraits<Self, AssetId = CustomAsset>;
        type MarketAssets: AssetTraits<Self, AssetId = MarketAsset>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

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
        ($currency_id:expr, $currency_method:ident, $asset_method:ident) => {
            if let Ok(currency) = Currencies::try_from($currency_id) {
                Ok(<T::Currencies as MultiCurrency<T::AccountId>>::$currency_method(currency))
            } else if let Ok(asset) = MarketAsset::try_from($currency_id) {
                Ok(<T as Config>::MarketAssets::$asset_method(asset))
            } else if let Ok(asset) = CampaignAsset::try_from($currency_id) {
                Ok(<T as Config>::CampaignAsset::$asset_method(asset))
            } else if let Ok(asset) = CustomAsset::try_from($currency_id)  {
                Ok(<T as Config>::CustomAsset::$asset_method(asset))
            } else {
                Err(Error::<T>::UnknownAsset)
            }
        };

        ($currency_id:expr, $currency_method:ident, $asset_method:ident, $($args:expr),+) => {
            if let Ok(currency) = Currencies::try_from($currency_id) {
                Ok(<T::Currencies as MultiCurrency<T::AccountId>>::$currency_method(currency, $($args),+))
            } else if let Ok(asset) = MarketAsset::try_from($currency_id) {
                Ok(<T as Config>::MarketAssets::$asset_method(asset, $($args),+))
            } else if let Ok(asset) = CampaignAsset::try_from($currency_id) {
                Ok(<T as Config>::CampaignAsset::$asset_method(asset, $($args),+))
            } else if let Ok(asset) = CustomAsset::try_from($currency_id)  {
                Ok(<T as Config>::CustomAsset::$asset_method(asset, $($args),+))
            } else {
                Err(Error::<T>::UnknownAsset)
            }
        };
    }

    // This macro delegates a call to currencies if the asset represents a currency, otherwise
    // It returns an error
    macro_rules! only_currency {
        ($currency_id:expr, $currency_trait:ident, $currency_method:ident, $($args:expr),+) => {
            if let Ok(currency) = Currencies::try_from($currency_id) {
                return <T::Currencies as $currency_trait<T::AccountId>>::$currency_method(currency, $($args),+);
            }
                
            Err(Error::<T>::Unsupported.into())
        };
    }

    impl<T: Config> MultiCurrency<T::AccountId> for Pallet<T> {
        type CurrencyId = Assets;
        type Balance = T::Balance;

        fn minimum_balance(currency_id: Self::CurrencyId) -> Self::Balance {
            let min_balance = route_call!(currency_id, minimum_balance, minimum_balance);
            min_balance.unwrap_or(0u8.into())
        }

        fn total_issuance(currency_id: Self::CurrencyId) -> Self::Balance {
            let total_issuance = route_call!(currency_id, total_issuance, total_issuance);
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
                <T as Config>::MarketAssets::reducible_balance(asset, who, false)
            } else if let Ok(asset) = CampaignAsset::try_from(currency_id) {
                <T as Config>::CampaignAsset::reducible_balance(asset, who, false)
            } else if let Ok(asset) = CustomAsset::try_from(currency_id) {
                <T as Config>::CustomAsset::reducible_balance(asset, who, false)
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
                <T as Config>::MarketAssets::can_withdraw(asset, who, amount)
            } else if let Ok(asset) = CampaignAsset::try_from(currency_id) {
                <T as Config>::CampaignAsset::can_withdraw(asset, who, amount)
            } else if let Ok(asset) = CustomAsset::try_from(currency_id) {
                <T as Config>::CustomAsset::can_withdraw(asset, who, amount)
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
                <T as Config>::MarketAssets::transfer(asset, from, to, amount, false).map(|_| ())
            } else if let Ok(asset) = CampaignAsset::try_from(currency_id) {
                <T as Config>::CampaignAsset::transfer(asset, from, to, amount, false).map(|_| ())
            } else if let Ok(asset) = CustomAsset::try_from(currency_id) {
                <T as Config>::CustomAsset::transfer(asset, from, to, amount, false).map(|_| ())
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
                <T::Currencies as MultiCurrency<T::AccountId>>::deposit(currency, who, amount)
            } else if let Ok(asset) = MarketAsset::try_from(currency_id) {
                // Resulting balance can be ignored as `burn_from` ensures that the
                // requested amount can be burned.
                <T as Config>::MarketAssets::burn_from(asset, who, amount).map(|_| ())
            } else if let Ok(asset) = CampaignAsset::try_from(currency_id) {
                <T as Config>::CampaignAsset::burn_from(asset, who, amount).map(|_| ())
            } else if let Ok(asset) = CustomAsset::try_from(currency_id) {
                <T as Config>::CustomAsset::burn_from(asset, who, amount).map(|_| ())
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
                <T as Config>::MarketAssets::reducible_balance(asset, who, false) >= value
            } else if let Ok(asset) = CampaignAsset::try_from(currency_id) {
                <T as Config>::CampaignAsset::reducible_balance(asset, who, false) >= value
            } else if let Ok(asset) = CustomAsset::try_from(currency_id) {
                <T as Config>::CustomAsset::reducible_balance(asset, who, false) >= value
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
                <T as Config>::MarketAssets::slash(asset, who, amount)
                    .map(|b| amount.saturating_sub(b))
                    .unwrap_or_else(|_| amount)
            } else if let Ok(asset) = CampaignAsset::try_from(currency_id) {
                <T as Config>::CampaignAsset::slash(asset, who, amount)
                    .map(|b| amount.saturating_sub(b))
                    .unwrap_or_else(|_| amount)
            } else if let Ok(asset) = CustomAsset::try_from(currency_id) {
                <T as Config>::CustomAsset::slash(asset, who, amount)
                    .map(|b| amount.saturating_sub(b))
                    .unwrap_or_else(|_| amount)
            } else {
                amount
            }
        }
    }

    impl<T: Config> MultiCurrencyExtended<T::AccountId> for Pallet<T>
    {
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
                return <T::Currencies as MultiLockableCurrency<T::AccountId>>::set_lock(lock_id, currency, who, amount);
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
                return <T::Currencies as MultiLockableCurrency<T::AccountId>>::extend_lock(lock_id, currency, who, amount);
            }

            Err(Error::<T>::Unsupported.into())
        }

        fn remove_lock(
            lock_id: LockIdentifier,
            currency_id: Self::CurrencyId,
            who: &T::AccountId,
        ) -> DispatchResult {
            if let Ok(currency) = Currencies::try_from(currency_id) {
                return <T::Currencies as MultiLockableCurrency<T::AccountId>>::remove_lock(lock_id, currency, who);
            }

            Err(Error::<T>::Unsupported.into())
        }
    }

    impl<T: Config> MultiReservableCurrency<T::AccountId> for Pallet<T> {
        /// Check if `who` can reserve `value` from their free balance.
        ///
        /// Always `true` if value to be reserved is zero.
        fn can_reserve(
            currency_id: Self::CurrencyId,
            who: &T::AccountId,
            value: Self::Balance,
        ) -> bool {
            // TODO
            true
        }

        /// Slash from reserved balance, returning any amount that was unable to
        /// be slashed.
        ///
        /// Is a no-op if the value to be slashed is zero.
        fn slash_reserved(
            currency_id: Self::CurrencyId,
            who: &T::AccountId,
            value: Self::Balance,
        ) -> Self::Balance {
            // TODO
            0u8.into()
        }

        fn reserved_balance(currency_id: Self::CurrencyId, who: &T::AccountId) -> Self::Balance {
            // TODO
            0u8.into()
        }

        /// Move `value` from the free balance from `who` to their reserved
        /// balance.
        ///
        /// Is a no-op if value to be reserved is zero.
        fn reserve(
            currency_id: Self::CurrencyId,
            who: &T::AccountId,
            value: Self::Balance,
        ) -> DispatchResult {
            // TODO
            Ok(())
        }

        /// Unreserve some funds, returning any amount that was unable to be
        /// unreserved.
        ///
        /// Is a no-op if the value to be unreserved is zero.
        fn unreserve(
            currency_id: Self::CurrencyId,
            who: &T::AccountId,
            value: Self::Balance,
        ) -> Self::Balance {
            // TODO
            0u8.into()
        }

        /// Move the reserved balance of one account into the balance of
        /// another, according to `status`.
        ///
        /// Is a no-op if:
        /// - the value to be moved is zero; or
        /// - the `slashed` id equal to `beneficiary` and the `status` is
        ///   `Reserved`.
        fn repatriate_reserved(
            currency_id: Self::CurrencyId,
            slashed: &T::AccountId,
            beneficiary: &T::AccountId,
            value: Self::Balance,
            status: BalanceStatus,
        ) -> Result<Self::Balance, DispatchError> {
            // TODO
            Ok(0u8.into())
        }
    }

    impl<T: Config> NamedMultiReservableCurrency<T::AccountId> for Pallet<T> {
        type ReserveIdentifier = [u8; 8];

        fn reserved_balance_named(
            id: &Self::ReserveIdentifier,
            currency_id: Self::CurrencyId,
            who: &T::AccountId,
        ) -> Self::Balance {
            //TODO
            0u8.into()
        }

        /// Move `value` from the free balance from `who` to a named reserve
        /// balance.
        ///
        /// Is a no-op if value to be reserved is zero.
        fn reserve_named(
            id: &Self::ReserveIdentifier,
            currency_id: Self::CurrencyId,
            who: &T::AccountId,
            value: Self::Balance,
        ) -> DispatchResult {
            //TODO
            Ok(())
        }

        /// Unreserve some funds, returning any amount that was unable to be
        /// unreserved.
        ///
        /// Is a no-op if the value to be unreserved is zero.
        fn unreserve_named(
            id: &Self::ReserveIdentifier,
            currency_id: Self::CurrencyId,
            who: &T::AccountId,
            value: Self::Balance,
        ) -> Self::Balance {
            //TODO
            0u8.into()
        }

        /// Slash from reserved balance, returning the amount that was unable to be
        /// slashed.
        ///
        /// Is a no-op if the value to be slashed is zero.
        fn slash_reserved_named(
            id: &Self::ReserveIdentifier,
            currency_id: Self::CurrencyId,
            who: &T::AccountId,
            value: Self::Balance,
        ) -> Self::Balance {
            //TODO
            0u8.into()
        }

        /// Move the reserved balance of one account into the balance of another,
        /// according to `status`. If `status` is `Reserved`, the balance will be
        /// reserved with given `id`.
        ///
        /// Is a no-op if:
        /// - the value to be moved is zero; or
        /// - the `slashed` id equal to `beneficiary` and the `status` is
        ///   `Reserved`.
        fn repatriate_reserved_named(
            id: &Self::ReserveIdentifier,
            currency_id: Self::CurrencyId,
            slashed: &T::AccountId,
            beneficiary: &T::AccountId,
            value: Self::Balance,
            status: Status,
        ) -> Result<Self::Balance, DispatchError> {
            //TODO
            Ok(0u8.into())
        }
    }
}
