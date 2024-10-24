// Copyright 2024 Forecasting Technologies LTD.
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
//
// This file incorporates work licensed under the GNU Lesser General
// Public License 3.0 but published without copyright notice by Gnosis
// (<https://gnosis.io>, info@gnosis.io) in the
// conditional-tokens-contracts repository
// <https://github.com/gnosis/conditional-tokens-contracts>,
// and has been relicensed under GPL-3.0-or-later in this repository.

#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod mock;
mod tests;
mod traits;
pub mod types;

pub use pallet::*;

#[frame_support::pallet]
mod pallet {
    use crate::traits::CombinatorialIdManager;
    use alloc::{vec, vec::Vec};
    use core::marker::PhantomData;
    use frame_support::{
        ensure,
        pallet_prelude::{IsType, StorageVersion},
        require_transactional, transactional, PalletId,
    };
    use frame_system::{
        ensure_signed,
        pallet_prelude::{BlockNumberFor, OriginFor},
    };
    use orml_traits::MultiCurrency;
    use sp_runtime::{
        traits::{AccountIdConversion, Get, Zero},
        DispatchError, DispatchResult,
    };
    use zeitgeist_primitives::{
        math::{checked_ops_res::CheckedAddRes, fixed::FixedMul},
        traits::{MarketCommonsPalletApi, PayoutApi},
        types::{Asset, CombinatorialId},
    };

    #[cfg(feature = "runtime-benchmarks")]
    use zeitgeist_primitives::traits::CombinatorialTokensBenchmarkHelper;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        #[cfg(feature = "runtime-benchmarks")]
        type BenchmarkHelper: CombinatorialTokensBenchmarkHelper<Balance = BalanceOf<Self>>;

        type CombinatorialIdManager: CombinatorialIdManager<
                Asset = AssetOf<Self>,
                MarketId = MarketIdOf<Self>,
                CombinatorialId = CombinatorialId,
            >;

        type MarketCommons: MarketCommonsPalletApi<AccountId = Self::AccountId, BlockNumber = BlockNumberFor<Self>>;

        type MultiCurrency: MultiCurrency<Self::AccountId, CurrencyId = AssetOf<Self>>;

        type Payout: PayoutApi<Balance = BalanceOf<Self>, MarketId = MarketIdOf<Self>>;

        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        #[pallet::constant]
        type PalletId: Get<PalletId>;
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(PhantomData<T>);

    pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
    pub(crate) type AssetOf<T> = Asset<MarketIdOf<T>>;
    pub(crate) type BalanceOf<T> =
        <<T as Config>::MultiCurrency as MultiCurrency<AccountIdOf<T>>>::Balance;
    pub(crate) type CombinatorialIdOf<T> =
        <<T as Config>::CombinatorialIdManager as CombinatorialIdManager>::CombinatorialId;
    pub(crate) type MarketIdOf<T> =
        <<T as Config>::MarketCommons as MarketCommonsPalletApi>::MarketId;

    pub(crate) const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T>
    where
        T: Config,
    {
        /// User `who` has split `amount` units of token `asset_in` into the same amount of each
        /// token in `assets_out` using `partition`. The ith element of `partition` matches the ith
        /// element of `assets_out`, so `assets_out[i]` is the outcome represented by the specified
        /// `parent_collection_id` when split using `partition[i]` in `market_id`. The same goes for
        /// the `collection_ids` vector, the ith element of which specifies the collection ID of
        /// `assets_out[i]`.
        TokenSplit {
            who: AccountIdOf<T>,
            parent_collection_id: Option<CombinatorialId>,
            market_id: MarketIdOf<T>,
            partition: Vec<Vec<bool>>,
            asset_in: AssetOf<T>,
            assets_out: Vec<AssetOf<T>>,
            collection_ids: Vec<CombinatorialId>,
            amount: BalanceOf<T>,
        },

        /// User `who` has merged `amount` units of each of the tokens in `assets_in` into the same
        /// amount of `asset_out`. The ith element of the `partition` matches the ith element of
        /// `assets_in`, so `assets_in[i]` is the outcome represented by the specified
        /// `parent_collection_id` when split using `partition[i]` in `market_id`. Note that the
        /// `parent_collection_id` is equal to the collection ID of the position `asset_out`; if
        /// `asset_out` is the collateral token, then `parent_collection_id` is `None`.
        TokenMerged {
            who: AccountIdOf<T>,
            parent_collection_id: Option<CombinatorialId>,
            market_id: MarketIdOf<T>,
            partition: Vec<Vec<bool>>,
            asset_out: AssetOf<T>,
            assets_in: Vec<AssetOf<T>>,
            amount: BalanceOf<T>,
        },

        /// User `who` has redeemed `amount_in` units of `asset_in` for `amount_out` units of
        /// `asset_out` using the report for the market specified by `market_id`. The
        /// `parent_collection_id` specifies the collection ID of the `asset_out`; it is `None` if
        /// the `asset_out` is the collateral token.
        TokenRedeemed {
            who: AccountIdOf<T>,
            parent_collection_id: Option<CombinatorialId>,
            market_id: MarketIdOf<T>,
            index_set: Vec<bool>,
            asset_in: AssetOf<T>,
            amount_in: BalanceOf<T>,
            asset_out: AssetOf<T>,
            amount_out: BalanceOf<T>,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Specified index set is trival, empty, or doesn't match the market's number of outcomes.
        InvalidIndexSet,

        /// Specified partition is empty, contains overlaps, is too long or doesn't match the
        /// market's number of outcomes.
        InvalidPartition,

        /// Specified collection ID is invalid.
        InvalidCollectionId,

        /// Specified market is not resolved.
        PayoutVectorNotFound,

        /// Account holds no tokens of this type.
        NoTokensFound,

        /// Specified token holds no redeemable value.
        TokenHasNoValue,

        /// Something unexpected happened. You shouldn't see this.
        UnexpectedError,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight({0})] // TODO
        #[transactional]
        pub fn split_position(
            origin: OriginFor<T>,
            // TODO Abstract this into a separate type.
            parent_collection_id: Option<CombinatorialIdOf<T>>,
            market_id: MarketIdOf<T>,
            partition: Vec<Vec<bool>>,
            amount: BalanceOf<T>,
            force_max_work: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_split_position(
                who,
                parent_collection_id,
                market_id,
                partition,
                amount,
                force_max_work,
            )
        }

        #[pallet::call_index(1)]
        #[pallet::weight({0})] // TODO
        #[transactional]
        pub fn merge_position(
            origin: OriginFor<T>,
            parent_collection_id: Option<CombinatorialIdOf<T>>,
            market_id: MarketIdOf<T>,
            partition: Vec<Vec<bool>>,
            amount: BalanceOf<T>,
            force_max_work: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_merge_position(
                who,
                parent_collection_id,
                market_id,
                partition,
                amount,
                force_max_work,
            )
        }

        #[pallet::call_index(2)]
        #[pallet::weight({0})] // TODO
        #[transactional]
        pub fn redeem_position(
            origin: OriginFor<T>,
            parent_collection_id: Option<CombinatorialIdOf<T>>,
            market_id: MarketIdOf<T>,
            index_set: Vec<bool>,
            force_max_work: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_redeem_position(
                who,
                parent_collection_id,
                market_id,
                index_set,
                force_max_work,
            )
        }
    }

    impl<T: Config> Pallet<T> {
        #[require_transactional]
        fn do_split_position(
            who: AccountIdOf<T>,
            parent_collection_id: Option<CombinatorialIdOf<T>>,
            market_id: MarketIdOf<T>,
            partition: Vec<Vec<bool>>,
            amount: BalanceOf<T>,
            force_max_work: bool,
        ) -> DispatchResult {
            let market = T::MarketCommons::market(&market_id)?;
            let collateral_token = market.base_asset;

            let free_index_set = Self::free_index_set(market_id, &partition)?;

            // Destroy/store the tokens to be split.
            let split_asset = if !free_index_set.iter().any(|&i| i) {
                // Vertical split.
                if let Some(pci) = parent_collection_id {
                    // Split combinatorial token into higher level position. Destroy the tokens.
                    let position_id =
                        T::CombinatorialIdManager::get_position_id(collateral_token, pci);
                    let position = Asset::CombinatorialToken(position_id);

                    // This will fail if the market has a different collateral than the previous
                    // markets. FIXME A cleaner error message would be nice though...
                    T::MultiCurrency::ensure_can_withdraw(position, &who, amount)?;
                    T::MultiCurrency::withdraw(position, &who, amount)?;

                    position
                } else {
                    // Split collateral into first level position. Store the collateral in the
                    // pallet account. This is the legacy `buy_complete_set`.
                    T::MultiCurrency::ensure_can_withdraw(collateral_token, &who, amount)?;
                    T::MultiCurrency::transfer(
                        collateral_token,
                        &who,
                        &Self::account_id(),
                        amount,
                    )?;

                    collateral_token
                }
            } else {
                // Horizontal split.
                let remaining_index_set = free_index_set.into_iter().map(|i| !i).collect();
                let position = Self::position_from_parent_collection(
                    parent_collection_id,
                    market_id,
                    remaining_index_set,
                    force_max_work,
                )?;
                T::MultiCurrency::ensure_can_withdraw(position, &who, amount)?;
                T::MultiCurrency::withdraw(position, &who, amount)?;

                position
            };

            // Deposit the new tokens.
            let collection_ids = partition
                .iter()
                .cloned()
                .map(|index_set| {
                    Self::collection_id_from_parent_collection(
                        parent_collection_id,
                        market_id,
                        index_set,
                        force_max_work,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            let positions = collection_ids
                .iter()
                .cloned()
                .map(|collection_id| Self::position_from_collection_id(market_id, collection_id))
                .collect::<Result<Vec<_>, _>>()?;
            // Security note: Safe as iterations are limited to the number of assets in the market
            // thanks to the `ensure!` invocations in `Self::free_index_set`.
            for &position in positions.iter() {
                T::MultiCurrency::deposit(position, &who, amount)?;
            }

            Self::deposit_event(Event::<T>::TokenSplit {
                who,
                parent_collection_id,
                market_id,
                partition,
                asset_in: split_asset,
                assets_out: positions,
                collection_ids,
                amount,
            });

            Ok(())
        }

        #[require_transactional]
        fn do_merge_position(
            who: AccountIdOf<T>,
            parent_collection_id: Option<CombinatorialIdOf<T>>,
            market_id: MarketIdOf<T>,
            partition: Vec<Vec<bool>>,
            amount: BalanceOf<T>,
            force_max_work: bool,
        ) -> DispatchResult {
            let market = T::MarketCommons::market(&market_id)?;
            let collateral_token = market.base_asset;

            let free_index_set = Self::free_index_set(market_id, &partition)?;

            // Destroy the old tokens.
            let positions = partition
                .iter()
                .cloned()
                .map(|index_set| {
                    Self::position_from_parent_collection(
                        parent_collection_id,
                        market_id,
                        index_set,
                        force_max_work,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            // Security note: Safe as iterations are limited to the number of assets in the market
            // thanks to the `ensure!` invocations in `Self::free_index_set`.
            for &position in positions.iter() {
                T::MultiCurrency::withdraw(position, &who, amount)?;
            }

            // Destroy/store the tokens to be split.
            let merged_token = if !free_index_set.iter().any(|&i| i) {
                // Vertical merge.
                if let Some(pci) = parent_collection_id {
                    // Merge combinatorial token into higher level position. Destroy the tokens.
                    let position_id =
                        T::CombinatorialIdManager::get_position_id(collateral_token, pci);
                    let position = Asset::CombinatorialToken(position_id);
                    T::MultiCurrency::deposit(position, &who, amount)?;

                    position
                } else {
                    // Merge first-level tokens into collateral. Move collateral from the pallet
                    // account to the user's wallet. This is the legacy `sell_complete_set`.
                    T::MultiCurrency::transfer(
                        collateral_token,
                        &Self::account_id(),
                        &who,
                        amount,
                    )?;

                    collateral_token
                }
            } else {
                // Horizontal merge.
                let remaining_index_set = free_index_set.into_iter().map(|i| !i).collect();
                let position = Self::position_from_parent_collection(
                    parent_collection_id,
                    market_id,
                    remaining_index_set,
                    force_max_work,
                )?;
                T::MultiCurrency::deposit(position, &who, amount)?;

                position
            };

            Self::deposit_event(Event::<T>::TokenMerged {
                who,
                parent_collection_id,
                market_id,
                partition,
                asset_out: merged_token,
                assets_in: positions,
                amount,
            });

            Ok(())
        }

        fn do_redeem_position(
            who: T::AccountId,
            parent_collection_id: Option<CombinatorialIdOf<T>>,
            market_id: MarketIdOf<T>,
            index_set: Vec<bool>,
            force_max_work: bool,
        ) -> DispatchResult {
            let payout_vector =
                T::Payout::payout_vector(market_id).ok_or(Error::<T>::PayoutVectorNotFound)?;

            let market = T::MarketCommons::market(&market_id)?;
            let asset_count = market.outcomes() as usize;
            let collateral_token = market.base_asset;

            ensure!(index_set.len() == asset_count, Error::<T>::InvalidIndexSet);
            ensure!(index_set.iter().any(|&b| b), Error::<T>::InvalidIndexSet);
            ensure!(!index_set.iter().all(|&b| b), Error::<T>::InvalidIndexSet);

            // Add up values of each outcome.
            let mut total_stake: BalanceOf<T> = Zero::zero();
            // Security note: Safe because `zip` will limit this loop to `payout_vector.len()`
            // iterations.
            for (&index, value) in index_set.iter().zip(payout_vector.iter()) {
                if index {
                    total_stake = total_stake.checked_add_res(value)?;
                }
            }

            ensure!(!total_stake.is_zero(), Error::<T>::TokenHasNoValue);

            let position = Self::position_from_parent_collection(
                parent_collection_id,
                market_id,
                index_set.clone(),
                force_max_work,
            )?;
            let amount = T::MultiCurrency::free_balance(position, &who);
            ensure!(!amount.is_zero(), Error::<T>::NoTokensFound);
            T::MultiCurrency::withdraw(position, &who, amount)?;

            let total_payout = total_stake.bmul(amount)?;

            let asset_out = if let Some(pci) = parent_collection_id {
                // Merge combinatorial token into higher level position. Destroy the tokens.
                let position_id = T::CombinatorialIdManager::get_position_id(collateral_token, pci);
                let position = Asset::CombinatorialToken(position_id);
                T::MultiCurrency::deposit(position, &who, total_payout)?;

                position
            } else {
                T::MultiCurrency::transfer(
                    collateral_token,
                    &Self::account_id(),
                    &who,
                    total_payout,
                )?;

                collateral_token
            };

            Self::deposit_event(Event::<T>::TokenRedeemed {
                who,
                parent_collection_id,
                market_id,
                index_set,
                asset_in: position,
                amount_in: amount,
                asset_out,
                amount_out: total_payout,
            });

            Ok(())
        }

        pub(crate) fn account_id() -> T::AccountId {
            T::PalletId::get().into_account_truncating()
        }

        fn free_index_set(
            market_id: MarketIdOf<T>,
            partition: &[Vec<bool>],
        ) -> Result<Vec<bool>, DispatchError> {
            let market = T::MarketCommons::market(&market_id)?;
            let asset_count = market.outcomes() as usize;
            let mut free_index_set = vec![true; asset_count];

            for index_set in partition.iter() {
                // Ensure that the partition is not trivial and matches the market's outcomes.
                ensure!(index_set.iter().any(|&i| i), Error::<T>::InvalidPartition);
                ensure!(index_set.len() == asset_count, Error::<T>::InvalidPartition);
                ensure!(!index_set.iter().all(|&i| i), Error::<T>::InvalidPartition);

                // Ensure that `index_set` is disjoint from the previously iterated elements of the
                // partition.
                ensure!(
                    free_index_set.iter().zip(index_set.iter()).all(|(i, j)| *i || !*j),
                    Error::<T>::InvalidPartition
                );

                // Remove indices of `index_set` from `free_index_set`.
                free_index_set =
                    free_index_set.iter().zip(index_set.iter()).map(|(i, j)| *i && !*j).collect();
            }

            Ok(free_index_set)
        }

        fn collection_id_from_parent_collection(
            parent_collection_id: Option<CombinatorialIdOf<T>>,
            market_id: MarketIdOf<T>,
            index_set: Vec<bool>,
            force_max_work: bool,
        ) -> Result<CombinatorialIdOf<T>, DispatchError> {
            T::CombinatorialIdManager::get_collection_id(
                parent_collection_id,
                market_id,
                index_set,
                force_max_work,
            )
            .ok_or(Error::<T>::InvalidCollectionId.into())
        }

        fn position_from_collection_id(
            market_id: MarketIdOf<T>,
            collection_id: CombinatorialIdOf<T>,
        ) -> Result<AssetOf<T>, DispatchError> {
            let market = T::MarketCommons::market(&market_id)?;
            let collateral_token = market.base_asset;

            let position_id =
                T::CombinatorialIdManager::get_position_id(collateral_token, collection_id);
            let asset = Asset::CombinatorialToken(position_id);

            Ok(asset)
        }

        fn position_from_parent_collection(
            parent_collection_id: Option<CombinatorialIdOf<T>>,
            market_id: MarketIdOf<T>,
            index_set: Vec<bool>,
            force_max_work: bool,
        ) -> Result<AssetOf<T>, DispatchError> {
            let collection_id = Self::collection_id_from_parent_collection(
                parent_collection_id,
                market_id,
                index_set,
                force_max_work,
            )?;

            Self::position_from_collection_id(market_id, collection_id)
        }
    }
}
