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

#![feature(proc_macro_hygiene)]
#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub use pallet::*;

#[macro_use]
mod macros;
#[cfg(test)]
mod mock;
pub mod pallet_impl;
#[cfg(test)]
mod tests;
mod types;

#[frame_support::pallet]
pub mod pallet {
    pub(crate) use super::types::*;
    pub(crate) use alloc::collections::BTreeMap;
    pub(crate) use core::{fmt::Debug, marker::PhantomData};
    pub(crate) use frame_support::{
        ensure, log,
        pallet_prelude::{DispatchError, DispatchResult, Hooks, StorageValue, ValueQuery, Weight},
        require_transactional,
        traits::{
            tokens::{
                fungibles::{Create, Destroy, Inspect, Mutate, Transfer, Unbalanced},
                DepositConsequence, WithdrawConsequence,
            },
            BalanceStatus as Status, ConstU32,
        },
        BoundedVec, Parameter,
    };
    pub(crate) use orml_traits::{
        arithmetic::Signed,
        currency::{
            MultiCurrency, MultiCurrencyExtended, MultiLockableCurrency, MultiReservableCurrency,
            NamedMultiReservableCurrency, TransferAll,
        },
        BalanceStatus, LockIdentifier,
    };
    pub(crate) use pallet_assets::ManagedDestroy;
    use parity_scale_codec::{FullCodec, MaxEncodedLen};
    use scale_info::TypeInfo;
    pub(crate) use sp_runtime::{
        traits::{
            AtLeast32BitUnsigned, Bounded, Get, MaybeSerializeDeserialize, Member, Saturating, Zero,
        },
        FixedPointOperand, SaturatedConversion,
    };
    use zeitgeist_macros::unreachable_non_terminating;
    pub(crate) use zeitgeist_primitives::traits::CheckedDivPerComponent;

    pub(crate) const LOG_TARGET: &str = "runtime::asset-router";
    pub(crate) const MAX_ASSET_DESTRUCTIONS_PER_BLOCK: u8 = 128;
    pub(crate) const MAX_ASSETS_IN_DESTRUCTION: u32 = 2048;
    const MAX_INDESTRUCTIBLE_ASSETS: u32 = 256;
    // 1 ms minimum computation time.
    pub(crate) const MIN_ON_IDLE_EXTRA_COMPUTATION_WEIGHT: u64 = 1_000_000_000;

    pub trait AssetTraits<T: Config, A>:
        Create<T::AccountId, AssetId = A, Balance = T::Balance>
        + Destroy<T::AccountId, AssetId = A, Balance = T::Balance>
        + Inspect<T::AccountId, AssetId = A, Balance = T::Balance>
        + Transfer<T::AccountId, AssetId = A, Balance = T::Balance>
        + Mutate<T::AccountId, AssetId = A, Balance = T::Balance>
        + Unbalanced<T::AccountId, AssetId = A, Balance = T::Balance>
    {
    }

    impl<G, T, A> AssetTraits<T, A> for G
    where
        G: Create<T::AccountId, AssetId = A, Balance = T::Balance>
            + Destroy<T::AccountId, AssetId = A, Balance = T::Balance>
            + Inspect<T::AccountId, AssetId = A, Balance = T::Balance>
            + Transfer<T::AccountId, AssetId = A, Balance = T::Balance>
            + Mutate<T::AccountId, AssetId = A, Balance = T::Balance>
            + Unbalanced<T::AccountId, AssetId = A, Balance = T::Balance>,
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
            + NamedMultiReservableCurrency<Self::AccountId>
            + Unbalanced<Self::AccountId, AssetId = Self::CurrencyType, Balance = Self::Balance>;
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

    /// Keeps track of assets that have to be destroyed.
    #[pallet::storage]
    pub(super) type DestroyAssets<T: Config> = StorageValue<_, DestroyAssetsT<T>, ValueQuery>;

    /// Keeps track of assets that can't be destroyed.
    #[pallet::storage]
    pub(crate) type IndestructibleAssets<T: Config> =
        StorageValue<_, BoundedVec<T::AssetType, ConstU32<MAX_ASSETS_IN_DESTRUCTION>>, ValueQuery>;

    #[pallet::error]
    pub enum Error<T> {
        /// Cannot convert Amount (MultiCurrencyExtended implementation) into Balance type.
        AmountIntoBalanceFailed,
        /// Cannot start managed destruction as the asset was marked as indestructible.
        AssetIndestructible,
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

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
        fn on_idle(_: T::BlockNumber, mut remaining_weight: Weight) -> Weight {
            let max_extra_weight = Self::on_idle_max_extra_weight();

            if !remaining_weight
                .all_gte(max_extra_weight.saturating_add(T::DbWeight::get().reads(1)))
            {
                return remaining_weight;
            };

            let mut destroy_assets = DestroyAssets::<T>::get();
            if destroy_assets.len() == 0 {
                return remaining_weight.saturating_sub(T::DbWeight::get().reads(1));
            }

            remaining_weight = remaining_weight
                .saturating_sub(T::DbWeight::get().reads_writes(1, 1))
                .saturating_sub(max_extra_weight);

            remaining_weight =
                Self::handle_asset_destruction(remaining_weight, &mut destroy_assets);

            DestroyAssets::<T>::put(destroy_assets);
            remaining_weight
        }
    }

    impl<T: Config> Pallet<T> {
        fn handle_asset_destruction(
            mut remaining_weight: Weight,
            destroy_assets: &mut DestroyAssetsT<T>,
        ) -> Weight {
            let mut saftey_counter_outer = 0u8;

            'outer: while let Some(mut asset) = destroy_assets.pop() {
                // Only reachable if there is an error in the implementation of pop() for Vec.
                unreachable_non_terminating!(
                    saftey_counter_outer != MAX_ASSET_DESTRUCTIONS_PER_BLOCK,
                    LOG_TARGET,
                    break,
                    "Destruction outer loop iteration guard triggered, iteration: {:?}",
                    saftey_counter_outer
                );

                let safety_counter_inner_max = DESTRUCTION_STATES;
                let mut safety_counter_inner = 0u8;

                while *asset.state() != DestructionState::Destroyed
                    && *asset.state() != DestructionState::Indestructible
                    && safety_counter_inner < safety_counter_inner_max
                {
                    match asset.state() {
                        DestructionState::Accounts => {
                            handle_asset_destruction!(
                                &mut asset,
                                remaining_weight,
                                destroy_assets,
                                handle_destroy_accounts,
                                'outer
                            );
                        }
                        DestructionState::Approvals => {
                            handle_asset_destruction!(
                                &mut asset,
                                remaining_weight,
                                destroy_assets,
                                handle_destroy_approvals,
                                'outer
                            );
                        }
                        DestructionState::Finalization => {
                            handle_asset_destruction!(
                                &mut asset,
                                remaining_weight,
                                destroy_assets,
                                handle_destroy_finish,
                                'outer
                            );
                        }
                        // Next two states should never occur. Just remove the asset.
                        DestructionState::Destroyed => {
                            unreachable_non_terminating!(
                                false,
                                LOG_TARGET,
                                "Asset {:?} has invalid state",
                                asset
                            );
                        }
                        DestructionState::Indestructible => {
                            unreachable_non_terminating!(
                                false,
                                LOG_TARGET,
                                "Asset {:?} has invalid state",
                                asset
                            );
                        }
                    }

                    safety_counter_inner = safety_counter_inner.saturating_add(1);
                }

                // Only reachable if there is an error in the destruction state machine.
                unreachable_non_terminating!(
                    safety_counter_inner != safety_counter_inner_max,
                    LOG_TARGET,
                    "Destruction inner loop iteration guard triggered, asset: {:?}",
                    asset
                );

                saftey_counter_outer = saftey_counter_outer.saturating_add(1);
            }

            remaining_weight
        }

        fn handle_destroy_accounts(
            asset: &mut AssetInDestruction<T::AssetType>,
            mut remaining_weight: Weight,
        ) -> DestructionResult {
            if *asset.state() != DestructionState::Accounts {
                return Err(DestructionError::WrongState(remaining_weight));
            }
            let destroy_account_weight = T::DestroyAccountWeight::get();

            let destroy_account_cap =
                match remaining_weight.checked_div_per_component(&destroy_account_weight) {
                    Some(amount) => amount,
                    None => return Ok(DestructionOk::Incomplete(remaining_weight)),
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
                        Ok(DestructionOk::Complete(remaining_weight))
                    } else {
                        Ok(DestructionOk::Incomplete(remaining_weight))
                    }
                }
                Err(e) => {
                    // In this case, it is not known how many accounts have been destroyed prior
                    // to triggering this error. The only safe handling is consuming all the
                    // remaining weight.
                    let _ = Self::mark_asset_as_indestructible(
                        asset,
                        remaining_weight,
                        remaining_weight,
                        e,
                    );
                    // Play it safe, consume all remaining weight.
                    Err(DestructionError::Indestructible(Weight::zero()))
                }
            }
        }

        fn handle_destroy_approvals(
            asset: &mut AssetInDestruction<T::AssetType>,
            mut remaining_weight: Weight,
        ) -> DestructionResult {
            if *asset.state() != DestructionState::Approvals {
                return Err(DestructionError::WrongState(remaining_weight));
            }
            let destroy_approval_weight = T::DestroyApprovalWeight::get();

            let destroy_approval_cap =
                match remaining_weight.checked_div_per_component(&destroy_approval_weight) {
                    Some(amount) => amount,
                    None => return Ok(DestructionOk::Incomplete(remaining_weight)),
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
                        Ok(DestructionOk::Complete(remaining_weight))
                    } else {
                        Ok(DestructionOk::Incomplete(remaining_weight))
                    }
                }
                Err(e) => {
                    // In this case, it is not known how many approvals have been destroyed prior
                    // to triggering this error. The only safe handling is consuming all the
                    // remaining weight.
                    let _ = Self::mark_asset_as_indestructible(
                        asset,
                        remaining_weight,
                        remaining_weight,
                        e,
                    );
                    // Play it safe, consume all remaining weight.
                    Err(DestructionError::Indestructible(Weight::zero()))
                }
            }
        }

        fn handle_destroy_finish(
            asset: &mut AssetInDestruction<T::AssetType>,
            remaining_weight: Weight,
        ) -> DestructionResult {
            if *asset.state() != DestructionState::Finalization {
                return Err(DestructionError::WrongState(remaining_weight));
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
                    return Err(DestructionError::Indestructible(remaining_weight_err));
                }

                asset.transit_state();
                return Ok(DestructionOk::Complete(
                    remaining_weight.saturating_sub(destroy_finish_weight),
                ));
            }

            Ok(DestructionOk::Incomplete(remaining_weight))
        }

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

        fn on_idle_max_extra_weight() -> Weight {
            let max_proof_size_destructibles: u64 =
                AssetInDestruction::<T::AssetType>::max_encoded_len()
                    .saturating_mul(MAX_ASSETS_IN_DESTRUCTION.saturated_into())
                    .saturated_into();
            let max_proof_size_indestructibles: u64 = T::AssetType::max_encoded_len()
                .saturating_mul(MAX_INDESTRUCTIBLE_ASSETS.saturated_into())
                .saturated_into();
            let max_proof_size_total =
                max_proof_size_destructibles.saturating_add(max_proof_size_indestructibles);

            Weight::from_parts(
                MIN_ON_IDLE_EXTRA_COMPUTATION_WEIGHT,
                // Maximum proof size assuming writes on full storage.
                max_proof_size_total,
            )
        }

        #[inline]
        pub(crate) fn log_unsupported(asset: T::AssetType, function: &str) {
            log::warn!(target: LOG_TARGET, "Asset {:?} not supported in function {:?}", asset, function);
        }
    }
}
