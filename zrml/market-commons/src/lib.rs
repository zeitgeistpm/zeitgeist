//! # Common market parameters used by `Simple disputes` and `Prediction markets` pallets.
//!
//! As stated by the contract of `MarketCommonsPalletApi::now`, the caller must ensure that the
//! time implementation returns milliseconds.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod market_commons_pallet_api;

pub use market_commons_pallet_api::MarketCommonsPalletApi;
pub use pallet::*;

#[frame_support::pallet]
mod pallet {
    use crate::MarketCommonsPalletApi;
    use alloc::vec::Vec;
    use core::marker::PhantomData;
    use frame_support::{
        dispatch::DispatchResult,
        pallet_prelude::{StorageMap, StorageValue, ValueQuery},
        traits::{Hooks, NamedReservableCurrency, Time},
        Blake2_128Concat, Parameter,
    };
    use sp_runtime::{
        traits::{AtLeast32Bit, CheckedAdd, MaybeSerializeDeserialize, Member},
        ArithmeticError, DispatchError,
    };
    use zeitgeist_primitives::types::{Market, PoolId, Report};

    type MomentOf<T> = <<T as Config>::Timestamp as frame_support::traits::Time>::Moment;

    #[pallet::call]
    impl<T: Config> Pallet<T> {}

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Native token
        //
        // Reserve identifiers can be pallet ids or any other sequence of bytes.
        type Currency: NamedReservableCurrency<Self::AccountId, ReserveIdentifier = [u8; 8]>;

        /// The identifier of individual markets.
        type MarketId: AtLeast32Bit
            + Copy
            + Default
            + MaybeSerializeDeserialize
            + Member
            + Parameter;

        /// Time tracker
        type Timestamp: Time;
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
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

    #[pallet::pallet]
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
            let id = if let Ok(current) = MarketCounter::<T>::try_get() {
                current.checked_add(&T::MarketId::from(1u8)).ok_or(ArithmeticError::Overflow)?
            } else {
                T::MarketId::from(0u8)
            };
            <MarketCounter<T>>::put(id);
            Ok(id)
        }
    }

    impl<T> MarketCommonsPalletApi for Pallet<T>
    where
        T: Config,
    {
        type AccountId = T::AccountId;
        type BlockNumber = T::BlockNumber;
        type Currency = T::Currency;
        type MarketId = T::MarketId;
        type Moment = MomentOf<T>;

        // Market

        fn latest_market_id() -> Result<Self::MarketId, DispatchError> {
            <MarketCounter<T>>::try_get().map_err(|_err| Error::<T>::NoMarketHasBeenCreated.into())
        }

        fn market(
            market_id: &Self::MarketId,
        ) -> Result<Market<Self::AccountId, Self::BlockNumber, Self::Moment>, DispatchError>
        {
            <Markets<T>>::try_get(market_id).map_err(|_err| Error::<T>::MarketDoesNotExist.into())
        }

        fn markets()
        -> Vec<(Self::MarketId, Market<Self::AccountId, Self::BlockNumber, Self::Moment>)> {
            <Markets<T>>::iter().collect()
        }

        fn mutate_market<F>(market_id: &Self::MarketId, cb: F) -> DispatchResult
        where
            F: FnOnce(
                &mut Market<Self::AccountId, Self::BlockNumber, Self::Moment>,
            ) -> DispatchResult,
        {
            <Markets<T>>::try_mutate(market_id, |opt| {
                if let Some(market) = opt {
                    cb(market)?;
                    return Ok(());
                }
                Err(Error::<T>::MarketDoesNotExist.into())
            })
        }

        fn push_market(
            market: Market<Self::AccountId, Self::BlockNumber, Self::Moment>,
        ) -> Result<Self::MarketId, DispatchError> {
            let market_id = Self::next_market_id()?;
            <Markets<T>>::insert(market_id, market);
            Ok(market_id)
        }

        fn remove_market(market_id: &Self::MarketId) -> DispatchResult {
            if !<Markets<T>>::contains_key(market_id) {
                return Err(Error::<T>::MarketDoesNotExist.into());
            }
            <Markets<T>>::remove(market_id);
            Ok(())
        }

        fn report(
            market: &Market<Self::AccountId, Self::BlockNumber, Self::Moment>,
        ) -> Result<&Report<Self::AccountId, Self::BlockNumber>, DispatchError> {
            let report = market.report.as_ref().ok_or(Error::<T>::NoReport)?;
            Ok(report)
        }

        // MarketPool

        fn insert_market_pool(market_id: Self::MarketId, pool_id: PoolId) {
            <MarketPool<T>>::insert(market_id, pool_id);
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
    pub type Markets<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::MarketId,
        Market<T::AccountId, T::BlockNumber, MomentOf<T>>,
    >;

    /// The number of markets that have been created (including removed markets) and the next
    /// identifier for a created market.
    #[pallet::storage]
    pub type MarketCounter<T: Config> = StorageValue<_, T::MarketId, ValueQuery>;

    /// Maps a market id to a related pool id. It is up to the caller to keep and sync valid
    /// existent markets with valid existent pools.
    #[pallet::storage]
    pub type MarketPool<T: Config> = StorageMap<_, Blake2_128Concat, T::MarketId, PoolId>;
}
