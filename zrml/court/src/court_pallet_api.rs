use crate::ResolutionCounters;
use alloc::vec::Vec;
use frame_support::dispatch::{DispatchError, DispatchResultWithPostInfo};
use zeitgeist_primitives::types::{Market, MarketDispute, OutcomeReport};

/// Court - Pallet Api
pub trait CourtPalletApi {
    type AccountId;
    type BlockNumber;
    type MarketId;
    type Origin;

    // Market

    /// Gets a market from the storage.
    fn market(
        market_id: &Self::MarketId,
    ) -> Result<Market<Self::AccountId, Self::BlockNumber>, DispatchError>;

    /// Inserts a market into the storage
    fn insert_market(
        market_id: &Self::MarketId,
        market: Market<Self::AccountId, Self::BlockNumber>,
    );

    /// Mutates a given market storage
    fn mutate_market<F>(market_id: &Self::MarketId, cb: F) -> Result<(), DispatchError>
    where
        F: FnOnce(&mut Market<Self::AccountId, Self::BlockNumber>);

    /// Removes a market from the storage.
    fn remove_market(market_id: &Self::MarketId) -> Result<(), DispatchError>;

    // MarketIdPerDisputeBlock

    /// Inserts a disputed market ids of a block into the storage
    fn insert_market_id_per_dispute_block(
        block: Self::BlockNumber,
        market_ids: Vec<Self::MarketId>,
    );

    /// Gets all disputed market ids of a block from the storage.
    fn market_ids_per_dispute_block(
        block: &Self::BlockNumber,
    ) -> Result<Vec<Self::MarketId>, DispatchError>;

    // MarketIdPerReportBlock

    /// Inserts a reported market ids of a block into the storage
    fn insert_market_id_per_report_block(block: Self::BlockNumber, market_ids: Vec<Self::MarketId>);

    /// Gets all reported market ids of a block from the storage.
    fn market_ids_per_report_block(
        block: &Self::BlockNumber,
    ) -> Result<Vec<Self::MarketId>, DispatchError>;

    /// Mutates a given set of reported market ids
    fn mutate_market_ids_per_report_block<F>(
        block: &Self::BlockNumber,
        cb: F,
    ) -> Result<(), DispatchError>
    where
        F: FnOnce(&mut Vec<Self::MarketId>);

    // Misc

    /// The number of stored disputes
    fn disputes(
        market_id: &Self::MarketId,
    ) -> Result<Vec<MarketDispute<Self::AccountId, Self::BlockNumber>>, DispatchError>;

    /// The stored disputing period
    fn dispute_period() -> Self::BlockNumber;

    /// The stored maximum number of disputes
    fn max_disputes() -> u32;

    /// Disputes a reported outcome.
    fn on_dispute(
        origin: Self::Origin,
        market_id: Self::MarketId,
        outcome: OutcomeReport,
    ) -> DispatchResultWithPostInfo;

    /// Manages markets resolutions moving all reported markets to resolved.
    fn on_resolution(now: Self::BlockNumber) -> Result<ResolutionCounters, DispatchError>;
}
