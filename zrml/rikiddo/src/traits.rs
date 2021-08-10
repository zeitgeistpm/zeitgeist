use crate::types::TimestampedVolume;
use frame_support::dispatch::DispatchResult;
use sp_runtime::DispatchError;
use substrate_fixed::traits::{Fixed, FixedUnsigned};

pub trait Sigmoid {
    type FS: Fixed;

    /// Calculate fee
    fn calculate(&self, r: Self::FS) -> Result<Self::FS, &'static str>;
}

pub trait MarketAverage {
    type FU: FixedUnsigned;

    /// Get average (sma, ema, wma, depending on the concrete implementation) of market volume
    fn get(&self) -> Option<Self::FU>;

    /// Clear market data
    fn clear(&mut self);

    /// Update market volume
    fn update(
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
    fn update(
        &mut self,
        volume: &TimestampedVolume<Self::FU>,
    ) -> Result<Option<Self::FU>, &'static str>;

    /// Fetch the current fee
    fn fee(&self) -> Result<Self::FU, &'static str>;
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

    /// Create Rikiddo instance for specifc asset pool
    fn create(poolid: Self::PoolId, rikiddo: Self::Rikiddo) -> DispatchResult;

    /// Clear market data for specific asset pool
    fn clear(poolid: Self::PoolId) -> DispatchResult;

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
    fn update(
        poolid: Self::PoolId,
        volume: Self::Balance,
    ) -> Result<Option<Self::Balance>, DispatchError>;
}
