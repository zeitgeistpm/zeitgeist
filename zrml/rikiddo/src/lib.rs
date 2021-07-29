//! # Court
//!
//! Manages market disputes and resolutions.

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
    use core::{fmt::Debug, marker::PhantomData};
    use frame_support::{
        pallet_prelude::StorageMap,
        traits::{Hooks, Time},
        Twox64Concat,
    };
    use parity_scale_codec::{Decode, Encode, FullCodec, FullEncode};
    use substrate_fixed::{
        traits::{FixedSigned, FixedUnsigned, LossyFrom},
        types::{
            extra::{U127, U128, U31, U32},
            I9F23, U1F127,
        },
        FixedI128, FixedI32, FixedU128, FixedU32,
    };

    use crate::{
        traits::{MarketAverage, RikiddoSigmoidMVPallet, Sigmoid},
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
        type FixedTypeU: Decode
            + Encode
            + FixedUnsigned
            + LossyFrom<FixedU32<U32>>
            + LossyFrom<FixedU128<U128>>;

        /// Will be used for the fractional part of the fixed point numbers
        type FixedTypeS: Decode
            + Encode
            + FixedSigned
            + From<I9F23>
            + LossyFrom<FixedI32<U31>>
            + LossyFrom<U1F127>
            + LossyFrom<FixedI128<U127>>
            + PartialOrd<I9F23>;

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

    impl<T: Config> RikiddoSigmoidMVPallet for Pallet<T> {
        type Balance = T::Balance;
        type PoolId = T::PoolId;
        type FS = T::FixedTypeS;
        type FU = T::FixedTypeU;

        /// Clear market data for specific asset pool
        fn clear(poolid: Self::PoolId) {
            // TODO
        }

        /// Return cost C(q) for all assets in q
        fn cost(
            poolid: Self::PoolId,
            asset_balances: Vec<Self::Balance>,
        ) -> Result<Self::Balance, &'static str> {
            // TODO
            Err("Unimplemented!")
        }

        /// Create Rikiddo instance for specifc asset pool
        fn create(
            poolid: Self::PoolId,
            fee_config: FeeSigmoidConfig<Self::FS>,
            ema_config_short: EmaConfig<Self::FU>,
            ema_config_long: EmaConfig<Self::FU>,
            balance_one_unit: Self::Balance,
        ) {
            // TODO
        }

        /// Destroy Rikiddo instance
        fn destroy(poolid: Self::PoolId) {
            // TODO
        }

        /// Return price P_i(q) for asset q_i in q
        fn price(
            poolid: Self::PoolId,
            asset_in_question: Self::Balance,
            asset_balances: Vec<Self::Balance>,
        ) -> Result<Self::Balance, &'static str> {
            // TODO
            Err("Unimplemented!")
        }

        /// Return price P_i(q) for all assets in q
        fn all_prices(
            poolid: Self::PoolId,
            asset_balances: Vec<Self::Balance>,
        ) -> Result<Vec<Self::Balance>, &'static str> {
            // TODO
            Err("Unimplemented!")
        }

        /// Update market data
        fn update(
            poolid: Self::PoolId,
            volume: Self::Balance,
        ) -> Result<Option<Self::Balance>, &'static str> {
            // TODO
            Err("Unimplemented!")
        }
    }
}
