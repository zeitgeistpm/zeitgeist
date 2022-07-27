use crate::*;
use sp_runtime::DispatchError;
use zeitgeist_primitives::types::OutcomeReport;

pub trait GlobalDisputesPalletApi {
    type Balance;

    fn get_latest_vote_id() -> VoteId;

    fn get_next_vote_id() -> Result<VoteId, DispatchError>;

    fn push_voting_outcome(
        outcome: OutcomeReport,
        vote_balance: Self::Balance,
    ) -> Result<(), DispatchError>;

    fn get_voting_winner(vote_id: VoteId) -> Result<OutcomeReport, DispatchError>;
}
