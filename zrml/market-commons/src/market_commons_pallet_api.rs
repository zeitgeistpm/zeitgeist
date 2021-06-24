use frame_support::{
    dispatch::{DispatchError, DispatchResult},
    pallet_prelude::{MaybeSerializeDeserialize, Member},
    Parameter,
};
use sp_runtime::traits::AtLeast32Bit;
use zeitgeist_primitives::types::{Market, PoolId};

/// Simple disputes - Pallet Api
pub trait MarketCommonsPalletApi {
    type AccountId;
    type BlockNumber;
    type MarketId: AtLeast32Bit + Copy + Default + MaybeSerializeDeserialize + Member + Parameter;

    // Market

    /// Latest attributed auto-generated ID
    ///
    /// Returns `Err` if no market has bees created
    fn latest_market_id() -> Result<Self::MarketId, DispatchError>;

    /// Gets a market from the storage.
    fn market(
        market_id: &Self::MarketId,
    ) -> Result<Market<Self::AccountId, Self::BlockNumber>, DispatchError>;

    /// Mutates a given market storage
    fn mutate_market<F>(market_id: &Self::MarketId, cb: F) -> DispatchResult
    where
        F: FnOnce(&mut Market<Self::AccountId, Self::BlockNumber>) -> DispatchResult;

    /// Pushes a new market into the storage, returning its related auto-incremented ID.
    fn push_market(
        market: Market<Self::AccountId, Self::BlockNumber>,
    ) -> Result<Self::MarketId, DispatchError>;

    /// Removes a market from the storage.
    fn remove_market(market_id: &Self::MarketId) -> DispatchResult;

    // MarketPool

    /// Connects a pool identified by `pool_id` to a market identified by `market_id`.
    fn insert_market_pool(market_id: Self::MarketId, pool_id: PoolId);

    /// Fetches the pool id associated with a given `market_id`.
    fn market_pool(market_id: &Self::MarketId) -> Result<PoolId, DispatchError>;

    // Migrations (Temporary)

    fn insert_market(market_id: Self::MarketId, market: Market<Self::AccountId, Self::BlockNumber>);

    fn set_market_counter(counter: Self::MarketId);
}
