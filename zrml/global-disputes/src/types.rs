// Copyright 2022-2023 Forecasting Technologies LTD.
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

/// The original voting outcome owner information.
#[derive(Debug, TypeInfo, Decode, Encode, MaxEncodedLen, Clone, PartialEq, Eq)]
pub enum Possession<AccountId, Balance, OwnerInfo> {
    /// The outcome is owned by a single account.
    /// This happens due to the call to `add_vote_outcome`.
    Paid { owner: AccountId, fee: Balance },
    /// The outcome is owned by multiple accounts.
    /// When a global dispute is triggered, these are the owners of the initially added outcomes.
    Shared { owners: OwnerInfo },
}

impl<AccountId, Balance, OwnerInfo> Possession<AccountId, Balance, OwnerInfo> {
    pub fn get_shared_owners(self) -> Option<OwnerInfo> {
        match self {
            Possession::Shared { owners } => Some(owners),
            _ => None,
        }
    }
}

/// The information about a voting outcome of a global dispute.
#[derive(Debug, TypeInfo, Decode, Encode, MaxEncodedLen, Clone, PartialEq, Eq)]
pub struct OutcomeInfo<AccountId, Balance, OwnerInfo> {
    /// The current sum of all locks on this outcome.
    pub outcome_sum: Balance,
    /// The information about the owner(s) and optionally additional fee.
    pub possession: Possession<AccountId, Balance, OwnerInfo>,
}

/// The general information about the global dispute.
#[derive(Debug, TypeInfo, Decode, Encode, MaxEncodedLen, Clone, PartialEq, Eq)]
pub struct GlobalDisputeInfo<AccountId, Balance, OwnerInfo, BlockNumber> {
    /// The outcome which is in the lead.
    pub winner_outcome: OutcomeReport,
    /// The information about the winning outcome.
    pub outcome_info: OutcomeInfo<AccountId, Balance, OwnerInfo>,
    /// The current status of the global dispute.
    pub status: GdStatus<BlockNumber>,
}

impl<AccountId, Balance: Saturating, OwnerInfo: Default, BlockNumber: Default>
    GlobalDisputeInfo<AccountId, Balance, OwnerInfo, BlockNumber>
{
    pub fn new(
        outcome: OutcomeReport,
        possession: Possession<AccountId, Balance, OwnerInfo>,
        vote_sum: Balance,
    ) -> Self {
        let outcome_info = OutcomeInfo { outcome_sum: vote_sum, possession };
        // `add_outcome_end` and `vote_end` gets set in `start_global_dispute`
        let status =
            GdStatus::Active { add_outcome_end: Default::default(), vote_end: Default::default() };
        GlobalDisputeInfo { winner_outcome: outcome, status, outcome_info }
    }

    pub fn update_winner(&mut self, outcome: OutcomeReport, vote_sum: Balance) {
        self.winner_outcome = outcome;
        self.outcome_info.outcome_sum = vote_sum;
    }
}

/// The current status of the global dispute.
#[derive(TypeInfo, Debug, Decode, Encode, MaxEncodedLen, Clone, PartialEq, Eq)]
pub enum GdStatus<BlockNumber> {
    /// The global dispute is in progress.
    /// The block number `add_outcome_end`, when the addition of new outcomes is over.
    /// The block number `vote_end`, when the global dispute voting period is over.
    Active { add_outcome_end: BlockNumber, vote_end: BlockNumber },
    /// The global dispute is finished.
    Finished,
    /// The global dispute is destroyed.
    Destroyed,
}

// TODO(#986): to remove after the storage migration

/// The information about a voting outcome of a global dispute.
#[derive(Debug, TypeInfo, Decode, Encode, MaxEncodedLen, Clone, PartialEq, Eq)]
pub struct OldOutcomeInfo<Balance, OwnerInfo> {
    /// The current sum of all locks on this outcome.
    pub outcome_sum: Balance,
    /// The vector of owners of the outcome.
    pub owners: OwnerInfo,
}

/// The information about the current highest winning outcome.
#[derive(TypeInfo, Decode, Encode, MaxEncodedLen, Clone, PartialEq, Eq)]
pub struct OldWinnerInfo<Balance, OwnerInfo> {
    /// The outcome, which is in the lead.
    pub outcome: OutcomeReport,
    /// The information about the winning outcome.
    pub outcome_info: OldOutcomeInfo<Balance, OwnerInfo>,
    /// Check, if the global dispute is finished.
    pub is_finished: bool,
}

impl<Balance: Saturating, OwnerInfo: Default> OldWinnerInfo<Balance, OwnerInfo> {
    pub fn new(outcome: OutcomeReport, vote_sum: Balance) -> Self {
        let outcome_info = OldOutcomeInfo { outcome_sum: vote_sum, owners: Default::default() };
        OldWinnerInfo { outcome, is_finished: false, outcome_info }
    }
}

/// An initial vote outcome item with the outcome owner and the initial vote amount.
pub struct InitialItem<AccountId, Balance> {
    /// The outcome which is added as initial global dispute vote possibility.
    pub outcome: OutcomeReport,
    /// The owner of the outcome. This account is rewarded in case the outcome is the winning one.
    pub owner: AccountId,
    /// The vote amount at the start of the global dispute.
    pub amount: Balance,
}
