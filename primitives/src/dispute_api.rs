use crate::types::{Market, OutcomeReport, ResolutionCounters};
use frame_support::dispatch::{DispatchError, DispatchResult};

/// Dispute Api
pub trait DisputeApi {
    type AccountId;
    type BlockNumber;
    type MarketId;
    type Origin;

    /// Disputes a reported outcome.
    fn on_dispute(
        origin: Self::Origin,
        market_id: Self::MarketId,
        outcome: OutcomeReport,
    ) -> Result<[u32; 2], DispatchError>;

    /// Manages markets resolutions moving all reported markets to resolved.
    fn on_resolution<F>(now: Self::BlockNumber, cb: F) -> DispatchResult
    where
        F: FnMut(&Market<Self::AccountId, Self::BlockNumber>, ResolutionCounters);
}
