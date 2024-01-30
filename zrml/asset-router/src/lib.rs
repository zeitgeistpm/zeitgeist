// Copyright 2023-2024 Forecasting Technologies LTD.
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
    use alloc::collections::BTreeMap;
    use core::{cmp::Ordering, fmt::Debug, marker::PhantomData};
    use frame_support::{
        log,
        pallet_prelude::{DispatchError, DispatchResult, Hooks, StorageValue, ValueQuery, Weight},
        traits::{
            tokens::{
                fungibles::{Create, Destroy, Inspect, Mutate, Transfer},
                DepositConsequence, WithdrawConsequence,
            },
            BalanceStatus as Status, ConstU32,
        },
        transactional, BoundedVec, Parameter,
    };
    use orml_traits::{
        arithmetic::Signed,
        currency::{
            MultiCurrency, MultiCurrencyExtended, MultiLockableCurrency, MultiReservableCurrency,
            NamedMultiReservableCurrency, TransferAll,
        },
        BalanceStatus, LockIdentifier,
    };
    use pallet_assets::ManagedDestroy;
    use parity_scale_codec::{Decode, Encode, FullCodec, MaxEncodedLen};
    use scale_info::TypeInfo;
    use sp_runtime::{
        traits::{
            AtLeast32BitUnsigned, Bounded, Get, MaybeSerializeDeserialize, Member, Saturating, Zero,
        },
        FixedPointOperand, SaturatedConversion,
    };
    use zeitgeist_primitives::traits::CheckedDivPerComponent;

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
        /// The overarching asset type that contains all assets classes.
        type AssetType: Copy
            + Debug
            + Eq
            + From<Self::CurrencyType>
            + From<Self::CampaignAssetType>
            + From<Self::CustomAssetType>
            + From<Self::MarketAssetType>
            + FullCodec
            + MaxEncodedLen
            + MaybeSerializeDeserialize
            + Ord
            + TypeInfo;

        /// The type that represents balances.
        type Balance: Parameter
            + Member
            + AtLeast32BitUnsigned
            + Default
            + Copy
            + MaybeSerializeDeserialize
            + MaxEncodedLen
            + FixedPointOperand;

        /// Logic that handles campaign assets by providing multiple fungible
        /// trait implementations.
        type CampaignAssets: AssetTraits<Self, Self::CampaignAssetType>;
        /// The custom asset type.
        type CampaignAssetType: TryFrom<Self::AssetType>
            + Copy
            + Debug
            + Eq
            + FullCodec
            + MaxEncodedLen
            + MaybeSerializeDeserialize
            + TypeInfo;

        /// Logic that handles currencies by providing multiple currencies
        /// trait implementations.
        type Currencies: TransferAll<Self::AccountId>
            + MultiCurrencyExtended<
                Self::AccountId,
                CurrencyId = Self::CurrencyType,
                Balance = Self::Balance,
            > + MultiLockableCurrency<Self::AccountId>
            + MultiReservableCurrency<Self::AccountId>
            + NamedMultiReservableCurrency<Self::AccountId>;
        /// The currency type.
        type CurrencyType: TryFrom<Self::AssetType>
            + Copy
            + Debug
            + Eq
            + FullCodec
            + MaxEncodedLen
            + MaybeSerializeDeserialize
            + TypeInfo;

        /// Logic that handles custom assets by providing multiple fungible
        /// trait implementations.
        type CustomAssets: AssetTraits<Self, Self::CustomAssetType>;
        /// The custom asset type.
        type CustomAssetType: TryFrom<Self::AssetType>
            + Copy
            + Debug
            + Eq
            + FullCodec
            + MaxEncodedLen
            + MaybeSerializeDeserialize
            + TypeInfo;

        /// Weight required for destroying one account.
        type DestroyAccountWeight: Get<Weight>;
        /// Weight required for destroying one approval.
        type DestroyApprovalWeight: Get<Weight>;
        /// Weight required for finishing the asset destruction process.
        type DestroyFinishWeight: Get<Weight>;

        /// Logic that handles market assets by providing multiple fungible
        /// trait implementations.
        type MarketAssets: AssetTraits<Self, Self::MarketAssetType>;
        /// The market asset type.
        type MarketAssetType: TryFrom<Self::AssetType>
            + Copy
            + Debug
            + Eq
            + FullCodec
            + MaxEncodedLen
            + MaybeSerializeDeserialize
            + TypeInfo;
    }

    const LOG_TARGET: &str = "runtime::asset-router";

    /// Keeps track of assets that have to be destroyed.
    #[pallet::storage]
    pub(super) type DestroyAssets<T: Config> =
        StorageValue<_, BoundedVec<AssetInDestruction<T::AssetType>, ConstU32<8192>>, ValueQuery>;

    /// Keeps track of assets that can't be destroyed.
    #[pallet::storage]
    pub(crate) type IndestructibleAssets<T: Config> =
        StorageValue<_, BoundedVec<T::AssetType, ConstU32<8192>>, ValueQuery>;

    #[pallet::error]
    pub enum Error<T> {
        /// Cannot convert Amount (MultiCurrencyExtended implementation) into Balance type.
        AmountIntoBalanceFailed,
        /// Cannot start managed destruction as a destruction for the asset is in progress.
        DestructionInProgress,
        /// The vector holding all assets to destroy reached it's boundary.
        TooManyManagedDestroys,
        /// Asset conversion failed.
        UnknownAsset,
        /// Operation is not supported for given asset
        Unsupported,
    }

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    /// This macro handles the single stages of the asset destruction.
    macro_rules! handle_asset_destruction {
        ($asset:expr, $remaining_weight:expr, $asset_storage:expr, $asset_method:ident) => {
            let state_before = *($asset.state());
            let call_result = Self::$asset_method($asset, $remaining_weight);
            $remaining_weight = call_result.map_or_else(|weight| weight, |weight| weight);

            // In case destruction failed during finalization, there is most likely still
            // some weight available.
            if call_result.is_err() && state_before != DestructionState::Finalization {
                break;
            }

            if *($asset.state()) == state_before {
                // Should be infallible since the asset was just poppoed and force inserting
                // is not possible.
                if let Err(e) = $asset_storage.try_insert(0, *($asset)) {
                    log::error!(
                        target: LOG_TARGET,
                        "Cannot reintroduce asset {:?} into DestroyAssets storage: {:?}",
                        $asset,
                        e
                    );
                }

                break;
            }
        };
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
        fn on_idle(_: T::BlockNumber, mut remaining_weight: Weight) -> Weight {
            let mut destroy_assets = DestroyAssets::<T>::get();

            if destroy_assets.len() == 0 {
                return remaining_weight.saturating_sub(T::DbWeight::get().reads(1));
            }

            remaining_weight =
                remaining_weight.saturating_sub(T::DbWeight::get().reads_writes(1, 1));

            while let Some(mut asset) = destroy_assets.pop() {
                // TODO loop
                match asset.state() {
                    DestructionState::Accounts => {
                        handle_asset_destruction!(
                            &mut asset,
                            remaining_weight,
                            destroy_assets,
                            handle_destroy_accounts
                        );
                    }
                    DestructionState::Approvals => {
                        handle_asset_destruction!(
                            &mut asset,
                            remaining_weight,
                            destroy_assets,
                            handle_destroy_approvals
                        );
                    }
                    DestructionState::Finalization => {
                        handle_asset_destruction!(
                            &mut asset,
                            remaining_weight,
                            destroy_assets,
                            handle_destroy_finish
                        );
                    }
                    // Next two states should never occur. Just remove the asset.
                    DestructionState::Destroyed => {
                        log::warn!(target: LOG_TARGET, "Asset {:?} has invalid state", asset);
                    }
                    DestructionState::Indestructible => {
                        log::warn!(target: LOG_TARGET, "Asset {:?} has invalid state", asset);
                    }
                }
            }

            DestroyAssets::<T>::put(destroy_assets);
            remaining_weight
        }
    }

    /// This macro converts the invoked asset type into the respective
    /// implementation that handles it and finally calls the $method on it.
    macro_rules! route_call {
        ($currency_id:expr, $currency_method:ident, $asset_method:ident, $($args:expr),*) => {
            if let Ok(currency) = T::CurrencyType::try_from($currency_id) {
                Ok(<T::Currencies as MultiCurrency<T::AccountId>>::$currency_method(currency, $($args),*))
            } else if let Ok(asset) = T::MarketAssetType::try_from($currency_id) {
                Ok(T::MarketAssets::$asset_method(asset, $($args),*))
            } else if let Ok(asset) = T::CampaignAssetType::try_from($currency_id) {
                Ok(T::CampaignAssets::$asset_method(asset, $($args),*))
            } else if let Ok(asset) = T::CustomAssetType::try_from($currency_id)  {
                Ok(T::CustomAssets::$asset_method(asset, $($args),*))
            } else {
                Err(Error::<T>::UnknownAsset)
            }
        };
    }

    /// This macro delegates a call to Currencies if the asset represents a currency, otherwise
    /// it returns an error.
    macro_rules! only_currency {
        ($currency_id:expr, $error:expr, $currency_trait:ident, $currency_method:ident, $($args:expr),+) => {
            if let Ok(currency) = T::CurrencyType::try_from($currency_id) {
                <T::Currencies as $currency_trait<T::AccountId>>::$currency_method(currency, $($args),+)
            } else {
                Self::log_unsupported($currency_id, stringify!($currency_method));
                $error
            }
        };
    }

    /// This macro delegates a call to one *Asset instance if the asset does not represent a currency, otherwise
    /// it returns an error.
    macro_rules! only_asset {
        ($asset_id:expr, $error:expr, $asset_trait:ident, $asset_method:ident, $($args:expr),*) => {
            if let Ok(asset) = T::MarketAssetType::try_from($asset_id) {
                <T::MarketAssets as $asset_trait<T::AccountId>>::$asset_method(asset, $($args),*)
            } else if let Ok(asset) = T::CampaignAssetType::try_from($asset_id) {
                T::CampaignAssets::$asset_method(asset, $($args),*)
            } else if let Ok(asset) = T::CustomAssetType::try_from($asset_id)  {
                T::CustomAssets::$asset_method(asset, $($args),*)
            } else {
                Self::log_unsupported($asset_id, stringify!($asset_method));
                $error
            }
        };
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq, Decode, Encode, MaxEncodedLen, TypeInfo)]
    pub(crate) enum DestructionState {
        Accounts,
        Approvals,
        Destroyed,
        Finalization,
        Indestructible,
    }

    #[derive(Clone, Copy, Encode, Debug, Decode, MaxEncodedLen, TypeInfo)]
    pub(crate) struct AssetInDestruction<A> {
        asset: A,
        state: DestructionState,
    }

    // Ordering hack for binary search of assets in destruction.
    impl<A> PartialEq for AssetInDestruction<A>
    where
        A: Eq + Ord + PartialEq + PartialOrd,
    {
        fn eq(&self, other: &Self) -> bool {
            self.asset == other.asset
        }
    }

    impl<A> Eq for AssetInDestruction<A> where A: Eq + Ord + PartialEq + PartialOrd {}

    impl<A> PartialOrd for AssetInDestruction<A>
    where
        A: Eq + Ord + PartialEq + PartialOrd,
    {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            self.asset.partial_cmp(&other.asset)
        }
    }

    impl<A> Ord for AssetInDestruction<A>
    where
        A: Eq + Ord + PartialEq + PartialOrd,
    {
        fn cmp(&self, other: &Self) -> Ordering {
            self.asset.cmp(&other.asset)
        }
    }

    impl<A> AssetInDestruction<A> {
        pub(crate) fn new(asset: A) -> Self {
            AssetInDestruction { asset, state: DestructionState::Accounts }
        }

        pub(crate) fn asset(&self) -> &A {
            &self.asset
        }

        pub(crate) fn state(&self) -> &DestructionState {
            &self.state
        }

        pub(crate) fn transit_indestructible(&mut self) {
            self.state = DestructionState::Indestructible;
        }

        // Returns the new state on change, None otherwise
        pub(crate) fn transit_state(&mut self) -> Option<&DestructionState> {
            let state_before = self.state;

            self.state = match self.state {
                DestructionState::Accounts => DestructionState::Approvals,
                DestructionState::Approvals => DestructionState::Finalization,
                DestructionState::Destroyed => DestructionState::Destroyed,
                DestructionState::Finalization => DestructionState::Destroyed,
                DestructionState::Indestructible => DestructionState::Indestructible,
            };

            if state_before != self.state { Some(&self.state) } else { None }
        }
    }

    impl<T: Config> Pallet<T> {
        fn mark_asset_as_indestructible(
            asset: &mut AssetInDestruction<T::AssetType>,
            mut remaining_weight: Weight,
            max_weight: Weight,
            error: DispatchError,
        ) -> Weight {
            let asset_inner = *asset.asset();

            log::error!(
                target: LOG_TARGET,
                "Error during managed asset account destruction of {:?}: {:?}",
                asset_inner,
                error
            );

            remaining_weight = remaining_weight.saturating_sub(max_weight);

            if let Err(e) = IndestructibleAssets::<T>::try_mutate(|assets| {
                let idx = assets.partition_point(|&asset_in_vec| asset_in_vec < asset_inner);
                assets.try_insert(idx, asset_inner)
            }) {
                log::error!(
                    target: LOG_TARGET,
                    "Error during storage of indestructible asset {:?}, dropping asset: {:?}",
                    asset_inner,
                    e
                );
            }

            asset.transit_indestructible();
            remaining_weight.saturating_sub(T::DbWeight::get().reads_writes(1, 1))
        }

        fn handle_destroy_accounts(
            asset: &mut AssetInDestruction<T::AssetType>,
            mut remaining_weight: Weight,
        ) -> Result<Weight, Weight> {
            if *asset.state() != DestructionState::Accounts {
                return Ok(remaining_weight);
            }
            let destroy_account_weight = T::DestroyAccountWeight::get();

            let destroy_account_cap =
                match remaining_weight.checked_div_per_component(&destroy_account_weight) {
                    Some(amount) => amount,
                    None => return Ok(remaining_weight),
                };

            match Self::destroy_accounts(*asset.asset(), destroy_account_cap.saturated_into()) {
                Ok(destroyed_accounts) => {
                    // TODO(#1202): More precise weights
                    remaining_weight = remaining_weight.saturating_sub(
                        destroy_account_weight
                            .saturating_mul(destroyed_accounts.into())
                            .max(destroy_account_weight),
                    );

                    if u64::from(destroyed_accounts) != destroy_account_cap {
                        asset.transit_state();
                    }

                    Ok(remaining_weight)
                }
                Err(e) => {
                    // In this case, it is not known how many accounts have been destroyed prior
                    // to triggering this error. The only safe handling is consuming all the
                    // remaining weight.
                    let remaining_weight_err = Self::mark_asset_as_indestructible(
                        asset,
                        remaining_weight,
                        destroy_account_weight.saturating_mul(destroy_account_cap),
                        e,
                    );
                    Err(remaining_weight_err)
                }
            }
        }

        fn handle_destroy_approvals(
            asset: &mut AssetInDestruction<T::AssetType>,
            mut remaining_weight: Weight,
        ) -> Result<Weight, Weight> {
            if *asset.state() != DestructionState::Approvals {
                return Ok(remaining_weight);
            }
            let destroy_approval_weight = T::DestroyAccountWeight::get();

            let destroy_approval_cap =
                match remaining_weight.checked_div_per_component(&destroy_approval_weight) {
                    Some(amount) => amount,
                    None => return Ok(remaining_weight),
                };

            match Self::destroy_approvals(*asset.asset(), destroy_approval_cap.saturated_into()) {
                Ok(destroyed_approvals) => {
                    // TODO(#1202): More precise weights
                    remaining_weight = remaining_weight.saturating_sub(
                        destroy_approval_weight
                            .saturating_mul(destroyed_approvals.into())
                            .max(destroy_approval_weight),
                    );

                    if u64::from(destroyed_approvals) != destroy_approval_cap {
                        asset.transit_state();
                    }

                    Ok(remaining_weight)
                }
                Err(e) => {
                    // In this case, it is not known how many approvals have been destroyed prior
                    // to triggering this error. The only safe handling is consuming all the
                    // remaining weight.
                    let remaining_weight_err = Self::mark_asset_as_indestructible(
                        asset,
                        remaining_weight,
                        destroy_approval_weight.saturating_mul(destroy_approval_cap),
                        e,
                    );
                    Err(remaining_weight_err)
                }
            }
        }

        fn handle_destroy_finish(
            asset: &mut AssetInDestruction<T::AssetType>,
            remaining_weight: Weight,
        ) -> Result<Weight, Weight> {
            if *asset.state() != DestructionState::Approvals {
                return Ok(remaining_weight);
            }
            let destroy_finish_weight = T::DestroyFinishWeight::get();

            if remaining_weight.all_gte(destroy_finish_weight) {
                // TODO(#1202): More precise weights
                if let Err(e) = Self::finish_destroy(*asset.asset()) {
                    let remaining_weight_err = Self::mark_asset_as_indestructible(
                        asset,
                        remaining_weight,
                        destroy_finish_weight,
                        e,
                    );
                    return Err(remaining_weight_err);
                }
            }

            Ok(remaining_weight.saturating_sub(destroy_finish_weight))
        }

        #[inline]
        fn log_unsupported(asset: T::AssetType, function: &str) {
            log::warn!(target: LOG_TARGET, "Asset {:?} not supported in function {:?}", asset, function);
        }
    }

    impl<T: Config> TransferAll<T::AccountId> for Pallet<T> {
        #[transactional]
        fn transfer_all(source: &T::AccountId, dest: &T::AccountId) -> DispatchResult {
            // Only transfers assets maintained in orml-tokens, not implementable for pallet-assets
            <T::Currencies as TransferAll<T::AccountId>>::transfer_all(source, dest)
        }
    }

    impl<T: Config> MultiCurrency<T::AccountId> for Pallet<T> {
        type CurrencyId = T::AssetType;
        type Balance = T::Balance;

        fn minimum_balance(currency_id: Self::CurrencyId) -> Self::Balance {
            let min_balance = route_call!(currency_id, minimum_balance, minimum_balance,);
            min_balance.unwrap_or_else(|_b| {
                Self::log_unsupported(currency_id, "minimum_balance");
                Self::Balance::zero()
            })
        }

        fn total_issuance(currency_id: Self::CurrencyId) -> Self::Balance {
            let total_issuance = route_call!(currency_id, total_issuance, total_issuance,);
            total_issuance.unwrap_or_else(|_b| {
                Self::log_unsupported(currency_id, "total_issuance");
                Self::Balance::zero()
            })
        }

        fn total_balance(currency_id: Self::CurrencyId, who: &T::AccountId) -> Self::Balance {
            let total_balance = route_call!(currency_id, total_balance, balance, who);
            total_balance.unwrap_or_else(|_b| {
                Self::log_unsupported(currency_id, "total_balance");
                Self::Balance::zero()
            })
        }

        fn free_balance(currency_id: Self::CurrencyId, who: &T::AccountId) -> Self::Balance {
            if let Ok(currency) = T::CurrencyType::try_from(currency_id) {
                <T::Currencies as MultiCurrency<T::AccountId>>::free_balance(currency, who)
            } else if let Ok(asset) = T::MarketAssetType::try_from(currency_id) {
                T::MarketAssets::reducible_balance(asset, who, false)
            } else if let Ok(asset) = T::CampaignAssetType::try_from(currency_id) {
                T::CampaignAssets::reducible_balance(asset, who, false)
            } else if let Ok(asset) = T::CustomAssetType::try_from(currency_id) {
                T::CustomAssets::reducible_balance(asset, who, false)
            } else {
                Self::log_unsupported(currency_id, "free_balance");
                Self::Balance::zero()
            }
        }

        fn ensure_can_withdraw(
            currency_id: Self::CurrencyId,
            who: &T::AccountId,
            amount: Self::Balance,
        ) -> DispatchResult {
            if let Ok(currency) = T::CurrencyType::try_from(currency_id) {
                return <T::Currencies as MultiCurrency<T::AccountId>>::ensure_can_withdraw(
                    currency, who, amount,
                );
            }

            let withdraw_consequence = if let Ok(asset) = T::MarketAssetType::try_from(currency_id)
            {
                T::MarketAssets::can_withdraw(asset, who, amount)
            } else if let Ok(asset) = T::CampaignAssetType::try_from(currency_id) {
                T::CampaignAssets::can_withdraw(asset, who, amount)
            } else if let Ok(asset) = T::CustomAssetType::try_from(currency_id) {
                T::CustomAssets::can_withdraw(asset, who, amount)
            } else {
                return Err(Error::<T>::UnknownAsset.into());
            };

            withdraw_consequence.into_result().map(|_| ())
        }

        fn transfer(
            currency_id: Self::CurrencyId,
            from: &T::AccountId,
            to: &T::AccountId,
            amount: Self::Balance,
        ) -> DispatchResult {
            if let Ok(currency) = T::CurrencyType::try_from(currency_id) {
                <T::Currencies as MultiCurrency<T::AccountId>>::transfer(currency, from, to, amount)
            } else if let Ok(asset) = T::MarketAssetType::try_from(currency_id) {
                T::MarketAssets::transfer(asset, from, to, amount, false).map(|_| ())
            } else if let Ok(asset) = T::CampaignAssetType::try_from(currency_id) {
                T::CampaignAssets::transfer(asset, from, to, amount, false).map(|_| ())
            } else if let Ok(asset) = T::CustomAssetType::try_from(currency_id) {
                T::CustomAssets::transfer(asset, from, to, amount, false).map(|_| ())
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
            if let Ok(currency) = T::CurrencyType::try_from(currency_id) {
                <T::Currencies as MultiCurrency<T::AccountId>>::withdraw(currency, who, amount)
            } else if let Ok(asset) = T::MarketAssetType::try_from(currency_id) {
                // Resulting balance can be ignored as `burn_from` ensures that the
                // requested amount can be burned.
                T::MarketAssets::burn_from(asset, who, amount).map(|_| ())
            } else if let Ok(asset) = T::CampaignAssetType::try_from(currency_id) {
                T::CampaignAssets::burn_from(asset, who, amount).map(|_| ())
            } else if let Ok(asset) = T::CustomAssetType::try_from(currency_id) {
                T::CustomAssets::burn_from(asset, who, amount).map(|_| ())
            } else {
                Err(Error::<T>::UnknownAsset.into())
            }
        }

        fn can_slash(
            currency_id: Self::CurrencyId,
            who: &T::AccountId,
            value: Self::Balance,
        ) -> bool {
            if let Ok(currency) = T::CurrencyType::try_from(currency_id) {
                <T::Currencies as MultiCurrency<T::AccountId>>::can_slash(currency, who, value)
            } else if let Ok(asset) = T::MarketAssetType::try_from(currency_id) {
                T::MarketAssets::reducible_balance(asset, who, false) >= value
            } else if let Ok(asset) = T::CampaignAssetType::try_from(currency_id) {
                T::CampaignAssets::reducible_balance(asset, who, false) >= value
            } else if let Ok(asset) = T::CustomAssetType::try_from(currency_id) {
                T::CustomAssets::reducible_balance(asset, who, false) >= value
            } else {
                Self::log_unsupported(currency_id, "can_slash");
                false
            }
        }

        fn slash(
            currency_id: Self::CurrencyId,
            who: &T::AccountId,
            amount: Self::Balance,
        ) -> Self::Balance {
            if let Ok(currency) = T::CurrencyType::try_from(currency_id) {
                <T::Currencies as MultiCurrency<T::AccountId>>::slash(currency, who, amount)
            } else if let Ok(asset) = T::MarketAssetType::try_from(currency_id) {
                T::MarketAssets::slash(asset, who, amount)
                    .map(|b| amount.saturating_sub(b))
                    .unwrap_or_else(|_| amount)
            } else if let Ok(asset) = T::CampaignAssetType::try_from(currency_id) {
                T::CampaignAssets::slash(asset, who, amount)
                    .map(|b| amount.saturating_sub(b))
                    .unwrap_or_else(|_| amount)
            } else if let Ok(asset) = T::CustomAssetType::try_from(currency_id) {
                T::CustomAssets::slash(asset, who, amount)
                    .map(|b| amount.saturating_sub(b))
                    .unwrap_or_else(|_| amount)
            } else {
                Self::log_unsupported(currency_id, "slash");
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
            if let Ok(currency) = T::CurrencyType::try_from(currency_id) {
                return <T::Currencies as MultiCurrencyExtended<T::AccountId>>::update_balance(
                    currency, who, by_amount,
                );
            }

            if by_amount.is_zero() {
                return Ok(());
            }

            // Ensure that no overflows happen during abs().
            let by_amount_abs = if by_amount == Self::Amount::min_value() {
                return Err(Error::<T>::AmountIntoBalanceFailed.into());
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
            if let Ok(currency) = T::CurrencyType::try_from(currency_id) {
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
            if let Ok(currency) = T::CurrencyType::try_from(currency_id) {
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
            if let Ok(currency) = T::CurrencyType::try_from(currency_id) {
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
            if let Ok(currency) = T::CurrencyType::try_from(currency_id) {
                return <T::Currencies as NamedMultiReservableCurrency<T::AccountId>>::reserved_balance_named(
                    id, currency, who,
                );
            }

            Self::log_unsupported(currency_id, "reserved_balance_named");
            Zero::zero()
        }

        fn reserve_named(
            id: &Self::ReserveIdentifier,
            currency_id: Self::CurrencyId,
            who: &T::AccountId,
            value: Self::Balance,
        ) -> DispatchResult {
            if let Ok(currency) = T::CurrencyType::try_from(currency_id) {
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
            if let Ok(currency) = T::CurrencyType::try_from(currency_id) {
                return <T::Currencies as NamedMultiReservableCurrency<T::AccountId>>::unreserve_named(
                    id, currency, who, value
                );
            }

            Self::log_unsupported(currency_id, "unreserve_named");
            value
        }

        fn slash_reserved_named(
            id: &Self::ReserveIdentifier,
            currency_id: Self::CurrencyId,
            who: &T::AccountId,
            value: Self::Balance,
        ) -> Self::Balance {
            if let Ok(currency) = T::CurrencyType::try_from(currency_id) {
                return <T::Currencies as NamedMultiReservableCurrency<T::AccountId>>::slash_reserved_named(
                    id, currency, who, value
                );
            }

            Self::log_unsupported(currency_id, "slash_reserved_named");
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
            if let Ok(currency) = T::CurrencyType::try_from(currency_id) {
                return <T::Currencies as NamedMultiReservableCurrency<T::AccountId>>::repatriate_reserved_named(
                    id, currency, slashed, beneficiary, value, status
                );
            }

            Err(Error::<T>::Unsupported.into())
        }
    }

    // Supertrait of Create and Destroy
    impl<T: Config> Inspect<T::AccountId> for Pallet<T> {
        type AssetId = T::AssetType;
        type Balance = T::Balance;

        fn total_issuance(asset: Self::AssetId) -> Self::Balance {
            route_call!(asset, total_issuance, total_issuance,).unwrap_or(Zero::zero())
        }

        fn minimum_balance(asset: Self::AssetId) -> Self::Balance {
            route_call!(asset, minimum_balance, minimum_balance,).unwrap_or(Zero::zero())
        }

        fn balance(asset: Self::AssetId, who: &T::AccountId) -> Self::Balance {
            route_call!(asset, total_balance, balance, who).unwrap_or(Zero::zero())
        }

        fn reducible_balance(
            asset: Self::AssetId,
            who: &T::AccountId,
            keep_alive: bool,
        ) -> Self::Balance {
            if T::CurrencyType::try_from(asset).is_ok() {
                <Self as MultiCurrency<T::AccountId>>::free_balance(asset, who)
            } else {
                only_asset!(asset, Zero::zero(), Inspect, reducible_balance, who, keep_alive)
            }
        }

        fn can_deposit(
            asset: Self::AssetId,
            who: &T::AccountId,
            amount: Self::Balance,
            mint: bool,
        ) -> DepositConsequence {
            if T::CurrencyType::try_from(asset).is_err() {
                return only_asset!(
                    asset,
                    DepositConsequence::UnknownAsset,
                    Inspect,
                    can_deposit,
                    who,
                    amount,
                    mint
                );
            }

            let total_balance = <Self as MultiCurrency<T::AccountId>>::total_balance(asset, who);
            let min_balance = <Self as MultiCurrency<T::AccountId>>::minimum_balance(asset);

            if total_balance.saturating_add(amount) < min_balance {
                DepositConsequence::BelowMinimum
            } else {
                DepositConsequence::Success
            }
        }

        fn can_withdraw(
            asset: Self::AssetId,
            who: &T::AccountId,
            amount: Self::Balance,
        ) -> WithdrawConsequence<Self::Balance> {
            if T::CurrencyType::try_from(asset).is_err() {
                return only_asset!(
                    asset,
                    WithdrawConsequence::UnknownAsset,
                    Inspect,
                    can_withdraw,
                    who,
                    amount
                );
            }

            let can_withdraw =
                <Self as MultiCurrency<T::AccountId>>::ensure_can_withdraw(asset, who, amount);

            if let Err(_e) = can_withdraw {
                return WithdrawConsequence::NoFunds;
            }

            let total_balance = <Self as MultiCurrency<T::AccountId>>::total_balance(asset, who);
            let min_balance = <Self as MultiCurrency<T::AccountId>>::minimum_balance(asset);
            let remainder = total_balance.saturating_sub(amount);

            if remainder < min_balance {
                WithdrawConsequence::ReducedToZero(remainder)
            } else {
                WithdrawConsequence::Success
            }
        }

        fn asset_exists(asset: Self::AssetId) -> bool {
            if T::CurrencyType::try_from(asset).is_ok() {
                true
            } else {
                only_asset!(asset, false, Inspect, asset_exists,)
            }
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

    impl<T: Config> ManagedDestroy<T::AccountId> for Pallet<T> {
        fn managed_destroy(
            asset: Self::AssetId,
            maybe_check_owner: Option<T::AccountId>,
        ) -> DispatchResult {
            Self::asset_exists(asset).then_some(()).ok_or(Error::<T>::UnknownAsset)?;
            let mut destroy_assets = DestroyAssets::<T>::get();
            frame_support::ensure!(!destroy_assets.is_full(), Error::<T>::TooManyManagedDestroys);
            let asset_to_insert = AssetInDestruction::new(asset);

            let idx = match destroy_assets.binary_search(&asset_to_insert) {
                Ok(_) => return Err(Error::<T>::DestructionInProgress.into()),
                Err(idx) => {
                    if IndestructibleAssets::<T>::get().binary_search(&asset).is_ok() {
                        return Err(Error::<T>::DestructionInProgress.into());
                    }

                    idx
                }
            };

            destroy_assets
                .try_insert(idx, asset_to_insert)
                .map_err(|_| Error::<T>::TooManyManagedDestroys)?;

            Self::start_destroy(asset, maybe_check_owner)?;
            DestroyAssets::<T>::put(destroy_assets);

            Ok(())
        }

        #[frame_support::transactional]
        fn managed_destroy_multi(
            assets: BTreeMap<Self::AssetId, Option<T::AccountId>>,
        ) -> DispatchResult {
            let mut destroy_assets = DestroyAssets::<T>::get();

            for (asset, maybe_check_owner) in assets {
                Self::asset_exists(asset).then_some(()).ok_or(Error::<T>::UnknownAsset)?;
                frame_support::ensure!(
                    !destroy_assets.is_full(),
                    Error::<T>::TooManyManagedDestroys
                );
                let asset_to_insert = AssetInDestruction::new(asset);

                let idx = match destroy_assets.binary_search(&asset_to_insert) {
                    Ok(_) => return Err(Error::<T>::DestructionInProgress.into()),
                    Err(idx) => {
                        if IndestructibleAssets::<T>::get().binary_search(&asset).is_ok() {
                            return Err(Error::<T>::DestructionInProgress.into());
                        }

                        idx
                    }
                };

                destroy_assets
                    .try_insert(idx, asset_to_insert)
                    .map_err(|_| Error::<T>::TooManyManagedDestroys)?;

                Self::start_destroy(asset, maybe_check_owner)?;
            }

            DestroyAssets::<T>::put(destroy_assets);
            Ok(())
        }
    }
}
