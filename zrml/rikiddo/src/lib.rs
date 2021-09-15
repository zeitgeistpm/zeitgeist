//! # Rikiddo
//!
//! Generic and modular implemenation of Rikiddo market scoring rule.
//!
//! Provides traits and implementations for sigmoid fee caluclation, calculation of ema based on
//! market volume, LMSR and Rikiddo using sigmoid fee calculation and two ema periods.
//!
//! Rikiddo is a liquidity-sensitive logarithm market scoring algorithm, which can be used
//! to determine the prices of event assets and their corresponding probabilities. It incorporates
//! historical trading data to optimize it's reactiveness to abrupt and longer lasting changes
//! in the market trend. More information at [blog.zeitgeist.pm].
//!
//! [blog.zeitgeist.pm]: https://blog.zeitgeist.pm/introducing-zeitgeists-rikiddo-scoring-rule/

#![cfg_attr(not(feature = "std"), no_std)]
// This is required to be able to use the derive(Arbitrary) macro.
#![cfg_attr(feature = "arbitrary", allow(clippy::integer_arithmetic))]
#![deny(missing_docs)]

extern crate alloc;

pub mod constants;
pub mod mock;
mod tests;
pub mod traits;
pub mod types;
pub use pallet::*;

/// The pallet that bridges Rikiddo instances to pools.
///
/// Abstracts Rikiddo's core functions to be used within a Substrate chain.
///
/// This implementation of the Rikiddo pallet is solely a "bookkeeper" of Rikiddo instances,
/// i.e. it can spawn, update, destroy and use a specific instance to retrieve price
/// information for a given set of assets. Internally it uses a Rikiddo implementation that
/// is based on a port of the rust fixed library, [substrate-fixed], consequently it must
/// handle all conversions between the `Balance` type and the selected fixed point type.
///
/// This pallet is highly configurable, you can select the balance type, the fixed point types
/// and the actual implementation of Rikiddo (for example ema or wma of market data) for your
/// specific use case. By using multiple instances, potentially multiple Rikiddo variants
/// can run simulatenously on one chain, which can be used to ease migrations.
///
/// [substrate-fixed]: https://github.com/encointer/substrate-fixed
#[frame_support::pallet]
// The allow(missing_docs) attribute seems to be necessary, because some attribute-like macros
// from the Substrate framework generate undocumented code. It seems to be impossible to move
// the code into an anonymous module to resolve this issue.
#[allow(missing_docs)]
pub mod pallet {
    use crate::{
        traits::{FromFixedDecimal, FromFixedToDecimal, Lmsr, RikiddoMV, RikiddoMVPallet},
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
        /// Defines the type of traded amounts.
        type Balance: Copy + Into<u128> + TryFrom<u128> + Debug;

        /// Offers timestamping functionality.
        type Timestamp: Time;

        /// Will be used for the fractional part of the fixed point numbers.
        /// Calculation: Select FixedTYPE<UWIDTH>, such that TYPE = the type of Balance (i.e. FixedU128)
        /// Select the generic UWIDTH = floor(log2(10.pow(fractional_decimals))).
        type FixedTypeU: Decode
            + Encode
            + FixedUnsigned
            + LossyFrom<FixedU32<U32>>
            + LossyFrom<FixedU128<U128>>;

        /// Will be used for the fractional part of the fixed point numbers.
        /// Calculation: Select FixedTYPE, such that it is the signed variant of FixedTypeU.
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

        /// Number of fractional decimal places for one unit of currency.
        #[pallet::constant]
        type BalanceFractionalDecimals: Get<u8>;

        /// Type that's used as an id for pools.
        type PoolId: Copy + Decode + FullEncode;

        /// Rikiddo variant.
        type Rikiddo: RikiddoMV<FU = Self::FixedTypeU> + Decode + FullCodec;
    }

    
    /// Potential errors within the Rikiddo pallet.
    #[pallet::error]
    pub enum Error<T, I = ()> {
        /// Conversion between the `Balance` and the internal Rikiddo core type failed.
        FixedConversionImpossible,
        /// For a given `poolid`, no Rikiddo instance could be found.
        RikiddoNotFoundForPool,
        /// Trying to create a Rikiddo instance for a `poolid` that already has a Rikiddo instance.
        RikiddoAlreadyExistsForPool,
    }

    /// Storage that maps pool ids to Rikiddo instances.
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

    impl<T: Config<I>, I: 'static> RikiddoMVPallet for Pallet<T, I>
    where
        <T::FixedTypeS as Fixed>::Bits: Copy + ToFixed + AddAssign + BitOrAssign + ShlAssign,
        <T::Timestamp as Time>::Moment: Into<UnixTimestamp>,
    {
        type Balance = T::Balance;
        type PoolId = T::PoolId;
        type FU = T::FixedTypeU;
        type Rikiddo = T::Rikiddo;

        /// Returns a vector of prices for a given set of assets (same order as `asset_balances`).
        ///
        /// # Arguments
        ///
        /// * `poolid`: The id of the asset pool for which all asset prices shall be calculated.
        /// * `asset_balances`: The balance vector of the assets.
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

        /// Clear market data for a specific asset pool.
        ///
        /// # Arguments
        ///
        /// * `poolid`: The id of the asset pool for which all asset prices shall be calculated.
        fn clear(poolid: Self::PoolId) -> Result<(), DispatchError> {
            let mut rikiddo = Self::get_rikiddo(&poolid)?;
            rikiddo.clear();
            Ok(())
        }

        /// Returns the total cost for a specific vector of assets (see [LS-LMSR paper]).
        ///
        /// [LS-LMSR paper]: https://www.eecs.harvard.edu/cs286r/courses/fall12/papers/OPRS10.pdf
        /// # Arguments
        ///
        /// * `poolid`: The id of the asset pool for which all asset prices shall be calculated.
        /// * `asset_balances`: The balance vector of the assets.
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

        /// Create Rikiddo instance for specifc asset pool.
        ///
        /// # Arguments
        ///
        /// * `poolid`: The id of the asset pool for which all asset prices shall be calculated.
        /// * `rikiddo`: A specific type of Rikiddo as specified in the pallet's configuration.
        fn create(poolid: Self::PoolId, rikiddo: Self::Rikiddo) -> DispatchResult {
            if Self::get_rikiddo(&poolid).is_ok() {
                return Err(Error::<T, I>::RikiddoAlreadyExistsForPool.into());
            }

            <RikiddoPerPool<T, I>>::insert(poolid, rikiddo);
            Ok(())
        }

        /// Destroy Rikiddo instance for a specific pool.
        ///
        /// # Arguments
        ///
        /// * `poolid`: The id of the asset pool for which all asset prices shall be calculated.
        fn destroy(poolid: Self::PoolId) -> DispatchResult {
            let _ = Self::get_rikiddo(&poolid)?;
            <RikiddoPerPool<T, I>>::remove(poolid);
            Ok(())
        }

        /// Returns the current fee.
        ///
        /// # Arguments
        ///
        /// * `poolid`: The id of the asset pool for which all asset prices shall be calculated.
        /// * `rikiddo`: A specific type of Rikiddo as specified in the pallet's configuration.
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

        /// Returns the price of one specific asset.
        ///
        /// # Arguments
        ///
        /// * `poolid`: The id of the asset pool for which all asset prices shall be calculated.
        /// * `asset_in_question`: The balance of the asset for which the price should be returned.
        /// * `asset_balances`: The balance vector of the assets.
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

        /// Update the market data by adding volume.
        ///
        /// # Arguments
        ///
        /// * `poolid`: The id of the asset pool for which all asset prices shall be calculated.
        /// * `volume`: The volume that was traded in the pool with id `poolid`.
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
                Ok(res) => {
                    if let Some(inner) = res {
                        inner
                    } else {
                        <RikiddoPerPool<T, I>>::insert(poolid, rikiddo);
                        return Ok(None);
                    }
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
