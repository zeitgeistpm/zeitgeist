use frame_support::dispatch::DispatchError;
use zeitgeist_primitives::types::Market;

/// Court - Pallet Api
pub trait MarketCommonsPalletApi {
    type AccountId;
    type BlockNumber;
    type MarketId;

    /// Gets a market from the storage.
    fn market(
        market_id: &Self::MarketId,
    ) -> Result<Market<Self::AccountId, Self::BlockNumber>, DispatchError>;

    /// Inserts a market into the storage
    fn insert_market(market_id: Self::MarketId, market: Market<Self::AccountId, Self::BlockNumber>);

    /// Mutates a given market storage
    fn mutate_market<F>(market_id: &Self::MarketId, cb: F) -> Result<(), DispatchError>
    where
        F: FnOnce(&mut Market<Self::AccountId, Self::BlockNumber>);

    /// Removes a market from the storage.
    fn remove_market(market_id: &Self::MarketId) -> Result<(), DispatchError>;
}
