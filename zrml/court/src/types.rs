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
pub struct CrowdfundInfo<Balance> {
    pub(crate) index: u128,
    pub(crate) threshold: Balance,
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
    pub(crate) crowdfund_end: BlockNumber,
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
    pub(crate) is_funded: bool,
}

impl AppealInfo {
    pub fn is_appeal_ready(&self) -> bool {
        self.is_drawn && self.is_funded
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
pub struct CourtInfo<Balance, BlockNumber> {
    pub(crate) crowdfund_info: CrowdfundInfo<Balance>,
    pub(crate) appeal_info: AppealInfo,
    pub(crate) winner: Option<OutcomeReport>,
    pub(crate) periods: Periods<BlockNumber>,
}

impl<Balance: sp_runtime::traits::Saturating, BlockNumber: sp_runtime::traits::Saturating + Copy>
    CourtInfo<Balance, BlockNumber>
{
    pub fn new(
        crowdfund_info: CrowdfundInfo<Balance>,
        now: BlockNumber,
        periods: Periods<BlockNumber>,
        max_appeals: u8,
    ) -> Self {
        let crowdfund_end = now.saturating_add(periods.crowdfund_end);
        let vote_end = crowdfund_end.saturating_add(periods.vote_end);
        let aggregation_end = vote_end.saturating_add(periods.aggregation_end);
        let appeal_end = aggregation_end.saturating_add(periods.appeal_end);
        let periods = Periods { crowdfund_end, vote_end, aggregation_end, appeal_end };
        let appeal_info =
            AppealInfo { current: 1, max: max_appeals, is_drawn: false, is_funded: false };
        Self { crowdfund_info, appeal_info, winner: None, periods }
    }

    pub fn update_periods(&mut self, periods: Periods<BlockNumber>, now: BlockNumber) {
        self.periods.crowdfund_end = now.saturating_add(periods.crowdfund_end);
        self.periods.vote_end = self.periods.crowdfund_end.saturating_add(periods.vote_end);
        self.periods.aggregation_end =
            self.periods.vote_end.saturating_add(periods.aggregation_end);
        self.periods.appeal_end = self.periods.aggregation_end.saturating_add(periods.appeal_end);
    }
}
