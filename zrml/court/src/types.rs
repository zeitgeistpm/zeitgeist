// Copyright 2022-2023 Forecasting Technologies LTD.
// Copyright 2021-2022 Zeitgeist PM LLC.
//
// This file is part of Zeitgeist.
//
// Zeitgeist is free software: you can redistribute it and/or modify it
// under the terms of the GNU General Public License as published by the
// Free Software Foundation, either version 3 of the License, or (at
// your option) any later version.
//
// Zeitgeist is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Zeitgeist. If not, see <https://www.gnu.org/licenses/>.
extern crate alloc;
use alloc::{vec, vec::Vec};
use zeitgeist_primitives::types::OutcomeReport;

/// The type of the court identifier.
pub type CourtId = u128;

/// The different court vote types. This can be extended to allow different decision making options.
#[derive(
    parity_scale_codec::Decode,
    parity_scale_codec::Encode,
    parity_scale_codec::MaxEncodedLen,
    scale_info::TypeInfo,
    Clone,
    Debug,
    PartialEq,
    Eq,
)]
pub enum VoteItemType {
    Outcome,
    Binary,
}

/// The different court vote types with their raw values.
/// This can be extended to allow different decision making options.
#[derive(
    parity_scale_codec::Decode,
    parity_scale_codec::Encode,
    parity_scale_codec::MaxEncodedLen,
    scale_info::TypeInfo,
    Clone,
    Debug,
    PartialEq,
    Eq,
    Ord,
    PartialOrd,
)]
pub enum VoteItem {
    Outcome(OutcomeReport),
    Binary(bool),
}

/// Simple implementations to handle vote items easily.
impl VoteItem {
    pub fn into_outcome(self) -> Option<OutcomeReport> {
        match self {
            Self::Outcome(report) => Some(report),
            _ => None,
        }
    }
}

/// The general information about a particular court participant (juror or delegator).
#[derive(
    parity_scale_codec::Decode,
    parity_scale_codec::Encode,
    parity_scale_codec::MaxEncodedLen,
    scale_info::TypeInfo,
    Clone,
    Debug,
    PartialEq,
    Eq,
)]
pub struct CourtParticipantInfo<Balance, BlockNumber, Delegations> {
    /// The court participants amount in the stake weighted pool.
    /// This amount is used to find a court participant with a binary search on the pool.
    pub(crate) stake: Balance,
    /// The current amount of funds which are locked in courts.
    pub(crate) active_lock: Balance,
    /// The block number when an exit from court was requested.
    pub(crate) prepare_exit_at: Option<BlockNumber>,
    /// The delegations of the court participant. This determines the account as a delegator.
    pub(crate) delegations: Option<Delegations>,
}

/// The raw information behind the secret hash of a juror's vote.
pub(crate) struct RawCommitment<AccountId, Hash> {
    /// The juror's account id.
    pub(crate) juror: AccountId,
    /// The vote item which the juror voted for.
    pub(crate) vote_item: VoteItem,
    /// The salt which was used to hash the vote.
    pub(crate) salt: Hash,
}

/// All possible states of a vote.
#[derive(
    parity_scale_codec::Decode,
    parity_scale_codec::Encode,
    parity_scale_codec::MaxEncodedLen,
    scale_info::TypeInfo,
    Clone,
    Debug,
    PartialEq,
    Eq,
)]
pub enum Vote<Hash, DelegatedStakes> {
    /// The delegator delegated stake to other jurors.
    Delegated { delegated_stakes: DelegatedStakes },
    /// The juror was randomly selected to vote in a specific court case.
    Drawn,
    /// The juror casted a vote, only providing a hash, which meaning is unknown.
    Secret { commitment: Hash },
    /// The juror revealed her raw vote, letting anyone know what she voted.
    Revealed { commitment: Hash, vote_item: VoteItem, salt: Hash },
    /// The juror was denounced, because she revealed her raw vote during the vote phase.
    Denounced { commitment: Hash, vote_item: VoteItem, salt: Hash },
}

/// The status of a court case.
#[derive(
    parity_scale_codec::Decode,
    parity_scale_codec::Encode,
    parity_scale_codec::MaxEncodedLen,
    scale_info::TypeInfo,
    Clone,
    Debug,
    PartialEq,
    Eq,
)]
pub enum CourtStatus {
    /// The court case has been started.
    Open,
    /// The court case was closed, the winner vote item was determined.
    Closed { winner: VoteItem },
    /// The juror stakes from the court were reassigned
    Reassigned,
}

/// The information about an appeal for a court case.
#[derive(
    parity_scale_codec::Decode,
    parity_scale_codec::Encode,
    parity_scale_codec::MaxEncodedLen,
    scale_info::TypeInfo,
    Clone,
    Debug,
    PartialEq,
    Eq,
)]
pub struct AppealInfo<AccountId, Balance> {
    /// The account which made the appeal.
    pub(crate) backer: AccountId,
    /// The amount of funds which were locked for the appeal.
    pub(crate) bond: Balance,
    /// The vote item which was appealed.
    pub(crate) appealed_vote_item: VoteItem,
}

/// The timing information about a court round.
#[derive(
    parity_scale_codec::Decode,
    parity_scale_codec::Encode,
    parity_scale_codec::MaxEncodedLen,
    scale_info::TypeInfo,
    Clone,
    Debug,
    PartialEq,
    Eq,
)]
pub struct RoundTiming<BlockNumber> {
    /// The end block of the pre-vote period.
    pub pre_vote: BlockNumber,
    /// The end block of the vote period.
    /// Note this can also be used as the block duration for votes,
    /// if it is used for the initialisation of the court round ends.
    pub vote: BlockNumber,
    /// The end block of the aggregation period.
    /// Note this can also be used as the block duration for revealing votes,
    /// if it is used for the initialisation of the court round ends.
    pub aggregation: BlockNumber,
    /// The end block of the appeal period.
    /// Note this can also be used as the block duration for appeals,
    /// if it is used for the initialisation of the court round ends.
    pub appeal: BlockNumber,
}

/// The information about a court case.
#[derive(
    parity_scale_codec::Decode,
    parity_scale_codec::Encode,
    parity_scale_codec::MaxEncodedLen,
    scale_info::TypeInfo,
    Clone,
    Debug,
    PartialEq,
    Eq,
)]
pub struct CourtInfo<BlockNumber, Appeals> {
    /// The status of the court case.
    pub status: CourtStatus,
    /// The list of all appeals.
    pub appeals: Appeals,
    /// This specifies the end blocks of the court case phases.
    pub round_ends: RoundTiming<BlockNumber>,
    /// The type of the vote item.
    pub vote_item_type: VoteItemType,
}

impl<BlockNumber: sp_runtime::traits::Saturating + Copy, Appeals: Default>
    CourtInfo<BlockNumber, Appeals>
{
    pub(crate) fn new(
        round_timing: RoundTiming<BlockNumber>,
        vote_item_type: VoteItemType,
    ) -> Self {
        let pre_vote = round_timing.pre_vote;
        let vote = pre_vote.saturating_add(round_timing.vote);
        let aggregation = vote.saturating_add(round_timing.aggregation);
        let appeal = aggregation.saturating_add(round_timing.appeal);
        let round_ends = RoundTiming { pre_vote, vote, aggregation, appeal };
        let status = CourtStatus::Open;
        Self { status, appeals: Default::default(), round_ends, vote_item_type }
    }

    pub(crate) fn update_round(&mut self, round_timing: RoundTiming<BlockNumber>) {
        self.round_ends.pre_vote = round_timing.pre_vote;
        self.round_ends.vote = self.round_ends.pre_vote.saturating_add(round_timing.vote);
        self.round_ends.aggregation = self.round_ends.vote.saturating_add(round_timing.aggregation);
        self.round_ends.appeal = self.round_ends.aggregation.saturating_add(round_timing.appeal);
    }
}

/// After a court participant was randomly selected to vote in a court case,
/// this information is relevant to handle the post-selection process.
#[derive(
    parity_scale_codec::Decode,
    parity_scale_codec::Encode,
    parity_scale_codec::MaxEncodedLen,
    scale_info::TypeInfo,
    Clone,
    Debug,
    PartialEq,
    Eq,
)]
pub struct Draw<AccountId, Balance, Hash, DelegatedStakes> {
    /// The court participant who was randomly selected.
    pub court_participant: AccountId,
    /// The weight of the juror in this court case.
    /// The higher the weight the more voice the juror has in the final winner decision.
    pub weight: u32,
    /// The information about the vote state.
    pub vote: Vote<Hash, DelegatedStakes>,
    /// The amount of funds which can be slashed for this court case.
    /// This is equal to a multiple of `MinJurorStake` to mitigate Sybil attacks.
    pub slashable: Balance,
}

/// All information related to one item in the stake weighted juror pool.
#[derive(
    parity_scale_codec::Decode,
    parity_scale_codec::Encode,
    parity_scale_codec::MaxEncodedLen,
    scale_info::TypeInfo,
    Clone,
    Debug,
    PartialEq,
    Eq,
)]
pub struct CourtPoolItem<AccountId, Balance, BlockNumber> {
    /// The amount of funds associated to a court participant
    /// in order to get selected for a court case.
    pub stake: Balance,
    /// The account which is the juror that might be selected in court cases.
    pub court_participant: AccountId,
    /// The consumed amount of the stake for all draws. This is useful to reduce the probability
    /// of a court participant to be selected again.
    pub consumed_stake: Balance,
    /// The block number at which the participant joined.
    pub joined_at: BlockNumber,
}

/// The information about an internal selected draw of a juror or delegator.
#[derive(
    parity_scale_codec::Decode,
    parity_scale_codec::Encode,
    parity_scale_codec::MaxEncodedLen,
    scale_info::TypeInfo,
    Clone,
    Debug,
    PartialEq,
    Eq,
)]
pub(crate) struct SelectionValue<Balance, DelegatedStakes> {
    /// The overall weight of the juror or delegator for a specific selected draw.
    pub(crate) weight: u32,
    /// The amount that can be slashed for this selected draw.
    pub(crate) slashable: Balance,
    /// The different portions of stake distributed over multiple jurors.
    /// The sum of all delegated stakes should be equal to `slashable`.
    pub(crate) delegated_stakes: DelegatedStakes,
}

/// The type to add one weight to the selected draws.
#[derive(
    parity_scale_codec::Decode,
    parity_scale_codec::Encode,
    parity_scale_codec::MaxEncodedLen,
    scale_info::TypeInfo,
    Clone,
    Debug,
    PartialEq,
    Eq,
)]
pub(crate) enum SelectionAdd<AccountId, Balance> {
    /// The variant to add an active juror, who is not a delegator.
    SelfStake { lock: Balance },
    /// The variant to decide that a delegator is added
    /// to the selected draws and locks stake on a delegated juror.
    DelegationStake { delegated_juror: AccountId, lock: Balance },
    /// The variant to know that one weight for the delegation to the delegated juror is added.
    DelegationWeight,
}

/// The information about an active juror who voted for a court.
pub(crate) struct SelfInfo<Balance> {
    /// The slashable amount of the juror herself.
    pub(crate) slashable: Balance,
    /// The item for which the juror voted.
    pub(crate) vote_item: VoteItem,
}

pub(crate) struct JurorVoteWithStakes<AccountId, Balance> {
    /// An optional information about an active juror herself, who was selected and voted.
    /// This could be None, because delegators could have delegated to a juror who failed to vote.
    pub(crate) self_info: Option<SelfInfo<Balance>>,
    // many delegators can have delegated to the same juror
    // that's why the value is a vector and should be sorted (binary search by key)
    // the key is the delegator account
    // the value is the delegated stake
    pub(crate) delegations: Vec<(AccountId, Balance)>,
}

impl<AccountId, Balance> Default for JurorVoteWithStakes<AccountId, Balance> {
    fn default() -> Self {
        JurorVoteWithStakes { self_info: None, delegations: vec![] }
    }
}

/// An internal error type to determine how the selection of draws fails.
pub(crate) enum SelectionError {
    NoValidDelegatedJuror,
}
