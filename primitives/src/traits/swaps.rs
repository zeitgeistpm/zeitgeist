use crate::types::{Asset, MarketType, OutcomeReport, Pool, PoolId, ScoringRule};
use alloc::vec::Vec;
use frame_support::dispatch::{DispatchError, DispatchResult, Weight};

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
    /// * `base_asset`: The base asset in a prediction market swap pool (usually a currency).
    /// * `market_id`: The market id of the market the pool belongs to.
    /// * `scoring_rule`: The scoring rule that's used to determine the asset prices.
    /// * `swap_fee`: The fee applied to each swap (in case the scoring rule doesn't provide fees).
    /// * `weights`: These are the denormalized weights (the raw weights).
    fn create_pool(
        creator: AccountId,
        assets: Vec<Asset<Self::MarketId>>,
        base_asset: Option<Asset<Self::MarketId>>,
        market_id: Self::MarketId,
        scoring_rule: ScoringRule,
        swap_fee: Option<Self::Balance>,
        weights: Option<Vec<u128>>,
    ) -> Result<PoolId, DispatchError>;

    fn pool_exit_with_exact_asset_amount(
        who: AccountId,
        pool_id: PoolId,
        asset: Asset<Self::MarketId>,
        asset_amount: Self::Balance,
        max_pool_amount: Self::Balance,
    ) -> Result<Weight, DispatchError>;

    fn pool_join_with_exact_asset_amount(
        who: AccountId,
        pool_id: PoolId,
        asset_in: Asset<Self::MarketId>,
        asset_amount: Self::Balance,
        min_pool_amount: Self::Balance,
    ) -> Result<Weight, DispatchError>;

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
    ) -> DispatchResult;
}
