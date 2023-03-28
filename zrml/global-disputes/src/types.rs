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
use sp_runtime::traits::Saturating;
use zeitgeist_primitives::types::OutcomeReport;

/// The information about a voting outcome of a global dispute.
#[derive(Debug, TypeInfo, Decode, Encode, MaxEncodedLen, Clone, PartialEq, Eq)]
pub struct OutcomeInfo<Balance, OwnerInfo> {
    /// The current sum of all locks on this outcome.
    pub outcome_sum: Balance,
    /// The vector of owners of the outcome.
    pub owners: OwnerInfo,
}

/// The information about the current highest winning outcome.
#[derive(TypeInfo, Decode, Encode, MaxEncodedLen, Clone, PartialEq, Eq)]
pub struct WinnerInfo<Balance, OwnerInfo> {
    /// The outcome, which is in the lead.
    pub outcome: OutcomeReport,
    /// The information about the winning outcome.
    pub outcome_info: OutcomeInfo<Balance, OwnerInfo>,
    /// Check, if the global dispute is finished.
    pub is_finished: bool,
}

impl<Balance: Saturating, OwnerInfo: Default> WinnerInfo<Balance, OwnerInfo> {
    pub fn new(outcome: OutcomeReport, vote_sum: Balance) -> Self {
        let outcome_info = OutcomeInfo { outcome_sum: vote_sum, owners: Default::default() };
        WinnerInfo { outcome, is_finished: false, outcome_info }
    }
}
