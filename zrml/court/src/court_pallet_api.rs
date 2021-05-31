use frame_support::dispatch::{DispatchError, DispatchResultWithPostInfo, Weight};
use zeitgeist_primitives::types::OutcomeReport;

/// Court - Pallet Api
pub trait CourtPalletApi {
    type BlockNumber;
    type MarketId;
    type Origin;

    /// Disputes a reported outcome.
    fn on_dispute(
        origin: Self::Origin,
        market_id: Self::MarketId,
        outcome: OutcomeReport,
    ) -> DispatchResultWithPostInfo;

    /// Manages markets resolutions moving all reported markets to resolved.
    fn on_resolution(now: Self::BlockNumber) -> Result<Weight, DispatchError>;
}
