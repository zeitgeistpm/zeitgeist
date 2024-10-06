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

// TODO Refactor so that collection IDs are their own type with an `Fq` field and an `odd` field?

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
        traits::{AccountIdConversion, Get},
        DispatchError, DispatchResult,
    };
    use zeitgeist_primitives::{
        traits::MarketCommonsPalletApi,
        types::{Asset, CombinatorialId},
    };

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type CombinatorialIdManager: CombinatorialIdManager<
                Asset = AssetOf<Self>,
                MarketId = MarketIdOf<Self>,
                CombinatorialId = CombinatorialId,
            >;

        type MarketCommons: MarketCommonsPalletApi<AccountId = Self::AccountId, BlockNumber = BlockNumberFor<Self>>;

        type MultiCurrency: MultiCurrency<Self::AccountId, CurrencyId = AssetOf<Self>>;

        #[pallet::constant]
        type PalletId: Get<PalletId>;

        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
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

    // TODO Types
    pub(crate) const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

    // TODO Storage Items

    #[pallet::event]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T>
    where
        T: Config,
    {
        TokenSplit,
        TokenMerged,
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The specified partition is empty, contains overlaps or is too long.
        InvalidPartition,

        /// The specified collection ID is invalid.
        InvalidCollectionId,
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
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_split_position(who, parent_collection_id, market_id, partition, amount)
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
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_merge_position(who, parent_collection_id, market_id, partition, amount)
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
        ) -> DispatchResult {
            let market = T::MarketCommons::market(&market_id)?;
            let collateral_token = market.base_asset;

            let free_index_set = Self::free_index_set(market_id, &partition)?;

            // Destroy/store the tokens to be split.
            if !free_index_set.iter().any(|&i| i) {
                // Vertical split.
                if let Some(pci) = parent_collection_id {
                    // Split combinatorial token into higher level position. Destroy the tokens.
                    let position_id =
                        T::CombinatorialIdManager::get_position_id(collateral_token, pci);
                    let position = Asset::CombinatorialToken(position_id);
                    T::MultiCurrency::withdraw(position, &who, amount)?;
                } else {
                    // Split collateral into first level position. Store the collateral in the
                    // pallet account. This is the legacy `buy_complete_set`.
                    T::MultiCurrency::transfer(
                        collateral_token,
                        &who,
                        &Self::account_id(),
                        amount,
                    )?;
                }
            } else {
                // Horizontal split.
                let remaining_index_set = free_index_set.into_iter().map(|i| !i).collect();
                let position = Self::position_from_collection(
                    parent_collection_id,
                    market_id,
                    remaining_index_set,
                )?;
                T::MultiCurrency::withdraw(position, &who, amount)?;
            }

            // Deposit the new tokens.
            let position_ids = partition
                .iter()
                .cloned()
                .map(|index_set| {
                    Self::position_from_collection(parent_collection_id, market_id, index_set)
                })
                .collect::<Result<Vec<_>, _>>()?;
            for &position in position_ids.iter() {
                T::MultiCurrency::deposit(position, &who, amount)?;
            }

            Self::deposit_event(Event::<T>::TokenSplit);

            Ok(())
        }

        #[require_transactional]
        fn do_merge_position(
            who: AccountIdOf<T>,
            parent_collection_id: Option<CombinatorialIdOf<T>>,
            market_id: MarketIdOf<T>,
            partition: Vec<Vec<bool>>,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            let market = T::MarketCommons::market(&market_id)?;
            let collateral_token = market.base_asset;

            let free_index_set = Self::free_index_set(market_id, &partition)?;

            // Destory the old tokens.
            let position_ids = partition
                .iter()
                .cloned()
                .map(|index_set| {
                    Self::position_from_collection(parent_collection_id, market_id, index_set)
                })
                .collect::<Result<Vec<_>, _>>()?;
            for &position in position_ids.iter() {
                T::MultiCurrency::withdraw(position, &who, amount)?;
            }

            // Destroy/store the tokens to be split.
            if !free_index_set.iter().any(|&i| i) {
                // Vertical merge.
                if let Some(pci) = parent_collection_id {
                    // Merge combinatorial token into higher level position. Destroy the tokens.
                    let position_id =
                        T::CombinatorialIdManager::get_position_id(collateral_token, pci);
                    let position = Asset::CombinatorialToken(position_id);
                    T::MultiCurrency::deposit(position, &who, amount)?;
                } else {
                    // Merge first-level tokens into collateral. Move collateral from the pallet
                    // account to the user's wallet. This is the legacy `sell_complete_set`.
                    T::MultiCurrency::transfer(
                        collateral_token,
                        &Self::account_id(),
                        &who,
                        amount,
                    )?;
                }
            } else {
                // Horizontal merge.
                let remaining_index_set = free_index_set.into_iter().map(|i| !i).collect();
                let position = Self::position_from_collection(
                    parent_collection_id,
                    market_id,
                    remaining_index_set,
                )?;
                T::MultiCurrency::deposit(position, &who, amount)?;
            }

            Self::deposit_event(Event::<T>::TokenMerged);

            Ok(())
        }

        fn free_index_set(
            market_id: MarketIdOf<T>,
            partition: &[Vec<bool>],
        ) -> Result<Vec<bool>, DispatchError> {
            let market = T::MarketCommons::market(&market_id)?;
            let asset_count = market.outcomes() as usize;
            let mut free_index_set = vec![true; asset_count];

            for index_set in partition.iter() {
                // Ensure that the partition is not trivial.
                let ones = index_set.iter().fold(0usize, |acc, &val| acc + (val as usize));
                ensure!(ones > 0, Error::<T>::InvalidPartition);
                ensure!(ones < asset_count, Error::<T>::InvalidPartition);

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

        fn position_from_collection(
            parent_collection_id: Option<CombinatorialIdOf<T>>,
            market_id: MarketIdOf<T>,
            index_set: Vec<bool>,
        ) -> Result<AssetOf<T>, DispatchError> {
            let market = T::MarketCommons::market(&market_id)?;
            let collateral_token = market.base_asset;

            let collection_id = T::CombinatorialIdManager::get_collection_id(
                parent_collection_id,
                market_id,
                index_set,
                false, // TODO Expose this parameter!
            )
            .ok_or(Error::<T>::InvalidCollectionId)?;

            let position_id =
                T::CombinatorialIdManager::get_position_id(collateral_token, collection_id);
            let asset = Asset::CombinatorialToken(position_id);

            Ok(asset)
        }

        fn account_id() -> T::AccountId {
            T::PalletId::get().into_account_truncating()
        }
    }
}
