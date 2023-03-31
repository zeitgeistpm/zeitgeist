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
pub struct JurorInfo<Balance> {
    /// The juror's amount in the stake weighted pool.
    /// This amount is used to find a juror with a binary search on the pool.
    pub(crate) stake: Balance,
    /// The current amount of funds which are locked in courts.
    pub(crate) active_lock: Balance,
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
pub enum Vote<Hash> {
    /// The juror was randomly selected to vote in a specific court case.
    Drawn,
    /// The juror casted a vote, only providing a hash, which meaning is unknown.
    Secret { secret: Hash },
    /// The juror revealed her raw vote, letting anyone know what she voted.
    Revealed { secret: Hash, outcome: OutcomeReport, salt: Hash },
    /// The juror was denounced, because she revealed her raw vote during the vote phase.
    Denounced { secret: Hash, outcome: OutcomeReport, salt: Hash },
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
pub struct Periods<BlockNumber> {
    /// The end block of the pre-vote period.
    pub(crate) pre_vote_end: BlockNumber,
    /// The end block of the vote period.
    pub(crate) vote_end: BlockNumber,
    /// The end block of the aggregation period.
    pub(crate) aggregation_end: BlockNumber,
    /// The end block of the appeal period.
    pub(crate) appeal_end: BlockNumber,
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
    /// `punished` indicates whether the tardy jurors were punished.
    /// `reassigned` indicates if the winning jurors got the funds from the losers already.
    Closed { winner: OutcomeReport, punished: bool, reassigned: bool },
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
    /// The outcome which was appealed.
    pub(crate) appealed_outcome: OutcomeReport,
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
    pub(crate) status: CourtStatus,
    /// The list of all appeals.
    pub(crate) appeals: Appeals,
    /// The information about the lifecycle of this court case.
    pub(crate) periods: Periods<BlockNumber>,
}

pub struct RoundTiming<BlockNumber> {
    pub(crate) pre_vote_end: BlockNumber,
    pub(crate) vote_period: BlockNumber,
    pub(crate) aggregation_period: BlockNumber,
    pub(crate) appeal_period: BlockNumber,
}

impl<BlockNumber: sp_runtime::traits::Saturating + Copy, Appeals: Default>
    CourtInfo<BlockNumber, Appeals>
{
    pub fn new(round_timing: RoundTiming<BlockNumber>) -> Self {
        let pre_vote_end = round_timing.pre_vote_end;
        let vote_end = pre_vote_end.saturating_add(round_timing.vote_period);
        let aggregation_end = vote_end.saturating_add(round_timing.aggregation_period);
        let appeal_end = aggregation_end.saturating_add(round_timing.appeal_period);
        let periods = Periods { pre_vote_end, vote_end, aggregation_end, appeal_end };
        let status = CourtStatus::Open;
        Self { status, appeals: Default::default(), periods }
    }

    pub fn update_periods(&mut self, round_timing: RoundTiming<BlockNumber>) {
        self.periods.pre_vote_end = round_timing.pre_vote_end;
        self.periods.vote_end = self.periods.pre_vote_end.saturating_add(round_timing.vote_period);
        self.periods.aggregation_end =
            self.periods.vote_end.saturating_add(round_timing.aggregation_period);
        self.periods.appeal_end =
            self.periods.aggregation_end.saturating_add(round_timing.appeal_period);
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
pub struct Draw<AccountId, Balance, Hash> {
    /// The juror who was randomly selected.
    pub(crate) juror: AccountId,
    /// The weight of the juror in this court case.
    /// The higher the weight the more voice the juror has in the final winner decision.
    pub(crate) weight: u32,
    /// The information about the vote state.
    pub(crate) vote: Vote<Hash>,
    /// The amount of funds which can be slashed for this court case.
    /// This is related to a multiple of `MinStake` to mitigate Sybil attacks.
    pub(crate) slashable: Balance,
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
    /// The key for the binary search to efficiently find the juror.
    /// It has to be a unqiue key for each juror.
    pub(crate) stake: Balance,
    /// The account which is the juror.
    pub(crate) juror: AccountId,
    /// The slashable amount for all random draws. This is useful to reduce the probability
    /// of a juror to be selected again.
    pub(crate) consumed_stake: Balance,
}
