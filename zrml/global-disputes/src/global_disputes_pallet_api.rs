use sp_runtime::DispatchError;
use zeitgeist_primitives::types::{OutcomeReport, VoteId};

pub trait GlobalDisputesPalletApi<MarketId, Balance> {
    fn push_voting_outcome(
        id: (&MarketId, &VoteId),
        outcome: OutcomeReport,
        vote_balance: Balance,
    ) -> Result<(), DispatchError>;

    fn get_voting_winner(id: (&MarketId, &VoteId)) -> Option<OutcomeReport>;
}
