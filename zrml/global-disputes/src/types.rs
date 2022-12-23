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
#[derive(TypeInfo, Decode, Encode, MaxEncodedLen, Clone, PartialEq, Eq)]
pub struct GDInfo<AccountId, Balance, Owners> {
    /// The outcome, which is in the lead.
    pub winner_outcome: OutcomeReport,
    /// The information about the winning outcome.
    pub outcome_info: OutcomeInfo<AccountId, Balance, Owners>,
    /// The current status of the global dispute.
    pub status: GDStatus,
}

impl<AccountId, Balance: Saturating, Owners: Default> GDInfo<AccountId, Balance, Owners> {
    pub fn new(outcome: OutcomeReport, vote_sum: Balance) -> Self {
        let outcome_info = OutcomeInfo { outcome_sum: vote_sum, possession: None };
        GDInfo { winner_outcome: outcome, status: GDStatus::Active, outcome_info }
    }
}

#[derive(TypeInfo, Debug, Decode, Encode, MaxEncodedLen, Clone, PartialEq, Eq)]
pub enum GDStatus {
    /// The global dispute is in progress.
    Active,
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
