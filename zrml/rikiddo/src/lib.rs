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
    use frame_support::{debug, Twox64Concat, dispatch::DispatchResult, pallet_prelude::StorageMap, traits::{Get, Hooks, Time}};
    use parity_scale_codec::{Decode, Encode, FullCodec, FullEncode};
    use sp_std::{ops::{AddAssign, BitOrAssign, ShlAssign}, marker::PhantomData};
    use sp_runtime::DispatchError;
    use substrate_fixed::{FixedI128, FixedI32, FixedU128, FixedU32, traits::{Fixed, FixedSigned, FixedUnsigned, LossyFrom, ToFixed}, types::{
            extra::{U127, U128, U31, U32},
            I9F23, U1F127,
        }};

    use crate::{
        traits::{MarketAverage, RikiddoMV, RikiddoSigmoidMVPallet, Sigmoid},
        types::{EmaConfig, FeeSigmoidConfig, RikiddoSigmoidMV},
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
            //+ Self::Bits::Copy;

        // Number of fractional decimal places for one unit of currency
        type BalanceFractionalDecimals: Get<u8>;

        /// Type that's used as an id for pools
        type PoolId: Decode + FullEncode;

        /// Type that's used to gather market data
        type MarketData: MarketAverage<FU = Self::FixedTypeU> + Decode + Encode;

        /// Type that's used to calculate fees
        type Fees: Sigmoid<FS = Self::FixedTypeS> + Decode + FullCodec;
    }

    #[pallet::error]
    pub enum Error<T> {
        ArithmeticOverflow,
        FixedConversionImpossible,
        PoolNotFound,
    }

    // This is the storage containing the Rikiddo instances per pool.
    #[pallet::storage]
    pub type LmsrPerPool<T: Config> = StorageMap<
        _,
        Twox64Concat,
        T::PoolId,
        RikiddoSigmoidMV<T::FixedTypeU, T::FixedTypeS, T::Fees, T::MarketData>,
    >;

    #[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::call]
    impl<T: Config> Pallet<T> {}

    impl<T: Config> RikiddoSigmoidMVPallet for Pallet<T> where
        <T::FixedTypeS as Fixed>::Bits: Copy + ToFixed + AddAssign + BitOrAssign + ShlAssign {
            
        type Balance = T::Balance;
        type PoolId = T::PoolId;
        type FS = T::FixedTypeS;
        type FU = T::FixedTypeU;

        /// Clear market data for specific asset pool
        fn clear(poolid: Self::PoolId) -> Result<(), DispatchError> {
            if let Ok(mut lmsr) = <LmsrPerPool<T>>::try_get(poolid) {
                lmsr.clear();
                Ok(())
            }
            else {
                Err(Error::<T>::PoolNotFound.into())
            }
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
        fn create(
            poolid: Self::PoolId,
            fee_config: FeeSigmoidConfig<Self::FS>,
            ema_config_short: EmaConfig<Self::FU>,
            ema_config_long: EmaConfig<Self::FU>,
            balance_one_unit: Self::Balance,
        ) -> DispatchResult {
            // TODO
            Err("Unimplemented!".into())
        }

        /// Destroy Rikiddo instance
        fn destroy(poolid: Self::PoolId) -> DispatchResult {
            // TODO
            Err("Unimplemented!".into())
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
