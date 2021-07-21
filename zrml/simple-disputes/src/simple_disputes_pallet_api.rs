use alloc::vec::Vec;
use frame_support::dispatch::DispatchError;
use zeitgeist_primitives::{
    traits::DisputeApi,
    types::{Market, MarketDispute, ResolutionCounters},
};

/// SimpleDisputes - Pallet Api
pub trait SimpleDisputesPalletApi: DisputeApi {
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

    /// Performs the logic for resolving a market, including slashing and distributing
    /// funds.
    ///
    /// NOTE: This function does not perform any checks on the market that is being given.
    /// In the function calling this you should that the market is already in a reported or
    /// disputed state.
    fn internal_resolve<D>(
        dispute_bound: D,
        market_id: &Self::MarketId,
        market: &Market<Self::AccountId, Self::BlockNumber>,
    ) -> Result<ResolutionCounters, DispatchError>
    where
        D: Fn(usize) -> Self::Balance;

    // Migrations (Temporary)

    fn dispute(
        market_id: &Self::MarketId,
    ) -> Option<Vec<MarketDispute<Self::AccountId, Self::BlockNumber>>>;

    fn insert_dispute(
        market_id: Self::MarketId,
        dispute: Vec<MarketDispute<Self::AccountId, Self::BlockNumber>>,
    );
}
