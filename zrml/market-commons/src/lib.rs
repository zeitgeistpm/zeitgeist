// Copyright 2022-2024 Forecasting Technologies LTD.
// Copyright 2021-2022 Zeitgeist PM LLC.
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

pub mod migrations;
mod mock;
mod tests;
pub mod types;

pub use pallet::*;
pub use zeitgeist_primitives::traits::MarketCommonsPalletApi;

#[frame_support::pallet]
mod pallet {
    use crate::MarketCommonsPalletApi;
    use alloc::fmt::Debug;
    use core::marker::PhantomData;
    use frame_support::{
        dispatch::DispatchResult,
        ensure,
        pallet_prelude::{StorageMap, StorageValue, ValueQuery},
        storage::PrefixIterator,
        traits::{Hooks, StorageVersion, Time},
        Blake2_128Concat, Parameter,
    };
    use frame_system::pallet_prelude::BlockNumberFor;
    use parity_scale_codec::{FullCodec, MaxEncodedLen};
    use sp_runtime::{
        traits::{
            AtLeast32Bit, AtLeast32BitUnsigned, MaybeSerializeDeserialize, Member, Saturating,
        },
        DispatchError,
    };
    use zeitgeist_primitives::{
        math::checked_ops_res::CheckedAddRes,
        traits::MarketBuilderTrait,
        types::{Asset, Deadlines, EarlyClose, Market, MarketBonds, MarketPeriod, PoolId, Report},
    };

    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(11);

    pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
    pub(crate) type AssetOf<T> = Asset<MarketIdOf<T>>;
    pub(crate) type BalanceOf<T> = <T as Config>::Balance;
    pub(crate) type MarketIdOf<T> = <T as Config>::MarketId;
    pub(crate) type MarketOf<T> =
        Market<AccountIdOf<T>, BalanceOf<T>, BlockNumberFor<T>, MomentOf<T>, MarketIdOf<T>>;
    pub(crate) type MomentOf<T> = <<T as Config>::Timestamp as frame_support::traits::Time>::Moment;
    pub(crate) type DeadlinesOf<T> = Deadlines<BlockNumberFor<T>>;
    pub(crate) type EarlyCloseOf<T> = EarlyClose<BlockNumberFor<T>, MomentOf<T>>;
    pub(crate) type MarketBondsOf<T> = MarketBonds<AccountIdOf<T>, BalanceOf<T>>;
    pub(crate) type MarketPeriodOf<T> = MarketPeriod<BlockNumberFor<T>, MomentOf<T>>;
    pub(crate) type ReportOf<T> = Report<AccountIdOf<T>, BlockNumberFor<T>>;

    #[pallet::call]
    impl<T: Config> Pallet<T> {}

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Balance: AtLeast32BitUnsigned
            + FullCodec
            + Copy
            + MaybeSerializeDeserialize
            + Debug
            + Default
            + scale_info::TypeInfo
            + MaxEncodedLen;

        /// The identifier of individual markets.
        type MarketId: AtLeast32Bit
            + Copy
            + Default
            + MaxEncodedLen
            + MaybeSerializeDeserialize
            + Member
            + Parameter;

        /// Time tracker
        type Timestamp: Time<Moment = u64>;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// A market with the provided ID does not exist.
        MarketDoesNotExist,
        /// Market does not have an stored associated pool id.
        MarketPoolDoesNotExist,
        /// It is not possible to fetch the latest market ID when
        /// no market has been created.
        NoMarketHasBeenCreated,
        /// Market does not have a report
        NoReport,
        /// There's a pool registered for this market already.
        PoolAlreadyExists,
        /// Unexpectedly failed to build a market due to missing data.
        IncompleteMarketBuilder,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(PhantomData<T>);

    impl<T> Pallet<T>
    where
        T: Config,
    {
        // Stored and returns the next market id.
        //
        // Retrieval is based on the stored ID plus one, recording the same incremented number
        // on the storage so next following calls will return yet another incremented number.
        //
        // Returns `Err` if `MarketId` addition overflows.
        fn next_market_id() -> Result<T::MarketId, DispatchError> {
            let id = MarketCounter::<T>::get();
            let new_counter = id.checked_add_res(&1u8.into())?;
            <MarketCounter<T>>::put(new_counter);
            Ok(id)
        }
    }

    impl<T> MarketCommonsPalletApi for Pallet<T>
    where
        T: Config,
        <T as Config>::Timestamp: Time,
    {
        type AccountId = AccountIdOf<T>;
        type BlockNumber = BlockNumberFor<T>;
        type Balance = T::Balance;
        type MarketId = T::MarketId;
        type Moment = MomentOf<T>;

        // Market

        fn latest_market_id() -> Result<Self::MarketId, DispatchError> {
            match <MarketCounter<T>>::try_get() {
                Ok(market_id) => {
                    Ok(market_id.saturating_sub(1u8.into())) // Note: market_id > 0!
                }
                _ => Err(Error::<T>::NoMarketHasBeenCreated.into()),
            }
        }

        fn market_iter() -> PrefixIterator<(Self::MarketId, MarketOf<T>)> {
            <Markets<T>>::iter()
        }

        fn market(market_id: &Self::MarketId) -> Result<MarketOf<T>, DispatchError> {
            <Markets<T>>::try_get(market_id).map_err(|_err| Error::<T>::MarketDoesNotExist.into())
        }

        fn mutate_market<F>(market_id: &Self::MarketId, cb: F) -> DispatchResult
        where
            F: FnOnce(&mut MarketOf<T>) -> DispatchResult,
        {
            <Markets<T>>::try_mutate(market_id, |opt| {
                if let Some(market) = opt {
                    cb(market)?;
                    return Ok(());
                }
                Err(Error::<T>::MarketDoesNotExist.into())
            })
        }

        fn push_market(mut market: MarketOf<T>) -> Result<Self::MarketId, DispatchError> {
            let market_id = Self::next_market_id()?;
            market.market_id = market_id;
            Markets::<T>::insert(market_id, market.clone());
            Ok(market_id)
        }

        fn build_market<U>(
            mut market_builder: U,
        ) -> Result<(Self::MarketId, MarketOf<T>), DispatchError>
        where
            U: MarketBuilderTrait<
                    Self::AccountId,
                    Self::Balance,
                    Self::BlockNumber,
                    Self::Moment,
                    Self::MarketId,
                >,
        {
            let market_id = Self::next_market_id()?;
            market_builder.market_id(market_id);
            let market = market_builder.build()?;
            <Markets<T>>::insert(market_id, market.clone());
            Ok((market_id, market))
        }

        fn remove_market(market_id: &Self::MarketId) -> DispatchResult {
            if !<Markets<T>>::contains_key(market_id) {
                return Err(Error::<T>::MarketDoesNotExist.into());
            }
            <Markets<T>>::remove(market_id);
            Ok(())
        }

        // MarketPool

        fn insert_market_pool(market_id: Self::MarketId, pool_id: PoolId) -> DispatchResult {
            ensure!(!<MarketPool<T>>::contains_key(market_id), Error::<T>::PoolAlreadyExists);
            ensure!(<Markets<T>>::contains_key(market_id), Error::<T>::MarketDoesNotExist);
            <MarketPool<T>>::insert(market_id, pool_id);
            Ok(())
        }

        fn remove_market_pool(market_id: &Self::MarketId) -> DispatchResult {
            if !<MarketPool<T>>::contains_key(market_id) {
                return Err(Error::<T>::MarketPoolDoesNotExist.into());
            }
            <MarketPool<T>>::remove(market_id);
            Ok(())
        }

        fn market_pool(market_id: &Self::MarketId) -> Result<PoolId, DispatchError> {
            <MarketPool<T>>::try_get(market_id)
                .map_err(|_err| Error::<T>::MarketPoolDoesNotExist.into())
        }

        // Etc

        fn now() -> Self::Moment {
            T::Timestamp::now()
        }
    }

    /// Holds all markets
    #[pallet::storage]
    pub type Markets<T: Config> = StorageMap<_, Blake2_128Concat, T::MarketId, MarketOf<T>>;

    /// The number of markets that have been created (including removed markets) and the next
    /// identifier for a created market.
    #[pallet::storage]
    pub type MarketCounter<T: Config> = StorageValue<_, T::MarketId, ValueQuery>;

    /// Maps a market ID to a related pool ID. It is up to the caller to keep and sync valid
    /// existent markets with valid existent pools.
    ///
    /// Beware! DEPRECATED as of v0.5.0.
    #[pallet::storage]
    pub type MarketPool<T: Config> = StorageMap<_, Blake2_128Concat, T::MarketId, PoolId>;
}
