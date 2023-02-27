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
    pub(crate) stake: Balance,
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
pub enum Vote<Hash> {
    Drawn,
    Secret { secret: Hash },
    Revealed { secret: Hash, outcome: OutcomeReport, salt: Hash },
    Denounced { secret: Hash, outcome: OutcomeReport, salt: Hash },
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
pub struct Periods<BlockNumber> {
    pub(crate) backing_end: BlockNumber,
    pub(crate) vote_end: BlockNumber,
    pub(crate) aggregation_end: BlockNumber,
    pub(crate) appeal_end: BlockNumber,
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
pub enum CourtStatus {
    Open,
    Closed { winner: OutcomeReport, punished: bool, reassigned: bool },
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
pub struct AppealInfo<AccountId, Balance> {
    pub(crate) backer: AccountId,
    pub(crate) bond: Balance,
    pub(crate) appealed_outcome: OutcomeReport,
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
pub struct CourtInfo<BlockNumber, Appeals> {
    pub(crate) status: CourtStatus,
    pub(crate) is_appeal_backed: bool,
    pub(crate) is_drawn: bool,
    pub(crate) appeals: Appeals,
    pub(crate) periods: Periods<BlockNumber>,
}

impl<BlockNumber: sp_runtime::traits::Saturating + Copy, Appeals: Default>
    CourtInfo<BlockNumber, Appeals>
{
    pub fn new(now: BlockNumber, periods: Periods<BlockNumber>) -> Self {
        let backing_end = now.saturating_add(periods.backing_end);
        let vote_end = backing_end.saturating_add(periods.vote_end);
        let aggregation_end = vote_end.saturating_add(periods.aggregation_end);
        let appeal_end = aggregation_end.saturating_add(periods.appeal_end);
        let periods = Periods { backing_end, vote_end, aggregation_end, appeal_end };
        let status = CourtStatus::Open;
        Self {
            status,
            is_appeal_backed: false,
            is_drawn: false,
            appeals: Default::default(),
            periods,
        }
    }

    pub fn update_periods(&mut self, periods: Periods<BlockNumber>, now: BlockNumber) {
        self.periods.backing_end = now.saturating_add(periods.backing_end);
        self.periods.vote_end = self.periods.backing_end.saturating_add(periods.vote_end);
        self.periods.aggregation_end =
            self.periods.vote_end.saturating_add(periods.aggregation_end);
        self.periods.appeal_end = self.periods.aggregation_end.saturating_add(periods.appeal_end);
    }
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
pub struct Draw<AccountId, Balance, Hash> {
    pub(crate) juror: AccountId,
    pub(crate) weight: u32,
    pub(crate) vote: Vote<Hash>,
    pub(crate) slashable: Balance,
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
pub struct JurorPoolItem<AccountId, Balance> {
    pub(crate) stake: Balance,
    pub(crate) juror: AccountId,
    pub(crate) slashed: Balance,
}
