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
        convert::TryFrom,
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
        types::{
            EmaConfig, FeeSigmoidConfig, IntoFixedDecimal, IntoFixedFromDecimal, RikiddoConfig, RikiddoSigmoidMV,
            TimestampedVolume, UnixTimestamp,
        },
    };
    use parity_scale_codec::Codec;
    use sp_runtime::traits::AtLeast32BitUnsigned;
    use substrate_fixed::types::extra::LeEqU128;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Defines the type of traded amounts
        type Balance: Into<u128> + TryFrom<u128>;

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
        TransactionIsOlderThanPrevious,
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
        fn get_rikiddo(poolid: T::PoolId) -> Result<T::Rikiddo, DispatchError> {
            if let Ok(rikiddo) = <RikiddoPerPool<T>>::try_get(poolid) {
                Ok(rikiddo)
            } else {
                Err(Error::<T>::RikiddoNotFoundForPool.into())
            }
        }
    }

    impl<T: Config> RikiddoSigmoidMVPallet for Pallet<T>
    where
        <T::FixedTypeS as Fixed>::Bits: Copy + ToFixed + AddAssign + BitOrAssign + ShlAssign,
        <T::Timestamp as Time>::Moment: Into<UnixTimestamp>,
    {
        type Balance = T::Balance;
        type PoolId = T::PoolId;
        type FU = T::FixedTypeU;
        type Rikiddo = T::Rikiddo;

        /// Clear market data for specific asset pool
        fn clear(poolid: Self::PoolId) -> Result<(), DispatchError> {
            let mut rikiddo = Self::get_rikiddo(poolid)?;
            rikiddo.clear();
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
            if Self::get_rikiddo(poolid).is_ok() {
                return Err(Error::<T>::RikiddoAlreadyExistsForPool.into());
            }

            <RikiddoPerPool<T>>::insert(poolid, rikiddo);
            Ok(())
        }

        /// Destroy Rikiddo instance
        fn destroy(poolid: Self::PoolId) -> DispatchResult {
            let _ = Self::get_rikiddo(poolid)?;
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
            // Convert to Fixed type
            let timestamp: UnixTimestamp = T::Timestamp::now().into();
            let volume_fixed: Self::FU;

            match volume.to_fixed_from_fixed_decimal(T::BalanceFractionalDecimals::get()) {
                Ok(res) => volume_fixed = res,
                Err(err) => {
                    debug(&err);
                    return Err(Error::<T>::FixedConversionImpossible.into());
                }
            };

            let timestamped_volume =
                TimestampedVolume { timestamp: timestamp.into(), volume: volume_fixed };
            let mut rikiddo = Self::get_rikiddo(poolid)?;

            // Update rikiddo market data by adding the TimestampedVolume
            let balance_fixed = match rikiddo.update(&timestamped_volume) {
                Ok(res) => {
                    if let Some(inner) = res {
                        inner
                    } else  {
                        <RikiddoPerPool<T>>::insert(poolid, rikiddo);
                        return Ok(None);
                    }
                }
                Err(err) => {
                    debug(&err);

                    if err == "[EmaMarketVolume] Incoming volume timestamp is older than previous timestamp" {
                        // Using the default Timestamp pallet makes this branch unreachable
                        return Err(Error::<T>::TransactionIsOlderThanPrevious.into());
                    } else {
                        return Err(Error::<T>::ArithmeticOverflow.into());
                    }
                }
            };

            // Convert result back into Balance type
            let converted: Result<T::Balance, &'static str> = balance_fixed.to_fixed_decimal(T::BalanceFractionalDecimals::get());
            
            match converted {
                Ok(res) => {
                    <RikiddoPerPool<T>>::insert(poolid, rikiddo);
                    return Ok(Some(res));
                },
                Err(err) => {
                    debug(&err);
                    return Err(Error::<T>::FixedConversionImpossible.into());
                }
            }
        }
    }
}
