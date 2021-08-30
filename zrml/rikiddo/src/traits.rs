use crate::types::TimestampedVolume;
use core::convert::TryFrom;
use frame_support::dispatch::DispatchResult;
use sp_runtime::DispatchError;
use substrate_fixed::{
    traits::{Fixed, FixedUnsigned},
    ParseFixedError,
};

/// Fee calculating structures that take one argument.
pub trait Fee {
    /// A fixed point type.
    type FS: Fixed;

    /// Calculate fee
    fn calculate_fee(&self, r: Self::FS) -> Result<Self::FS, &'static str>;
}

/// Market average implementations, such as EMA, SMA, median, WMA, etc.
pub trait MarketAverage {
    /// An unsigned fixed point type.
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

/// "Logarithmic Market Scoring Rule" (LMSR) specification.
pub trait Lmsr {
    /// An unsigned fixed point type.
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

/// A specification for any implementation of the Rikiddo variant that uses the market volume.
pub trait RikiddoMV: Lmsr {
    /// Clear market data
    fn clear(&mut self);

    /// Update market data
    fn update_volume(
        &mut self,
        volume: &TimestampedVolume<Self::FU>,
    ) -> Result<Option<Self::FU>, &'static str>;
}

/// A specification that a pallet should follow if it wants to offer Rikiddo
/// functionality, that is based on the [`RikiddoMV`](trait@RikiddoMV) trait.
pub trait RikiddoMVPallet {
    /// The type that represents on-chain balances.
    type Balance;
    /// The id of the pool of assets that's associated to one Rikiddo instance.
    type PoolId: Copy;
    /// An unsigned fixed point type.
    type FU: FixedUnsigned;
    /// A type that implements the RikiddoMV trait (LMSR + Rikiddo based on MarketVolume).
    type Rikiddo: RikiddoMV;

    /// Return price P_i(q) for all assets in q
    /// Pool - Exit
    ///
    /// Retrieves all prices for a given set of assets.
    /// Returns a vector of prices (same order as `asset_balances`).
    ///
    /// # Arguments
    ///
    /// * `poolid`: The id of the asset pool for which all asset prices shall be calculated.
    /// * `asset_balances`: The balance vector of the assets.
    fn all_prices(
        poolid: Self::PoolId,
        asset_balances: &[Self::Balance],
    ) -> Result<Vec<Self::Balance>, DispatchError>;

    /// Clear market data for a specific asset pool.
    ///
    /// # Arguments
    ///
    /// * `poolid`: The id of the asset pool for which all asset prices shall be calculated.
    fn clear(poolid: Self::PoolId) -> DispatchResult;

    /// Returns the total cost for a specific vector of assets (see <a href="https://www.eecs.harvard.edu/cs286r/courses/fall12/papers/OPRS10.pdf">LS-LMSR paper</a>).
    ///
    /// # Arguments
    ///
    /// * `poolid`: The id of the asset pool for which all asset prices shall be calculated.
    /// * `asset_balances`: The balance vector of the assets.
    fn cost(
        poolid: Self::PoolId,
        asset_balances: &[Self::Balance],
    ) -> Result<Self::Balance, DispatchError>;

    /// Create Rikiddo instance for specifc asset pool.
    ///
    /// # Arguments
    ///
    /// * `poolid`: The id of the asset pool for which all asset prices shall be calculated.
    /// * `rikiddo`: A specific type of Rikiddo as specified in the pallet's configuration.
    fn create(poolid: Self::PoolId, rikiddo: Self::Rikiddo) -> DispatchResult;

    /// Destroy Rikiddo instance for a specific pool.
    ///
    /// # Arguments
    ///
    /// * `poolid`: The id of the asset pool for which all asset prices shall be calculated.
    fn destroy(poolid: Self::PoolId) -> DispatchResult;

    /// Returns the current fee.
    ///
    /// # Arguments
    ///
    /// * `poolid`: The id of the asset pool for which all asset prices shall be calculated.
    /// * `rikiddo`: A specific type of Rikiddo as specified in the pallet's configuration.
    fn fee(poolid: Self::PoolId) -> Result<Self::Balance, DispatchError>;

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
    ) -> Result<Self::Balance, DispatchError>;

    /// Update the market data by adding volume.
    ///
    /// # Arguments
    ///
    /// * `poolid`: The id of the asset pool for which all asset prices shall be calculated.
    /// * `volume`: The volume that was traded in the pool with id `poolid`.
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
