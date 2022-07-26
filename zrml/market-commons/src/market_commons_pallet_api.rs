#![allow(clippy::type_complexity)]
use frame_support::{
    dispatch::{DispatchError, DispatchResult},
    pallet_prelude::{MaybeSerializeDeserialize, Member},
    storage::PrefixIterator,
    traits::NamedReservableCurrency,
    Parameter,
};
use parity_scale_codec::MaxEncodedLen;
use sp_runtime::traits::AtLeast32Bit;
use zeitgeist_primitives::types::{Market, PoolId};

/// Simple disputes - Pallet Api
pub trait MarketCommonsPalletApi {
    type AccountId;
    type BlockNumber: AtLeast32Bit;
    type Currency: NamedReservableCurrency<Self::AccountId, ReserveIdentifier = [u8; 8]>;
    type MarketId: AtLeast32Bit
        + Copy
        + Default
        + MaybeSerializeDeserialize
        + MaxEncodedLen
        + Member
        + Parameter;
    type Moment: AtLeast32Bit + Copy + Default + Parameter + MaxEncodedLen;

    // Market

    /// Latest attributed auto-generated ID
    ///
    /// Returns `Err` if no market has bees created
    fn latest_market_id() -> Result<Self::MarketId, DispatchError>;

    /// Return an iterator over the key-value pairs of markets. Altering market storage during
    /// iteration results in undefined behavior.
    fn market_iter()
    -> PrefixIterator<(Self::MarketId, Market<Self::AccountId, Self::BlockNumber, Self::Moment>)>;

    /// Gets a market from the storage.
    fn market(
        market_id: &Self::MarketId,
    ) -> Result<Market<Self::AccountId, Self::BlockNumber, Self::Moment>, DispatchError>;

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

    // MarketPool

    /// Connects a pool identified by `pool_id` to a market identified by `market_id`.
    fn insert_market_pool(market_id: Self::MarketId, pool_id: PoolId) -> DispatchResult;

    /// Removes the pool id associated with a given `market_id`
    fn remove_market_pool(market_id: &Self::MarketId) -> DispatchResult;

    /// Fetches the pool id associated with a given `market_id`.
    fn market_pool(market_id: &Self::MarketId) -> Result<PoolId, DispatchError>;

    // Etc

    /// Returns the current UTC time (milliseconds)
    fn now() -> Self::Moment;
}
