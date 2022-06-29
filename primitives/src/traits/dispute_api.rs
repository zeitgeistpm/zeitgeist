use crate::{market::MarketDispute, outcome_report::OutcomeReport, types::Market};
use frame_support::{dispatch::DispatchResult, storage::PrefixIterator};
use sp_runtime::DispatchError;

pub trait DisputeApi {
    type AccountId;
    type Balance;
    type BlockNumber;
    type MarketId;
    type Moment;
    type Origin;

    /// Initiate a dispute of a reported outcome.
    ///
    /// Further interaction with the dispute API (if necessary) **should** happen through an
    /// associated pallet. **May** assume that `market.mdm` refers to the calling dispute API.
    fn on_dispute(
        previous_disputes: &[MarketDispute<Self::AccountId, Self::BlockNumber>],
        market_id: &Self::MarketId,
        market: &Market<Self::AccountId, Self::BlockNumber, Self::Moment>,
    ) -> DispatchResult;

    /// Manage market resolution of a disputed market.
    ///
    /// **Should** only be called if the market was disputed before resolving. **May** assume that
    /// `market.mdm` refers to the calling dispute API.
    ///
    /// # Returns
    ///
    /// Returns the dispute mechanism's report if available, otherwise `None`. If `None` is
    /// returned, this means that the dispute could not be resolved.
    fn on_resolution(
        disputes: &[MarketDispute<Self::AccountId, Self::BlockNumber>],
        market_id: &Self::MarketId,
        market: &Market<Self::AccountId, Self::BlockNumber, Self::Moment>,
    ) -> Result<Option<OutcomeReport>, DispatchError>;

    fn on_global_dispute_resolution(
        disputes: &[MarketDispute<Self::AccountId, Self::BlockNumber>],
        dispute_votes: PrefixIterator<(u32, Self::Balance)>,
        market_id: &Self::MarketId,
        market: &Market<Self::AccountId, Self::BlockNumber, Self::Moment>,
    ) -> Result<Option<OutcomeReport>, DispatchError>;
}
