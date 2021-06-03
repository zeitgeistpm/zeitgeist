//! # Common market parameters used by `Court` and `Prediction Markets` pallets.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod market_commons_pallet_api;

pub use market_commons_pallet_api::MarketCommonsPalletApi;
pub use pallet::*;

#[frame_support::pallet]
mod pallet {
    use crate::MarketCommonsPalletApi;
    use core::marker::PhantomData;
    use frame_support::{pallet_prelude::StorageMap, traits::Hooks, Blake2_128Concat, Parameter};
    use sp_runtime::{DispatchError, traits::{AtLeast32Bit, MaybeSerializeDeserialize, Member}};
    use zeitgeist_primitives::types::Market;

    #[pallet::call]
    impl<T: Config> Pallet<T> {}

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The identifier of individual markets.
        type MarketId: AtLeast32Bit
            + Copy
            + Default
            + MaybeSerializeDeserialize
            + Member
            + Parameter;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// A market with the provided ID does not exist.
        MarketDoesNotExist,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    impl<T> MarketCommonsPalletApi for Pallet<T>
    where
        T: Config,
    {
        type AccountId = T::AccountId;
        type BlockNumber = T::BlockNumber;
        type MarketId = T::MarketId;

        fn market(
            market_id: &Self::MarketId,
        ) -> Result<Market<Self::AccountId, Self::BlockNumber>, DispatchError> {
            <Markets<T>>::try_get(market_id).map_err(|_err| Error::<T>::MarketDoesNotExist.into())
        }

        fn mutate_market<F>(market_id: &Self::MarketId, cb: F) -> Result<(), DispatchError>
        where
            F: FnOnce(&mut Market<Self::AccountId, Self::BlockNumber>),
        {
            <Markets<T>>::try_mutate(market_id, |opt| {
                if let Some(market) = opt {
                    cb(market);
                    return Ok(());
                }
                Err(Error::<T>::MarketDoesNotExist.into())
            })
        }

        fn insert_market(
            market_id: Self::MarketId,
            market: Market<Self::AccountId, Self::BlockNumber>,
        ) {
            <Markets<T>>::insert(market_id, market);
        }

        fn remove_market(market_id: &Self::MarketId) -> Result<(), DispatchError> {
            if !<Markets<T>>::contains_key(market_id) {
                return Err(Error::<T>::MarketDoesNotExist.into());
            }
            <Markets<T>>::remove(market_id);
            Ok(())
        }
    }

    /// Holds all markets
    #[pallet::storage]
    #[pallet::getter(fn markets)]
    pub type Markets<T: Config> =
        StorageMap<_, Blake2_128Concat, T::MarketId, Market<T::AccountId, T::BlockNumber>>;
}
