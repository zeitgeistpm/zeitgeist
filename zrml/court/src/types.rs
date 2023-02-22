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
pub struct AppealInfo {
    pub(crate) current: u8,
    pub(crate) max: u8,
    pub(crate) is_drawn: bool,
    pub(crate) is_backed: bool,
}

impl AppealInfo {
    pub fn is_appeal_ready(&self) -> bool {
        self.is_drawn && self.is_backed
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
pub struct CourtInfo<BlockNumber> {
    pub(crate) status: CourtStatus,
    pub(crate) appeal_info: AppealInfo,
    pub(crate) periods: Periods<BlockNumber>,
}

impl<BlockNumber: sp_runtime::traits::Saturating + Copy> CourtInfo<BlockNumber> {
    pub fn new(now: BlockNumber, periods: Periods<BlockNumber>, max_appeals: u8) -> Self {
        let backing_end = now.saturating_add(periods.backing_end);
        let vote_end = backing_end.saturating_add(periods.vote_end);
        let aggregation_end = vote_end.saturating_add(periods.aggregation_end);
        let appeal_end = aggregation_end.saturating_add(periods.appeal_end);
        let periods = Periods { backing_end, vote_end, aggregation_end, appeal_end };
        // 2^1 * 3 + 2^1 - 1 = 7 jurors in the first appeal round (`necessary_jurors_num`)
        let appeal_info =
            AppealInfo { current: 1, max: max_appeals, is_drawn: false, is_backed: false };
        let status = CourtStatus::Open;
        Self { status, appeal_info, periods }
    }

    pub fn update_periods(&mut self, periods: Periods<BlockNumber>, now: BlockNumber) {
        self.periods.backing_end = now.saturating_add(periods.backing_end);
        self.periods.vote_end = self.periods.backing_end.saturating_add(periods.vote_end);
        self.periods.aggregation_end =
            self.periods.vote_end.saturating_add(periods.aggregation_end);
        self.periods.appeal_end = self.periods.aggregation_end.saturating_add(periods.appeal_end);
    }
}
