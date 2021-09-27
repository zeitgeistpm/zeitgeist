use crate::{market::MarketDispute, outcome_report::OutcomeReport, types::Market};
use frame_support::dispatch::DispatchResult;
use sp_runtime::DispatchError;

/// Dispute Api
pub trait DisputeApi {
    type AccountId;
    type Balance;
    type BlockNumber;
    type MarketId;
    type Moment;
    type Origin;

    /// Disputes a reported outcome.
    fn on_dispute(
        disputes: &[MarketDispute<Self::AccountId, Self::BlockNumber>],
        market_id: &Self::MarketId,
        market: &Market<Self::AccountId, Self::BlockNumber, Self::Moment>,
    ) -> DispatchResult;

    /// Manages markets resolutions moving all reported markets to resolved.
    fn on_resolution(
        disputes: &[MarketDispute<Self::AccountId, Self::BlockNumber>],
        market_id: &Self::MarketId,
        market: &Market<Self::AccountId, Self::BlockNumber, Self::Moment>,
    ) -> Result<OutcomeReport, DispatchError>;
}
