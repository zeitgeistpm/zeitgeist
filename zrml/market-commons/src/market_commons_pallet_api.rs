use alloc::vec::Vec;
use frame_support::{
    dispatch::{DispatchError, DispatchResult},
    pallet_prelude::{MaybeSerializeDeserialize, Member},
    traits::NamedReservableCurrency,
    Parameter,
};
use parity_scale_codec::MaxEncodedLen;
use sp_runtime::traits::AtLeast32Bit;
use zeitgeist_primitives::types::{Market, PoolId, Report};

/// Simple disputes - Pallet Api
pub trait MarketCommonsPalletApi {
    type AccountId;
    type BlockNumber: AtLeast32Bit;
    type Currency: NamedReservableCurrency<Self::AccountId, ReserveIdentifier = [u8; 8]>;
    type MarketId: AtLeast32Bit + Copy + Default + MaybeSerializeDeserialize + MaxEncodedLen + Member + Parameter;
    type Moment: AtLeast32Bit + Copy + Default + Parameter + MaxEncodedLen;

    // Market

    /// Latest attributed auto-generated ID
    ///
    /// Returns `Err` if no market has bees created
    fn latest_market_id() -> Result<Self::MarketId, DispatchError>;

    /// Gets a market from the storage.
    fn market(
        market_id: &Self::MarketId,
    ) -> Result<Market<Self::AccountId, Self::BlockNumber, Self::Moment>, DispatchError>;

    /// All stored markets
    fn markets() -> Vec<(Self::MarketId, Market<Self::AccountId, Self::BlockNumber, Self::Moment>)>;

    /// Mutates a given market storage
    fn mutate_market<F>(market_id: &Self::MarketId, cb: F) -> DispatchResult
    where
        F: FnOnce(&mut Market<Self::AccountId, Self::BlockNumber, Self::Moment>) -> DispatchResult;

    /// Pushes a new market into the storage, returning its related auto-incremented ID.
    fn push_market(
        market: Market<Self::AccountId, Self::BlockNumber, Self::Moment>,
    ) -> Result<Self::MarketId, DispatchError>;

    /// Removes a market from the storage.
    fn remove_market(market_id: &Self::MarketId) -> DispatchResult;

    /// If any, returns all information regarding the account that reported an outcome.
    fn report(
        market: &Market<Self::AccountId, Self::BlockNumber, Self::Moment>,
    ) -> Result<&Report<Self::AccountId, Self::BlockNumber>, DispatchError>;

    // MarketPool

    /// Connects a pool identified by `pool_id` to a market identified by `market_id`.
    fn insert_market_pool(market_id: Self::MarketId, pool_id: PoolId);

    /// Removes the pool id associated with a given `market_id`
    fn remove_market_pool(market_id: &Self::MarketId) -> DispatchResult;

    /// Fetches the pool id associated with a given `market_id`.
    fn market_pool(market_id: &Self::MarketId) -> Result<PoolId, DispatchError>;

    // Etc

    /// Returns the current UTC time (milliseconds)
    fn now() -> Self::Moment;
}
