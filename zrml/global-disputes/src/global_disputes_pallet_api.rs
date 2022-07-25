use sp_runtime::DispatchError;
use zeitgeist_primitives::types::OutcomeReport;

pub trait GlobalDisputesPalletApi {
    type Balance;
    type MarketId;

    fn push_voting_outcome(
        market_id: &Self::MarketId,
        outcome: OutcomeReport,
        vote_balance: Self::Balance,
    ) -> Result<(), DispatchError>;

    fn get_voting_winner(market_id: &Self::MarketId) -> Result<OutcomeReport, DispatchError>;
}
