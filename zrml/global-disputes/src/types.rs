use frame_support::pallet_prelude::{Decode, Encode, MaxEncodedLen, TypeInfo};
use sp_runtime::traits::Saturating;
use zeitgeist_primitives::types::OutcomeReport;

#[derive(Debug, TypeInfo, Decode, Encode, MaxEncodedLen, Clone, PartialEq, Eq)]
pub enum Possession<AccountId, Balance, Owners> {
    Paid { owner: AccountId, fee: Balance },
    Shared { owners: Owners },
}

impl<AccountId, Balance, Owners> Possession<AccountId, Balance, Owners> {
    pub fn is_shared(&self) -> bool {
        matches!(self, Possession::Shared { .. })
    }

    pub fn get_shared_owners(self) -> Option<Owners> {
        match self {
            Possession::Shared { owners } => Some(owners),
            _ => None,
        }
    }

    pub fn is_paid(&self) -> bool {
        matches!(self, Possession::Paid { .. })
    }
}

/// The information about a voting outcome of a global dispute.
#[derive(Debug, TypeInfo, Decode, Encode, MaxEncodedLen, Clone, PartialEq, Eq)]
pub struct OutcomeInfo<AccountId, Balance, Owners> {
    /// The current sum of all locks on this outcome.
    pub outcome_sum: Balance,
    /// The information about the owner(s) and optionally additional fee.
    pub possession: Option<Possession<AccountId, Balance, Owners>>,
}

/// The general information about the global dispute.
#[derive(Debug, TypeInfo, Decode, Encode, MaxEncodedLen, Clone, PartialEq, Eq)]
pub struct GDInfo<AccountId, Balance, Owners, BlockNumber> {
    /// The outcome, which is in the lead.
    pub winner_outcome: OutcomeReport,
    /// The information about the winning outcome.
    pub outcome_info: OutcomeInfo<AccountId, Balance, Owners>,
    /// The current status of the global dispute.
    pub status: GDStatus<BlockNumber>,
}

impl<AccountId, Balance: Saturating, Owners: Default, BlockNumber>
    GDInfo<AccountId, Balance, Owners, BlockNumber>
{
    pub fn new(outcome: OutcomeReport, vote_sum: Balance) -> Self {
        let outcome_info = OutcomeInfo { outcome_sum: vote_sum, possession: None };
        GDInfo { winner_outcome: outcome, status: GDStatus::Initialized, outcome_info }
    }

    pub fn update_winner(&mut self, outcome: OutcomeReport, vote_sum: Balance) {
        self.winner_outcome = outcome;
        self.outcome_info.outcome_sum = vote_sum;
    }
}

#[derive(TypeInfo, Debug, Decode, Encode, MaxEncodedLen, Clone, PartialEq, Eq)]
pub enum GDStatus<BlockNumber> {
    /// The global dispute is initialized.
    Initialized,
    /// The global dispute is in progress. Save the addition of outcome end and vote end.
    /// The block number `add_outcome_end`, when the addition of new outcomes is over.
    /// The block number `vote_end`, when the global dispute voting period is over.
    Active { add_outcome_end: BlockNumber, vote_end: BlockNumber },
    /// The global dispute is finished.
    Finished,
    /// The global dispute was triggered to get destroyed.
    Destroyed,
}

pub struct RewardInfo<MarketId, AccountId, Balance> {
    pub market_id: MarketId,
    pub reward: Balance,
    pub source: AccountId,
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
