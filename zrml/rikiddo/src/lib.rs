//! # Rikiddo
//!
//! Manages prices of event assets within a pool

#![cfg_attr(not(feature = "std"), no_std)]
// This is required to be able to use the derive(Arbitrary) macro.
#![cfg_attr(feature = "arbitrary", allow(clippy::integer_arithmetic))]

extern crate alloc;

pub mod constants;
pub mod mock;
mod tests;
pub mod traits;
pub mod types;
pub use pallet::*;

#[frame_support::pallet]
mod pallet {
    use crate::{
        traits::{FromFixedDecimal, FromFixedToDecimal, Lmsr, RikiddoMV, RikiddoSigmoidMVPallet},
        types::{TimestampedVolume, UnixTimestamp},
    };
    use core::{
        convert::TryFrom,
        fmt::Debug,
        marker::PhantomData,
        ops::{AddAssign, BitOrAssign, ShlAssign},
    };
    use frame_support::{
        debug,
        dispatch::DispatchResult,
        pallet_prelude::StorageMap,
        traits::{Get, Hooks, Time},
        Twox64Concat,
    };
    use parity_scale_codec::{Decode, Encode, FullCodec, FullEncode};
    use sp_runtime::DispatchError;
    use substrate_fixed::{
        traits::{Fixed, FixedSigned, FixedUnsigned, LossyFrom, ToFixed},
        types::{
            extra::{U127, U128, U31, U32},
            I9F23, U1F127,
        },
        FixedI128, FixedI32, FixedU128, FixedU32,
    };
    #[pallet::config]
    pub trait Config<I: 'static = ()>: frame_system::Config {
        /// Defines the type of traded amounts
        type Balance: Copy + Into<u128> + TryFrom<u128> + Debug;

        /// Offers timestamping functionality
        type Timestamp: Time;

        /// Will be used for the fractional part of the fixed point numbers
        /// Calculation: Select FixedTYPE<UWIDTH>, such that TYPE = the type of Balance (i.e. FixedU128)
        /// Select the generic UWIDTH = floor(log2(10.pow(fractional_decimals)))
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
        #[pallet::constant]
        type BalanceFractionalDecimals: Get<u8>;

        /// Type that's used as an id for pools
        type PoolId: Copy + Decode + FullEncode;

        /// Rikiddo variant
        type Rikiddo: RikiddoMV<FU = Self::FixedTypeU> + Decode + FullCodec;
    }

    #[pallet::error]
    pub enum Error<T, I = ()> {
        FixedConversionImpossible,
        RikiddoNotFoundForPool,
        RikiddoAlreadyExistsForPool,
    }

    // This is the storage containing the Rikiddo instances per pool.
    #[pallet::storage]
    pub type RikiddoPerPool<T: Config<I>, I: 'static = ()> =
        StorageMap<_, Twox64Concat, T::PoolId, T::Rikiddo>;

    #[pallet::hooks]
    impl<T: Config<I>, I: 'static> Hooks<T::BlockNumber> for Pallet<T, I> {}

    #[pallet::pallet]
    pub struct Pallet<T, I = ()>(PhantomData<T>, PhantomData<I>);

    #[pallet::call]
    impl<T: Config<I>, I: 'static> Pallet<T, I> {}

    impl<T: Config<I>, I: 'static> Pallet<T, I> {
        fn get_rikiddo(poolid: &T::PoolId) -> Result<T::Rikiddo, DispatchError> {
            if let Ok(rikiddo) = <RikiddoPerPool<T, I>>::try_get(poolid) {
                Ok(rikiddo)
            } else {
                Err(Error::<T, I>::RikiddoNotFoundForPool.into())
            }
        }

        fn convert_balance_to_fixed(balance: &T::Balance) -> Result<T::FixedTypeU, DispatchError> {
            match T::FixedTypeU::from_fixed_decimal(*balance, T::BalanceFractionalDecimals::get()) {
                Ok(res) => Ok(res),
                Err(err) => {
                    debug(&err);
                    Err(Error::<T, I>::FixedConversionImpossible.into())
                }
            }
        }

        fn convert_fixed_to_balance(fixed: &T::FixedTypeU) -> Result<T::Balance, DispatchError> {
            match T::Balance::from_fixed_to_fixed_decimal(
                *fixed,
                T::BalanceFractionalDecimals::get(),
            ) {
                Ok(res) => Ok(res),
                Err(err) => {
                    debug(&err);
                    Err(Error::<T, I>::FixedConversionImpossible.into())
                }
            }
        }

        fn convert_balance_to_fixed_vector(
            balance: &[T::Balance],
        ) -> Result<Vec<T::FixedTypeU>, DispatchError> {
            balance
                .iter()
                .map(|e| Self::convert_balance_to_fixed(e))
                .collect::<Result<Vec<T::FixedTypeU>, DispatchError>>()
        }

        fn convert_fixed_to_balance_vector(
            fixed: &[T::FixedTypeU],
        ) -> Result<Vec<T::Balance>, DispatchError> {
            fixed
                .iter()
                .map(|e| Self::convert_fixed_to_balance(e))
                .collect::<Result<Vec<T::Balance>, DispatchError>>()
        }
    }

    impl<T: Config<I>, I: 'static> RikiddoSigmoidMVPallet for Pallet<T, I>
    where
        <T::FixedTypeS as Fixed>::Bits: Copy + ToFixed + AddAssign + BitOrAssign + ShlAssign,
        <T::Timestamp as Time>::Moment: Into<UnixTimestamp>,
    {
        type Balance = T::Balance;
        type PoolId = T::PoolId;
        type FU = T::FixedTypeU;
        type Rikiddo = T::Rikiddo;

        /// Return price P_i(q) for all assets in q
        fn all_prices(
            poolid: Self::PoolId,
            asset_balances: &[Self::Balance],
        ) -> Result<Vec<Self::Balance>, DispatchError> {
            let rikiddo = Self::get_rikiddo(&poolid)?;
            let balances_fixed = Self::convert_balance_to_fixed_vector(asset_balances)?;

            match rikiddo.all_prices(&balances_fixed) {
                Ok(prices) => Self::convert_fixed_to_balance_vector(&prices),
                Err(err) => {
                    debug(&err);
                    Err(err.into())
                }
            }
        }

        /// Clear market data for specific asset pool
        fn clear(poolid: Self::PoolId) -> Result<(), DispatchError> {
            let mut rikiddo = Self::get_rikiddo(&poolid)?;
            rikiddo.clear();
            Ok(())
        }

        /// Return cost C(q) for all assets in q
        fn cost(
            poolid: Self::PoolId,
            asset_balances: &[Self::Balance],
        ) -> Result<Self::Balance, DispatchError> {
            let rikiddo = Self::get_rikiddo(&poolid)?;
            let balances_fixed = Self::convert_balance_to_fixed_vector(asset_balances)?;

            match rikiddo.cost(&balances_fixed) {
                Ok(cost) => Self::convert_fixed_to_balance(&cost),
                Err(err) => {
                    debug(&err);
                    Err(err.into())
                }
            }
        }

        /// Create Rikiddo instance for specifc asset pool
        fn create(poolid: Self::PoolId, rikiddo: Self::Rikiddo) -> DispatchResult {
            if Self::get_rikiddo(&poolid).is_ok() {
                return Err(Error::<T, I>::RikiddoAlreadyExistsForPool.into());
            }

            <RikiddoPerPool<T, I>>::insert(poolid, rikiddo);
            Ok(())
        }

        /// Destroy Rikiddo instance
        fn destroy(poolid: Self::PoolId) -> DispatchResult {
            let _ = Self::get_rikiddo(&poolid)?;
            <RikiddoPerPool<T, I>>::remove(poolid);
            Ok(())
        }

        /// Fetch the current fee
        fn fee(poolid: Self::PoolId) -> Result<Self::Balance, DispatchError> {
            let rikiddo = Self::get_rikiddo(&poolid)?;

            match rikiddo.fee() {
                Ok(fee) => Self::convert_fixed_to_balance(&fee),
                Err(err) => {
                    debug(&err);
                    Err(err.into())
                }
            }
        }

        /// Return price P_i(q) for asset q_i in q
        fn price(
            poolid: Self::PoolId,
            asset_in_question: Self::Balance,
            asset_balances: &[Self::Balance],
        ) -> Result<Self::Balance, DispatchError> {
            let rikiddo = Self::get_rikiddo(&poolid)?;
            let balances_fixed = Self::convert_balance_to_fixed_vector(asset_balances)?;
            let balance_in_question = Self::convert_balance_to_fixed(&asset_in_question)?;

            match rikiddo.price(&balances_fixed, &balance_in_question) {
                Ok(price) => Self::convert_fixed_to_balance(&price),
                Err(err) => {
                    debug(&err);
                    Err(err.into())
                }
            }
        }

        /// Update market data
        fn update_volume(
            poolid: Self::PoolId,
            volume: Self::Balance,
        ) -> Result<Option<Self::Balance>, DispatchError> {
            // Convert to Fixed type
            let timestamp: UnixTimestamp = T::Timestamp::now().into();
            let volume_fixed: Self::FU = Self::convert_balance_to_fixed(&volume)?;

            let timestamped_volume = TimestampedVolume { timestamp, volume: volume_fixed };
            let mut rikiddo = Self::get_rikiddo(&poolid)?;

            // Update rikiddo market data by adding the TimestampedVolume
            let balance_fixed = match rikiddo.update_volume(&timestamped_volume) {
                Ok(Some(inner)) => inner,
                Ok(None) => {
                    <RikiddoPerPool<T, I>>::insert(poolid, rikiddo);
                    return Ok(None);
                }
                Err(err) => {
                    debug(&err);
                    return Err(err.into());
                }
            };

            // Convert result back into Balance type
            let result = Self::convert_fixed_to_balance(&balance_fixed)?;
            <RikiddoPerPool<T, I>>::insert(poolid, rikiddo);
            Ok(Some(result))
        }
    }
}
