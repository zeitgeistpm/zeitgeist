use core::convert::TryFrom;
use crate::types::TimestampedVolume;
use frame_support::dispatch::DispatchResult;
use sp_runtime::DispatchError;
use substrate_fixed::{
    traits::{Fixed, FixedUnsigned},
    ParseFixedError,
};

pub trait Sigmoid {
    type FS: Fixed;

    /// Calculate fee
    fn calculate_fee(&self, r: Self::FS) -> Result<Self::FS, &'static str>;
}

pub trait MarketAverage {
    type FU: FixedUnsigned;

    /// Get average (sma, ema, wma, depending on the concrete implementation) of market volume
    fn get(&self) -> Option<Self::FU>;

    /// Clear market data
    fn clear(&mut self);

    /// Update market volume
    fn update_volume(
        &mut self,
        volume: &TimestampedVolume<Self::FU>,
    ) -> Result<Option<Self::FU>, &'static str>;
}

pub trait Lmsr {
    type FU: FixedUnsigned;

    /// Return price P_i(q) for all assets in q
    fn all_prices(&self, asset_balances: &[Self::FU]) -> Result<Vec<Self::FU>, &'static str>;

    /// Return cost C(q) for all assets in q
    fn cost(&self, asset_balances: &[Self::FU]) -> Result<Self::FU, &'static str>;

    /// Fetch the current fee
    fn fee(&self) -> Result<Self::FU, &'static str>;

    /// Return price P_i(q) for asset q_i in q
    fn price(
        &self,
        asset_balances: &[Self::FU],
        asset_in_question_balance: &Self::FU,
    ) -> Result<Self::FU, &'static str>;
}

pub trait RikiddoMV: Lmsr {
    /// Clear market data
    fn clear(&mut self);

    /// Update market data
    fn update_volume(
        &mut self,
        volume: &TimestampedVolume<Self::FU>,
    ) -> Result<Option<Self::FU>, &'static str>;
}

pub trait RikiddoSigmoidMVPallet {
    type Balance;
    type PoolId: Copy;
    type FU: FixedUnsigned;
    type Rikiddo: RikiddoMV;

    /// Return price P_i(q) for all assets in q
    fn all_prices(
        poolid: Self::PoolId,
        asset_balances: &[Self::Balance],
    ) -> Result<Vec<Self::Balance>, DispatchError>;

    /// Clear market data for specific asset pool
    fn clear(poolid: Self::PoolId) -> DispatchResult;

    /// Create Rikiddo instance for specifc asset pool
    fn create(poolid: Self::PoolId, rikiddo: Self::Rikiddo) -> DispatchResult;

    /// Return cost C(q) for all assets in q
    fn cost(
        poolid: Self::PoolId,
        asset_balances: &[Self::Balance],
    ) -> Result<Self::Balance, DispatchError>;

    /// Destroy Rikiddo instance
    fn destroy(poolid: Self::PoolId) -> DispatchResult;

    /// Fetch the current fee
    fn fee(poolid: Self::PoolId) -> Result<Self::Balance, DispatchError>;

    /// Return price P_i(q) for asset q_i in q
    fn price(
        poolid: Self::PoolId,
        asset_in_question: Self::Balance,
        asset_balances: &[Self::Balance],
    ) -> Result<Self::Balance, DispatchError>;

    /// Update market data
    fn update_volume(
        poolid: Self::PoolId,
        volume: Self::Balance,
    ) -> Result<Option<Self::Balance>, DispatchError>;
}

/// Converts a fixed point decimal number into another type
pub trait FromFixedDecimal<N: Into<u128>>
where
    Self: Sized,
{
    fn from_fixed_decimal(decimal: N, places: u8) -> Result<Self, ParseFixedError>;
}

/// Converts a fixed point decimal number into Fixed type (Balance -> Fixed)
pub trait IntoFixedFromDecimal<F: Fixed> {
    fn to_fixed_from_fixed_decimal(self, places: u8) -> Result<F, ParseFixedError>;
}

/// Converts a Fixed type into fixed point decimal number
pub trait FromFixedToDecimal<F>
where
    Self: Sized + TryFrom<u128>,
{
    fn from_fixed_to_fixed_decimal(fixed: F, places: u8) -> Result<Self, &'static str>;
}

/// Converts a fixed point decimal number into Fixed type
pub trait IntoFixedDecimal<N: TryFrom<u128>> {
    fn to_fixed_decimal(self, places: u8) -> Result<N, &'static str>;
}
