use crate::types::{
    Asset, MarketType, OutcomeReport, Pool, PoolId, ResultWithWeightInfo, ScoringRule,
};
use alloc::vec::Vec;
use frame_support::dispatch::{DispatchError, Weight};
use parity_scale_codec::MaxEncodedLen;

pub trait Swaps<AccountId> {
    type Balance: MaxEncodedLen;
    type MarketId: MaxEncodedLen;

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

    /// Pool will be marked as `PoolStatus::Active`, if the market is currently in subsidy
    /// state and all other conditions are met. Returns the result of the operation and
    /// the total weight. If the result is false, not enough subsidy was gathered and the
    /// state transition was aborted.
    ///
    /// # Arguments
    ///
    /// * `pool_id`: Unique pool identifier associated with the pool to be made active.
    /// than the given value.
    fn end_subsidy_phase(pool_id: PoolId) -> Result<ResultWithWeightInfo<bool>, DispatchError>;

    /// All supporters will receive their reserved funds back and the pool is destroyed.
    ///
    /// # Arguments
    ///
    /// * `pool_id`: Unique pool identifier associated with the pool to be destroyed.
    fn destroy_pool_in_subsidy_phase(pool_id: PoolId) -> Result<Weight, DispatchError>;

    /// Pool - Exit with exact pool amount
    ///
    /// Takes an asset from `pool_id` and transfers to `origin`. Differently from `pool_exit`,
    /// this method injects the exactly amount of `asset_amount` to `origin`.
    ///
    /// # Arguments
    ///
    /// * `who`: Liquidity Provider (LP). The account whose assets should be received.
    /// * `pool_id`: Unique pool identifier.
    /// * `asset`: Asset leaving the pool.
    /// * `asset_amount`: Asset amount that is leaving the pool.
    /// * `max_pool_amount`: The calculated amount of assets for the pool must be equal or
    /// greater than the given value.
    fn pool_exit_with_exact_asset_amount(
        who: AccountId,
        pool_id: PoolId,
        asset: Asset<Self::MarketId>,
        asset_amount: Self::Balance,
        max_pool_amount: Self::Balance,
    ) -> Result<Weight, DispatchError>;

    /// Pool - Join with exact asset amount
    ///
    /// Joins an asset provided from `origin` to `pool_id`. Differently from `pool_join`,
    /// this method transfers the exactly amount of `asset_amount` to `pool_id`.
    ///
    /// # Arguments
    ///
    /// * `who`: Liquidity Provider (LP). The account whose assets should be received.
    /// * `pool_id`: Unique pool identifier.
    /// * `asset_in`: Asset entering the pool.
    /// * `asset_amount`: Asset amount that is entering the pool.
    /// * `min_pool_amount`: The calculated amount for the pool must be equal or greater
    /// than the given value.
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
    /// that is not ZTG or winning assets from the selected pool. Additionally, it distributes
    /// the rewards to all pool share holders.
    ///
    /// Does nothing if pool is already stale. Returns `Err` if `pool_id` does not exist.
    ///
    /// # Arguments
    ///
    /// * `market_type`: Type of the market (e.g. categorical or scalar).
    /// * `pool_id`: Unique pool identifier associated with the pool to be made stale.
    /// * `outcome_report`: The resulting outcome.
    /// * `winner_payout_account`: The account that exchanges winning assets against rewards.
    fn set_pool_as_stale(
        market_type: &MarketType,
        pool_id: PoolId,
        outcome_report: &OutcomeReport,
        winner_payout_account: &AccountId,
    ) -> Result<Weight, DispatchError>;

    /// Swap - Exact amount in
    ///
    /// Swaps a given `asset_amount_in` of the `asset_in/asset_out` pair to `pool_id`.
    ///
    /// # Arguments
    ///
    /// * `who`: The account whose assets should be transferred.
    /// * `pool_id`: Unique pool identifier.
    /// * `asset_in`: Asset entering the pool.
    /// * `asset_amount_in`: Amount that will be transferred from the provider to the pool.
    /// * `asset_out`: Asset leaving the pool.
    /// * `min_asset_amount_out`: Minimum asset amount that can leave the pool.
    /// * `max_price`: Market price must be equal or less than the provided value.
    fn swap_exact_amount_in(
        who: AccountId,
        pool_id: PoolId,
        asset_in: Asset<Self::MarketId>,
        asset_amount_in: Self::Balance,
        asset_out: Asset<Self::MarketId>,
        min_asset_amount_out: Self::Balance,
        max_price: Self::Balance,
    ) -> Result<Weight, DispatchError>;

    /// Swap - Exact amount out
    ///
    /// Swaps a given `asset_amount_out` of the `asset_in/asset_out` pair to `origin`.
    ///
    /// # Arguments
    ///
    /// * `who`: The account whose assets should be transferred.
    /// * `pool_id`: Unique pool identifier.
    /// * `asset_in`: Asset entering the pool.
    /// * `max_amount_asset_in`: Maximum asset amount that can enter the pool.
    /// * `asset_out`: Asset leaving the pool.
    /// * `asset_amount_out`: Amount that will be transferred from the pool to the provider.
    /// * `max_price`: Market price must be equal or less than the provided value.
    fn swap_exact_amount_out(
        who: AccountId,
        pool_id: PoolId,
        asset_in: Asset<Self::MarketId>,
        max_amount_asset_in: Self::Balance,
        asset_out: Asset<Self::MarketId>,
        asset_amount_out: Self::Balance,
        max_price: Self::Balance,
    ) -> Result<Weight, DispatchError>;
}
