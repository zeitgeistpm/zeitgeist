// Copyright 2022 Forecasting Technologies LTD.
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

use frame_support::pallet_prelude::{Decode, Encode, MaxEncodedLen, TypeInfo};
use zeitgeist_primitives::types::OutcomeReport;

#[derive(Debug, TypeInfo, Decode, Encode, MaxEncodedLen, Clone, PartialEq, Eq)]
pub struct FundItemInfo<Balance> {
    pub raised: Balance,
}

#[derive(Debug, TypeInfo, Decode, Encode, MaxEncodedLen, Clone, PartialEq, Eq)]
pub struct BackerInfo<Balance> {
    pub reserved: Balance,
}

#[derive(Debug, TypeInfo, Decode, Encode, MaxEncodedLen, Clone, PartialEq, Eq)]
pub enum CrowdfundStatus {
    Active,
    Locked,
    Finished { winner: OutcomeReport },
}

#[derive(Debug, TypeInfo, Decode, Encode, MaxEncodedLen, Clone, PartialEq, Eq)]
pub struct CrowdfundInfo<Balance> {
    pub status: CrowdfundStatus,
    // this should be multiplied by 2 for each new appeal
    pub appeal_threshold: Balance,
}

#[derive(Debug, TypeInfo, Decode, Encode, MaxEncodedLen, Clone, PartialEq, Eq)]
pub enum UniqueFundItem {
    Outcome(OutcomeReport),
    Appeal(u8),
}