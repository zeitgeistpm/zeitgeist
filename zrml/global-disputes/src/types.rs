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

/// The general information about the global dispute.
#[derive(TypeInfo, Decode, Encode, MaxEncodedLen, Clone, PartialEq, Eq)]
pub struct GDInfo<Balance, OwnerInfo> {
    /// The outcome, which is in the lead.
    pub outcome: OutcomeReport,
    /// The information about the winning outcome.
    pub outcome_info: OutcomeInfo<Balance, OwnerInfo>,
    /// The current status of the global dispute.
    pub status: GDStatus,
}

impl<Balance: Saturating, OwnerInfo: Default> GDInfo<Balance, OwnerInfo> {
    pub fn new(outcome: OutcomeReport, vote_sum: Balance) -> Self {
        let outcome_info = OutcomeInfo { outcome_sum: vote_sum, owners: Default::default() };
        GDInfo { outcome, status: GDStatus::Active, outcome_info }
    }
}

#[derive(TypeInfo, Decode, Encode, MaxEncodedLen, Clone, PartialEq, Eq)]
pub enum GDStatus {
    /// The global dispute is in progress.
    Active,
    /// The global dispute is finished.
    Finished,
    /// The global dispute was triggered to get destroyed.
    Destroyed,
}
