use crate::{market::MarketDispute, outcome_report::OutcomeReport, types::Market};
use frame_support::dispatch::DispatchResult;
use sp_runtime::DispatchError;

pub trait DisputeApi {
    type AccountId;
    type Balance;
    type BlockNumber;
    type MarketId;
    type Moment;
    type Origin;

    /// Dispute a reported outcome.
    ///
    /// # Arguments
    ///
    /// * `disputes` - The disputes of `market`
    /// * `market_id` - The id of `market`
    /// * `market` - The market to dispute
    ///
    /// # Errors
    ///
    /// Returns an error if the market's dispute mechanism is not as expected.
    fn on_dispute(
        disputes: &[MarketDispute<Self::AccountId, Self::BlockNumber>],
        market_id: &Self::MarketId,
        market: &Market<Self::AccountId, Self::BlockNumber, Self::Moment>,
    ) -> DispatchResult;

    /// Manage market resolution of disputed market.
    ///
    /// Should only be called if the market was disputed before resolving.
    ///
    /// # Arguments
    ///
    /// * `disputes` - The disputes of `market`
    /// * `market_id` - The id of `market`
    /// * `market` - The market to dispute
    ///
    /// # Errors
    ///
    /// Returns an error if the market's dispute mechanism is not as expected.
    fn on_resolution(
        disputes: &[MarketDispute<Self::AccountId, Self::BlockNumber>],
        market_id: &Self::MarketId,
        market: &Market<Self::AccountId, Self::BlockNumber, Self::Moment>,
    ) -> Result<OutcomeReport, DispatchError>;
}
