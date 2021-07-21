use crate::types::{Asset, MarketType, OutcomeReport, Pool, PoolId};
use alloc::vec::Vec;
use frame_support::dispatch::DispatchError;

pub trait Swaps<AccountId> {
    type Balance;
    type MarketId;

    /// Creates an initial active pool.
    ///
    /// # Arguments
    ///
    /// * `who`: The account that is the creator of the pool. Must have enough
    /// funds for each of the assets to cover the `MinLiqudity`.
    /// * `assets`: The assets that are used in the pool.
    /// * `swap_fee`: The fee applied to each swap.
    /// * `weights`: These are the denormalized weights (the raw weights).
    fn create_pool(
        creator: AccountId,
        assets: Vec<Asset<Self::MarketId>>,
        market_id: Self::MarketId,
        swap_fee: Self::Balance,
        weights: Vec<u128>,
    ) -> Result<PoolId, DispatchError>;

    /// Returns the pool instance of a corresponding `pool_id`.
    fn pool(pool_id: PoolId) -> Result<Pool<Self::Balance, Self::MarketId>, DispatchError>;

    /// Pool will be marked as `PoolStatus::Stale`. If market is categorical, removes everything
    /// that is not ZTG or winning assets from the selected pool.
    ///
    /// Does nothing if pool is already stale. Returns `Err` if `pool_id` does not exist.
    fn set_pool_as_stale(
        market_type: &MarketType,
        pool_id: PoolId,
        outcome_report: &OutcomeReport,
    ) -> Result<(), DispatchError>;
}
