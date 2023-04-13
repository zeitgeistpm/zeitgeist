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
use alloc::vec::Vec;
use zeitgeist_primitives::types::OutcomeReport;
/// The general information about a particular juror.
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
pub struct JurorInfo<Balance, BlockNumber, Delegations> {
    /// The juror's amount in the stake weighted pool.
    /// This amount is used to find a juror with a binary search on the pool.
    pub stake: Balance,
    /// The current amount of funds which are locked in courts.
    pub active_lock: Balance,
    /// The block number when a juror exit from court was requested.
    pub prepare_exit_at: Option<BlockNumber>,
    pub delegations: Delegations,
}

/// The raw information behind the secret hash of a juror's vote.
pub struct RawCommitment<AccountId, Hash> {
    /// The juror's account id.
    pub juror: AccountId,
    /// The outcome which the juror voted for.
    pub outcome: OutcomeReport,
    /// The salt which was used to hash the vote.
    pub salt: Hash,
}

/// The raw information which is hashed to create the secret hash of a juror's vote.
pub struct CommitmentMatcher<AccountId, Hash> {
    /// The juror's hashed commitment
    pub hashed: Hash,
    /// The raw commitment which is intended to lead to the hashed commitment.
    pub raw: RawCommitment<AccountId, Hash>,
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
    /// The juror delegated stake to other jurors.
    Delegated { delegated_stakes: DelegatedStakes },
    /// The juror was randomly selected to vote in a specific court case.
    Drawn,
    /// The juror casted a vote, only providing a hash, which meaning is unknown.
    Secret { commitment: Hash },
    /// The juror revealed her raw vote, letting anyone know what she voted.
    Revealed { commitment: Hash, outcome: OutcomeReport, salt: Hash },
    /// The juror was denounced, because she revealed her raw vote during the vote phase.
    Denounced { commitment: Hash, outcome: OutcomeReport, salt: Hash },
}

/// The information about the lifecycle of a court case.
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
pub struct CycleEnds<BlockNumber> {
    /// The end block of the pre-vote period.
    pub pre_vote: BlockNumber,
    /// The end block of the vote period.
    pub vote: BlockNumber,
    /// The end block of the aggregation period.
    pub aggregation: BlockNumber,
    /// The end block of the appeal period.
    pub appeal: BlockNumber,
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
    /// The court case was closed, the winner outcome was determined.
    Closed { winner: OutcomeReport },
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
    pub backer: AccountId,
    /// The amount of funds which were locked for the appeal.
    pub bond: Balance,
    /// The outcome which was appealed.
    pub appealed_outcome: OutcomeReport,
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
    /// The information about the lifecycle of this court case.
    pub cycle_ends: CycleEnds<BlockNumber>,
}

/// The timing information about a court case.
pub struct RoundTiming<BlockNumber> {
    /// The end block of the pre-vote period.
    pub pre_vote_end: BlockNumber,
    /// The block duration for votes.
    pub vote_period: BlockNumber,
    /// The block duration for revealing votes.
    pub aggregation_period: BlockNumber,
    /// The block duration for appeals.
    pub appeal_period: BlockNumber,
}

impl<BlockNumber: sp_runtime::traits::Saturating + Copy, Appeals: Default>
    CourtInfo<BlockNumber, Appeals>
{
    pub fn new(round_timing: RoundTiming<BlockNumber>) -> Self {
        let pre_vote = round_timing.pre_vote_end;
        let vote = pre_vote.saturating_add(round_timing.vote_period);
        let aggregation = vote.saturating_add(round_timing.aggregation_period);
        let appeal = aggregation.saturating_add(round_timing.appeal_period);
        let cycle_ends = CycleEnds { pre_vote, vote, aggregation, appeal };
        let status = CourtStatus::Open;
        Self { status, appeals: Default::default(), cycle_ends }
    }

    pub fn update_lifecycle(&mut self, round_timing: RoundTiming<BlockNumber>) {
        self.cycle_ends.pre_vote = round_timing.pre_vote_end;
        self.cycle_ends.vote = self.cycle_ends.pre_vote.saturating_add(round_timing.vote_period);
        self.cycle_ends.aggregation =
            self.cycle_ends.vote.saturating_add(round_timing.aggregation_period);
        self.cycle_ends.appeal =
            self.cycle_ends.aggregation.saturating_add(round_timing.appeal_period);
    }
}

/// After a juror was randomly selected to vote in a court case,
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
    /// The juror who was randomly selected.
    pub juror: AccountId,
    /// The weight of the juror in this court case.
    /// The higher the weight the more voice the juror has in the final winner decision.
    pub weight: u32,
    /// The information about the vote state.
    pub vote: Vote<Hash, DelegatedStakes>,
    /// The amount of funds which can be slashed for this court case.
    /// This is related to a multiple of `MinStake` to mitigate Sybil attacks.
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
pub struct JurorPoolItem<AccountId, Balance> {
    /// The amount of funds associated to a juror in order to get selected for a court case.
    pub stake: Balance,
    /// The account which is the juror that might be selected in court cases.
    pub juror: AccountId,
    /// The consumed amount of the stake for all draws. This is useful to reduce the probability
    /// of a juror to be selected again.
    pub consumed_stake: Balance,
}

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
pub struct SelectionValue<Balance, DelegatedStakes> {
    pub weight: u32,
    pub slashable: Balance,
    pub delegated_stakes: DelegatedStakes,
}

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
pub enum SelectionAdd<AccountId, Balance> {
    SelfStake { lock: Balance },
    DelegationStake { delegated_juror: AccountId, lock: Balance },
    DelegationWeight,
}

pub struct SelfInfo<Balance> {
    pub slashable: Balance,
    pub outcome: OutcomeReport,
}

pub struct JurorVoteWithStakes<AccountId, Balance> {
    pub self_info: Option<SelfInfo<Balance>>,
    // many delegators can have delegated to the same juror
    // that's why the value is a vector and should be sorted (binary search by key)
    // the key is the delegator account
    // the value is the delegated stake
    pub delegations: Vec<(AccountId, Balance)>,
}
