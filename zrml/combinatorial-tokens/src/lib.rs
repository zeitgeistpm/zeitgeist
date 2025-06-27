// Copyright 2025 Forecasting Technologies LTD.
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

mod benchmarking;
pub mod mock;
mod tests;
pub mod traits;
pub mod types;
pub mod weights;

pub use pallet::*;

#[frame_support::pallet]
mod pallet {
    use crate::{
        traits::CombinatorialIdManager,
        types::{CollectionIdError, TransmutationType},
        weights::WeightInfoZeitgeist,
    };
    use alloc::{vec, vec::Vec};
    use core::{fmt::Debug, marker::PhantomData};
    use frame_support::{
        dispatch::DispatchResultWithPostInfo,
        ensure,
        pallet_prelude::{IsType, StorageVersion},
        require_transactional, transactional, PalletId,
    };
    use frame_system::{
        ensure_signed,
        pallet_prelude::{BlockNumberFor, OriginFor},
    };
    use orml_traits::MultiCurrency;
    use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
    use scale_info::TypeInfo;
    use sp_runtime::{
        traits::{AccountIdConversion, Get, Zero},
        DispatchError, DispatchResult, SaturatedConversion,
    };
    use zeitgeist_primitives::{
        math::{checked_ops_res::CheckedAddRes, fixed::FixedMul},
        traits::{
            CombinatorialTokensApi, CombinatorialTokensFuel, CombinatorialTokensUnsafeApi,
            MarketCommonsPalletApi, PayoutApi,
        },
        types::{Asset, CombinatorialId, SplitPositionDispatchInfo},
    };

    #[cfg(feature = "runtime-benchmarks")]
    use zeitgeist_primitives::traits::CombinatorialTokensBenchmarkHelper;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        #[cfg(feature = "runtime-benchmarks")]
        type BenchmarkHelper: CombinatorialTokensBenchmarkHelper<
            Balance = BalanceOf<Self>,
            MarketId = MarketIdOf<Self>,
        >;

        /// Interface for calculating collection and position IDs.
        type CombinatorialIdManager: CombinatorialIdManager<
            Asset = AssetOf<Self>,
            MarketId = MarketIdOf<Self>,
            CombinatorialId = CombinatorialId,
            Fuel = Self::Fuel,
        >;

        type Fuel: Clone
            + CombinatorialTokensFuel
            + Debug
            + Decode
            + Encode
            + Eq
            + MaxEncodedLen
            + PartialEq
            + TypeInfo;

        type MarketCommons: MarketCommonsPalletApi<
            AccountId = Self::AccountId,
            BlockNumber = BlockNumberFor<Self>,
        >;

        type MultiCurrency: MultiCurrency<Self::AccountId, CurrencyId = AssetOf<Self>>;

        /// Interface for acquiring the payout vector by market ID.
        type Payout: PayoutApi<Balance = BalanceOf<Self>, MarketId = MarketIdOf<Self>>;

        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        #[pallet::constant]
        type PalletId: Get<PalletId>;

        type WeightInfo: WeightInfoZeitgeist;
    }

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(PhantomData<T>);

    pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
    pub type AssetOf<T> = Asset<MarketIdOf<T>>;
    pub type BalanceOf<T> =
        <<T as Config>::MultiCurrency as MultiCurrency<AccountIdOf<T>>>::Balance;
    pub type CombinatorialIdOf<T> =
        <<T as Config>::CombinatorialIdManager as CombinatorialIdManager>::CombinatorialId;
    pub type MarketIdOf<T> = <<T as Config>::MarketCommons as MarketCommonsPalletApi>::MarketId;
    pub type FuelOf<T> = <<T as Config>::CombinatorialIdManager as CombinatorialIdManager>::Fuel;
    pub(crate) type SplitPositionDispatchInfoOf<T> =
        SplitPositionDispatchInfo<CombinatorialIdOf<T>, MarketIdOf<T>>;

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
        /// An error for the collection ID retrieval occured.
        CollectionIdRetrievalFailed(CollectionIdError),

        /// Specified index set is trival, empty, or doesn't match the market's number of outcomes.
        InvalidIndexSet,

        /// Specified partition is empty, contains overlaps, is too long or doesn't match the
        /// market's number of outcomes.
        InvalidPartition,

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
        /// Split `amount` units of the position specified by `parent_collection_id` over the market
        /// with ID `market_id` according to the given `partition`.
        ///
        /// The `partition` is specified as a vector whose elements are equal-length `Vec<bool>`. A
        /// `true` entry at the `i`th index of a partition element means that the `i`th outcome
        /// token of the market is contained in this element of the partition.
        ///
        /// For each element `b` of the partition, the split mints a new outcome token which is made
        /// up of the position to be split and the conjunction `(x|...|z)` where `x, ..., z` are the
        /// items of `b`. The position to be split, in turn, is burned or transferred into the
        /// pallet account, depending on whether or not it is a true combinatorial token or
        /// collateral.
        ///
        /// If the `parent_collection_id` is `None`, then the position split is the collateral of the
        /// market given by `market_id`.
        ///
        /// If the `parent_collection_id` is `Some(pid)`, then there are two cases: vertical and
        /// horizontal split. If `partition` is complete (i.e. there is no index `i` so that `b[i]`
        /// is `false` for all `b` in `partition`), the position split is the position obtained by
        /// combining `pid` with the collateral of the market given by `market_id`. If `partition`
        /// is not complete, the position split is the position made up of the
        /// `parent_collection_id` and the conjunction `(x|...|z)` where `x, ..., z` are the items
        /// covered by `partition`.
        ///
        /// The `fuel` parameter specifies how much work the cryptographic id manager will do
        /// and can be used for benchmarking purposes.
        #[pallet::call_index(0)]
        #[pallet::weight(
            T::WeightInfo::split_position_vertical_sans_parent(
                partition.len().saturated_into(),
                fuel.total(),
            )
            .max(T::WeightInfo::split_position_vertical_with_parent(
                partition.len().saturated_into(),
                fuel.total(),
            ))
            .max(T::WeightInfo::split_position_horizontal(
                partition.len().saturated_into(),
                fuel.total(),
            ))
        )]
        #[transactional]
        pub fn split_position(
            origin: OriginFor<T>,
            parent_collection_id: Option<CombinatorialIdOf<T>>,
            market_id: MarketIdOf<T>,
            partition: Vec<Vec<bool>>,
            amount: BalanceOf<T>,
            fuel: FuelOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;

            let SplitPositionDispatchInfo { post_dispatch_info, .. } = Self::do_split_position(
                who,
                parent_collection_id,
                market_id,
                partition,
                amount,
                fuel,
            )?;

            DispatchResultWithPostInfo::Ok(post_dispatch_info)
        }

        /// Merge `amount` units of the tokens obtained by splitting `parent_collection_id` using
        /// `partition` into the position specified by `parent_collection_id` (vertical split) or
        /// the position obtained by splitting `parent_collection_id` according to `partiton` over
        /// the market with ID `market_id` (horizontal; see below for details).
        ///
        /// The `partition` is specified as a vector whose elements are equal-length `Vec<bool>`. A
        /// `true` entry at the `i`th index of a partition element means that the `i`th outcome
        /// token of the market is contained in this element of the partition.
        ///
        /// For each element `b` of the partition, the split burns the outcome tokens which are made
        /// up of the position to be split and the conjunction `(x|...|z)` where `x, ..., z` are the
        /// items of `b`. The position given by `parent_collection_id` is
        ///
        /// If the `parent_collection_id` is `None`, then the position split is the collateral of the
        /// market given by `market_id`.
        ///
        /// If the `parent_collection_id` is `Some(pid)`, then there are two cases: vertical and
        /// horizontal merge. If `partition` is complete (i.e. there is no index `i` so that `b[i]`
        /// is `false` for all `b` in `partition`), the the result of the merge is the position
        /// defined by `parent_collection_id`. If `partition` is not complete, the result of the
        /// merge is the position made up of the `parent_collection_id` and the conjunction
        /// `(x|...|z)` where `x, ..., z` are the items covered by `partition`.
        ///
        /// The `fuel` parameter specifies how much work the cryptographic id manager will do
        /// and can be used for benchmarking purposes.
        #[pallet::call_index(1)]
        #[pallet::weight(
            T::WeightInfo::merge_position_vertical_sans_parent(
                partition.len().saturated_into(),
                fuel.total(),
            )
            .max(T::WeightInfo::merge_position_vertical_with_parent(
                partition.len().saturated_into(),
                fuel.total(),
            ))
            .max(T::WeightInfo::merge_position_horizontal(
                partition.len().saturated_into(),
                fuel.total(),
            ))
        )]
        #[transactional]
        pub fn merge_position(
            origin: OriginFor<T>,
            parent_collection_id: Option<CombinatorialIdOf<T>>,
            market_id: MarketIdOf<T>,
            partition: Vec<Vec<bool>>,
            amount: BalanceOf<T>,
            fuel: FuelOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::do_merge_position(who, parent_collection_id, market_id, partition, amount, fuel)
        }

        /// (Partially) redeems a position if part of it belongs to a resolved market given by
        /// `market_id`.
        ///
        /// The position to be redeemed is the position obtained by combining the position given by
        /// `parent_collection_id` and `collateral` with the conjunction `(x|...|z)` where `x, ...
        /// z` are the outcome tokens of the market `market_id` given by `partition`.
        ///
        /// The position to be redeemed is completely removed from the origin's wallet. According to
        /// how much the conjunction `(x|...|z)` is valued, the user is paid in the position defined
        /// by `parent_collection_id` and `collateral`.
        ///
        /// The `fuel` parameter specifies how much work the cryptographic id manager will do
        /// and can be used for benchmarking purposes.
        #[pallet::call_index(2)]
        #[pallet::weight(
            T::WeightInfo::redeem_position_with_parent(
                index_set.len().saturated_into(),
                fuel.total(),
            )
            .max(T::WeightInfo::redeem_position_sans_parent(
                index_set.len().saturated_into(),
                fuel.total()
            ))
        )]
        #[transactional]
        pub fn redeem_position(
            origin: OriginFor<T>,
            parent_collection_id: Option<CombinatorialIdOf<T>>,
            market_id: MarketIdOf<T>,
            index_set: Vec<bool>,
            fuel: FuelOf<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Self::do_redeem_position(who, parent_collection_id, market_id, index_set, fuel)
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
            fuel: FuelOf<T>,
        ) -> Result<SplitPositionDispatchInfoOf<T>, DispatchError> {
            let (transmutation_type, position) = Self::transmutation_asset(
                parent_collection_id,
                market_id,
                partition.clone(),
                fuel.clone(),
            )?;

            // Destroy the token to be split.
            let weight = match transmutation_type {
                TransmutationType::VerticalWithParent => {
                    // Split combinatorial token into higher level position.
                    // This will fail if the market has a different collateral than the previous
                    // markets.
                    T::MultiCurrency::ensure_can_withdraw(position, &who, amount)?;
                    T::MultiCurrency::withdraw(position, &who, amount)?;

                    T::WeightInfo::split_position_vertical_with_parent(
                        partition.len().saturated_into(),
                        fuel.total(),
                    )
                }
                TransmutationType::VerticalSansParent => {
                    // Split collateral into first level position. Store the collateral in the
                    // pallet account. This is the legacy `buy_complete_set`.
                    T::MultiCurrency::ensure_can_withdraw(position, &who, amount)?;
                    T::MultiCurrency::transfer(position, &who, &Self::account_id(), amount)?;

                    T::WeightInfo::split_position_vertical_sans_parent(
                        partition.len().saturated_into(),
                        fuel.total(),
                    )
                }
                TransmutationType::Horizontal => {
                    // Horizontal split.
                    T::MultiCurrency::ensure_can_withdraw(position, &who, amount)?;
                    T::MultiCurrency::withdraw(position, &who, amount)?;

                    T::WeightInfo::split_position_horizontal(
                        partition.len().saturated_into(),
                        fuel.total(),
                    )
                }
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
                        fuel.clone(),
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
                asset_in: position,
                assets_out: positions.clone(),
                collection_ids: collection_ids.clone(),
                amount,
            });

            let dispatch_info = SplitPositionDispatchInfo {
                collection_ids,
                position_ids: positions,
                post_dispatch_info: Some(weight).into(),
            };

            Ok(dispatch_info)
        }

        #[require_transactional]
        fn do_merge_position(
            who: AccountIdOf<T>,
            parent_collection_id: Option<CombinatorialIdOf<T>>,
            market_id: MarketIdOf<T>,
            partition: Vec<Vec<bool>>,
            amount: BalanceOf<T>,
            fuel: FuelOf<T>,
        ) -> DispatchResultWithPostInfo {
            let (transmutation_type, position) = Self::transmutation_asset(
                parent_collection_id,
                market_id,
                partition.clone(),
                fuel.clone(),
            )?;

            // Destroy the old tokens.
            let positions = partition
                .iter()
                .cloned()
                .map(|index_set| {
                    Self::position_from_parent_collection(
                        parent_collection_id,
                        market_id,
                        index_set,
                        fuel.clone(),
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            // Security note: Safe as iterations are limited to the number of assets in the market
            // thanks to the `ensure!` invocations in `Self::free_index_set`.
            for &position in positions.iter() {
                T::MultiCurrency::withdraw(position, &who, amount)?;
            }

            let weight = match transmutation_type {
                TransmutationType::VerticalWithParent => {
                    // Merge combinatorial token into higher level position.
                    T::MultiCurrency::deposit(position, &who, amount)?;

                    T::WeightInfo::merge_position_vertical_with_parent(
                        partition.len().saturated_into(),
                        fuel.total(),
                    )
                }
                TransmutationType::VerticalSansParent => {
                    // Merge first-level tokens into collateral. Move collateral from the pallet
                    // account to the user's wallet. This is the legacy `sell_complete_set`.
                    T::MultiCurrency::transfer(position, &Self::account_id(), &who, amount)?;

                    T::WeightInfo::merge_position_vertical_sans_parent(
                        partition.len().saturated_into(),
                        fuel.total(),
                    )
                }
                TransmutationType::Horizontal => {
                    // Horizontal merge.
                    T::MultiCurrency::deposit(position, &who, amount)?;

                    T::WeightInfo::merge_position_horizontal(
                        partition.len().saturated_into(),
                        fuel.total(),
                    )
                }
            };

            Self::deposit_event(Event::<T>::TokenMerged {
                who,
                parent_collection_id,
                market_id,
                partition,
                asset_out: position,
                assets_in: positions,
                amount,
            });

            Ok(Some(weight).into())
        }

        fn do_redeem_position(
            who: T::AccountId,
            parent_collection_id: Option<CombinatorialIdOf<T>>,
            market_id: MarketIdOf<T>,
            index_set: Vec<bool>,
            fuel: FuelOf<T>,
        ) -> DispatchResultWithPostInfo {
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
                fuel.clone(),
            )?;
            let amount = T::MultiCurrency::free_balance(position, &who);
            ensure!(!amount.is_zero(), Error::<T>::NoTokensFound);
            T::MultiCurrency::withdraw(position, &who, amount)?;

            let total_payout = total_stake.bmul(amount)?;

            let (weight, asset_out) = if let Some(pci) = parent_collection_id {
                // Merge combinatorial token into higher level position. Destroy the tokens.
                let position_id = T::CombinatorialIdManager::get_position_id(collateral_token, pci);
                let position = Asset::CombinatorialToken(position_id);
                T::MultiCurrency::deposit(position, &who, total_payout)?;

                let weight = T::WeightInfo::redeem_position_with_parent(
                    index_set.len().saturated_into(),
                    fuel.total(),
                );

                (weight, position)
            } else {
                T::MultiCurrency::transfer(
                    collateral_token,
                    &Self::account_id(),
                    &who,
                    total_payout,
                )?;

                let weight = T::WeightInfo::redeem_position_sans_parent(
                    index_set.len().saturated_into(),
                    fuel.total(),
                );

                (weight, collateral_token)
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

            Ok(Some(weight).into())
        }

        pub fn account_id() -> T::AccountId {
            T::PalletId::get().into_account_truncating()
        }

        pub(crate) fn free_index_set(
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

        pub(crate) fn transmutation_asset(
            parent_collection_id: Option<CombinatorialIdOf<T>>,
            market_id: MarketIdOf<T>,
            partition: Vec<Vec<bool>>,
            fuel: FuelOf<T>,
        ) -> Result<(TransmutationType, AssetOf<T>), DispatchError> {
            let market = T::MarketCommons::market(&market_id)?;
            let collateral_token = market.base_asset;
            let free_index_set = Self::free_index_set(market_id, &partition)?;

            let result = if !free_index_set.iter().any(|&i| i) {
                // Vertical merge.
                if let Some(pci) = parent_collection_id {
                    let position_id =
                        T::CombinatorialIdManager::get_position_id(collateral_token, pci);
                    let position = Asset::CombinatorialToken(position_id);

                    (TransmutationType::VerticalWithParent, position)
                } else {
                    (TransmutationType::VerticalSansParent, collateral_token)
                }
            } else {
                let remaining_index_set = free_index_set.into_iter().map(|i| !i).collect();
                let position = Self::position_from_parent_collection(
                    parent_collection_id,
                    market_id,
                    remaining_index_set,
                    fuel,
                )?;

                (TransmutationType::Horizontal, position)
            };

            Ok(result)
        }

        pub(crate) fn collection_id_from_parent_collection(
            parent_collection_id: Option<CombinatorialIdOf<T>>,
            market_id: MarketIdOf<T>,
            index_set: Vec<bool>,
            fuel: FuelOf<T>,
        ) -> Result<CombinatorialIdOf<T>, DispatchError> {
            T::CombinatorialIdManager::get_collection_id(
                parent_collection_id,
                market_id,
                index_set,
                fuel,
            )
            .map_err(|collection_id_error| {
                Error::<T>::CollectionIdRetrievalFailed(collection_id_error).into()
            })
        }

        pub(crate) fn position_from_collection_id(
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

        pub fn position_from_parent_collection(
            parent_collection_id: Option<CombinatorialIdOf<T>>,
            market_id: MarketIdOf<T>,
            index_set: Vec<bool>,
            fuel: FuelOf<T>,
        ) -> Result<AssetOf<T>, DispatchError> {
            let collection_id = Self::collection_id_from_parent_collection(
                parent_collection_id,
                market_id,
                index_set,
                fuel,
            )?;

            Self::position_from_collection_id(market_id, collection_id)
        }
    }

    impl<T> CombinatorialTokensApi for Pallet<T>
    where
        T: Config,
    {
        type AccountId = T::AccountId;
        type Balance = BalanceOf<T>;
        type CombinatorialId = CombinatorialIdOf<T>;
        type MarketId = MarketIdOf<T>;
        type Fuel = <<T as Config>::CombinatorialIdManager as CombinatorialIdManager>::Fuel;

        fn split_position(
            who: Self::AccountId,
            parent_collection_id: Option<Self::CombinatorialId>,
            market_id: Self::MarketId,
            partition: Vec<Vec<bool>>,
            amount: Self::Balance,
            fuel: Self::Fuel,
        ) -> Result<SplitPositionDispatchInfoOf<T>, DispatchError> {
            Self::do_split_position(who, parent_collection_id, market_id, partition, amount, fuel)
        }
    }

    impl<T> CombinatorialTokensUnsafeApi for Pallet<T>
    where
        T: Config,
    {
        type AccountId = T::AccountId;
        type Balance = BalanceOf<T>;
        type MarketId = MarketIdOf<T>;

        fn split_position_unsafe(
            who: Self::AccountId,
            collateral: Asset<Self::MarketId>,
            assets: Vec<Asset<Self::MarketId>>,
            amount: Self::Balance,
        ) -> DispatchResult {
            T::MultiCurrency::ensure_can_withdraw(collateral, &who, amount)?;
            T::MultiCurrency::transfer(collateral, &who, &Pallet::<T>::account_id(), amount)?;

            for &asset in assets.iter() {
                T::MultiCurrency::deposit(asset, &who, amount)?;
            }

            Ok(())
        }

        fn merge_position_unsafe(
            who: Self::AccountId,
            collateral: Asset<Self::MarketId>,
            assets: Vec<Asset<Self::MarketId>>,
            amount: Self::Balance,
        ) -> DispatchResult {
            T::MultiCurrency::transfer(collateral, &Pallet::<T>::account_id(), &who, amount)?;

            for &asset in assets.iter() {
                T::MultiCurrency::ensure_can_withdraw(asset, &who, amount)?;
                T::MultiCurrency::withdraw(asset, &who, amount)?;
            }

            Ok(())
        }
    }
}
