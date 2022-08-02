use sp_runtime::DispatchError;
use zeitgeist_primitives::types::OutcomeReport;

pub trait GlobalDisputesPalletApi<MarketId, Balance> {
    fn push_voting_outcome(
        market_id: &MarketId,
        outcome: OutcomeReport,
        vote_balance: Balance,
    ) -> Result<(), DispatchError>;

    fn get_voting_winner(market_id: &MarketId) -> Option<OutcomeReport>;

    fn is_started(market_id: &MarketId) -> bool;
}
