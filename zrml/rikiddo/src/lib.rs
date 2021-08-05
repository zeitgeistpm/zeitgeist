//! # Rikiddo
//!
//! Manages prices of event assets within a pool

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod constants;
mod mock;
mod tests;
pub mod traits;
pub mod types;
pub use pallet::*;

#[frame_support::pallet]
mod pallet {
    use frame_support::{
        debug,
        dispatch::DispatchResult,
        pallet_prelude::StorageMap,
        traits::{Get, Hooks, Time},
        Twox64Concat,
    };
    use parity_scale_codec::{Decode, Encode, FullCodec, FullEncode};
    use sp_runtime::DispatchError;
    use sp_std::{
        marker::PhantomData,
        ops::{AddAssign, BitOrAssign, ShlAssign},
    };
    use substrate_fixed::{
        traits::{Fixed, FixedSigned, FixedUnsigned, LossyFrom, ToFixed},
        types::{
            extra::{U127, U128, U31, U32},
            I9F23, U1F127,
        },
        FixedI128, FixedI32, FixedU128, FixedU32,
    };

    use crate::{
        traits::{MarketAverage, RikiddoMV, RikiddoSigmoidMVPallet, Sigmoid},
        types::{EmaConfig, FeeSigmoidConfig, RikiddoConfig, RikiddoSigmoidMV},
    };
    use parity_scale_codec::Codec;
    use sp_runtime::traits::AtLeast32BitUnsigned;
    use substrate_fixed::types::extra::LeEqU128;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Defines the type of traded amounts
        type Balance;

        /// Offers timestamping functionality
        type Timestamp: Time;

        /// Will be used for the fractional part of the fixed point numbers
        /// Calculation: Select FixedTYPE<UWIDTH>, such that TYPE = the type of Balance (i.e. FixedU128)
        /// Select the generic UWIDTH = floor(log2(fractional_decimals))
        type FixedTypeU: Decode
            + Encode
            + FixedUnsigned
            + LossyFrom<FixedU32<U32>>
            + LossyFrom<FixedU128<U128>>;

        /// Will be used for the fractional part of the fixed point numbers
        /// Calculation: Select FixedTYPE, such that it is the signed variant of FixedTypeU
        /// It is possible to reduce the fractional bit count by one, effectively eliminating
        /// conversion overflows when the MSB of the unsigned fixed type is set, but in exchange
        /// Reducing the fractional precision by one bit.
        type FixedTypeS: Decode
            + Encode
            + FixedSigned
            + From<I9F23>
            + LossyFrom<FixedI32<U31>>
            + LossyFrom<U1F127>
            + LossyFrom<FixedI128<U127>>
            + PartialOrd<I9F23>;

        // Number of fractional decimal places for one unit of currency
        type BalanceFractionalDecimals: Get<u8>;

        /// Type that's used as an id for pools
        type PoolId: Copy + Decode + FullEncode;

        /// Rikiddo variant
        type Rikiddo: RikiddoMV<FU = Self::FixedTypeU> + Decode + FullCodec;
    }

    #[pallet::error]
    pub enum Error<T> {
        ArithmeticOverflow,
        FixedConversionImpossible,
        RikiddoNotFoundForPool,
        RikiddoAlreadyExistsForPool,
    }

    // This is the storage containing the Rikiddo instances per pool.
    #[pallet::storage]
    pub type RikiddoPerPool<T: Config> = StorageMap<_, Twox64Concat, T::PoolId, T::Rikiddo>;

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::call]
    impl<T: Config> Pallet<T> {}

    impl<T: Config> Pallet<T> {
        fn get_lmsr(poolid: T::PoolId) -> Result<T::Rikiddo, DispatchError> {
            if let Ok(lmsr) = <RikiddoPerPool<T>>::try_get(poolid) {
                Ok(lmsr)
            } else {
                Err(Error::<T>::RikiddoNotFoundForPool.into())
            }
        }
    }

    impl<T: Config> RikiddoSigmoidMVPallet for Pallet<T>
    where
        <T::FixedTypeS as Fixed>::Bits: Copy + ToFixed + AddAssign + BitOrAssign + ShlAssign,
    {
        type Balance = T::Balance;
        type PoolId = T::PoolId;
        type FU = T::FixedTypeU;
        type Rikiddo = T::Rikiddo;

        /// Clear market data for specific asset pool
        fn clear(poolid: Self::PoolId) -> Result<(), DispatchError> {
            let mut lmsr = Self::get_lmsr(poolid)?;
            lmsr.clear();
            Ok(())
        }

        /// Return cost C(q) for all assets in q
        fn cost(
            poolid: Self::PoolId,
            asset_balances: Vec<Self::Balance>,
        ) -> Result<Self::Balance, DispatchError> {
            // TODO
            Err("Unimplemented!".into())
        }

        /// Create Rikiddo instance for specifc asset pool
        fn create(poolid: Self::PoolId, rikiddo: Self::Rikiddo) -> DispatchResult {
            if Self::get_lmsr(poolid).is_ok() {
                return Err(Error::<T>::RikiddoAlreadyExistsForPool.into());
            }

            <RikiddoPerPool<T>>::insert(poolid, rikiddo);
            Ok(())
        }

        /// Destroy Rikiddo instance
        fn destroy(poolid: Self::PoolId) -> DispatchResult {
            let _ = Self::get_lmsr(poolid)?;
            <RikiddoPerPool<T>>::remove(poolid);
            Ok(())
        }

        /// Return price P_i(q) for asset q_i in q
        fn price(
            poolid: Self::PoolId,
            asset_in_question: Self::Balance,
            asset_balances: Vec<Self::Balance>,
        ) -> Result<Self::Balance, DispatchError> {
            // TODO
            Err("Unimplemented!".into())
        }

        /// Return price P_i(q) for all assets in q
        fn all_prices(
            poolid: Self::PoolId,
            asset_balances: Vec<Self::Balance>,
        ) -> Result<Vec<Self::Balance>, DispatchError> {
            // TODO
            Err("Unimplemented!".into())
        }

        /// Update market data
        fn update(
            poolid: Self::PoolId,
            volume: Self::Balance,
        ) -> Result<Option<Self::Balance>, DispatchError> {
            // TODO
            Err("Unimplemented!".into())
        }
    }
}
