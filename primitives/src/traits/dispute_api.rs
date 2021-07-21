use crate::types::{Market, OutcomeReport, ResolutionCounters};
use frame_support::dispatch::{DispatchError, DispatchResult};

/// Dispute Api
pub trait DisputeApi {
    type AccountId;
    type Balance;
    type BlockNumber;
    type MarketId;
    type Origin;

    /// Disputes a reported outcome.
    fn on_dispute<D>(
        dispute_bond: D,
        market_id: Self::MarketId,
        outcome: OutcomeReport,
        who: Self::AccountId,
    ) -> Result<(), DispatchError>
    where
        D: Fn(usize) -> Self::Balance;

    /// Manages markets resolutions moving all reported markets to resolved.
    fn on_resolution<D, F>(dispute_bound: D, now: Self::BlockNumber, cb: F) -> DispatchResult
    where
        D: Clone + Fn(usize) -> Self::Balance,
        F: FnMut(&Market<Self::AccountId, Self::BlockNumber>, ResolutionCounters);
}
