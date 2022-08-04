use zeitgeist_primitives::types::OutcomeReport;

pub trait GlobalDisputesPalletApi<MarketId, AccountId, Balance> {
    fn push_voting_outcome(
        market_id: &MarketId,
        outcome: OutcomeReport,
        owner: &AccountId,
        vote_balance: Balance,
    );

    fn get_voting_winner(market_id: &MarketId) -> Option<OutcomeReport>;

    fn is_started(market_id: &MarketId) -> bool;
}
