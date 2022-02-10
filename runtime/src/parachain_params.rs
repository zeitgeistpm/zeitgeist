#![allow(
    // Constants parameters inside `parameter_types!` already check
    // arithmetic operations at compile time
    clippy::integer_arithmetic
)]

use crate::{
    AccountId, Balances, Origin, ParachainInfo, ParachainSystem, XcmpQueue, BASE,
    BLOCKS_PER_MINUTE, MAXIMUM_BLOCK_WEIGHT,
};
use frame_support::{match_type, parameter_types, traits::Everything, weights::Weight};
use polkadot_parachain::primitives::Sibling;
use sp_runtime::{Perbill, Percent, SaturatedConversion};
use xcm::latest::{BodyId, Junction, Junctions, MultiLocation, NetworkId};
use xcm_builder::{
    AccountId32Aliases, AllowTopLevelPaidExecutionFrom, AllowUnpaidExecutionFrom, CurrencyAdapter,
    IsConcrete, ParentAsSuperuser, ParentIsDefault, RelayChainAsNative, SiblingParachainAsNative,
    SiblingParachainConvertsVia, SignedAccountId32AsNative, SovereignSignedViaLocation,
    TakeWeightCredit,
};
use zeitgeist_primitives::{constants::MICRO, types::Balance};

match_type! {
    pub type ParentOrParentsUnitPlurality: impl Contains<MultiLocation> = {
        MultiLocation { parents: 1, interior: Junctions::Here } |
        MultiLocation { parents: 1, interior: Junctions::X1(Junction::Plurality { id: BodyId::Unit, .. }) }
    };
}
parameter_types! {
    // Author-Mapping
    /// The amount that should be taken as a security deposit when registering a NimbusId.
    pub const CollatorDeposit: Balance = 2 * BASE;

    // Crowdloan
    pub const InitializationPayment: Perbill = Perbill::from_percent(30);
    pub const Initialized: bool = false;
    pub const MaxInitContributorsBatchSizes: u32 = 500;
    pub const MinimumReward: Balance = 0;
    pub const RelaySignaturesThreshold: Perbill = Perbill::from_percent(100);
    pub const SignatureNetworkIdentifier:  &'static [u8] = b"zeitgeist-";

    // Cumulus and Polkadot
    pub Ancestry: MultiLocation = Junction::Parachain(ParachainInfo::parachain_id().into()).into();
    pub const RelayChainLocation: MultiLocation = MultiLocation::parent();
    pub const RelayChainNetwork: NetworkId = NetworkId::Kusama;
    pub const ReservedDmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT / 4;
    pub const ReservedXcmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT / 4;
    pub RelayChainOrigin: Origin = cumulus_pallet_xcm::Origin::Relay.into();
    pub UnitWeightCost: Weight = MICRO.saturated_into();

    // Staking
    /// Rounds before the candidate bond increase/decrease can be executed
    pub const CandidateBondLessDelay: u32 = 2;
    /// Default fixed percent a collator takes off the top of due rewards
    pub const DefaultCollatorCommission: Perbill = Perbill::from_percent(20);
    /// Blocks per round
    pub const DefaultBlocksPerRound: u32 = 2 * BLOCKS_PER_MINUTE as u32;
    /// Default percent of inflation set aside for parachain bond every round
    pub const DefaultParachainBondReservePercent: Percent = Percent::from_percent(30);
    /// Rounds before the delegator bond increase/decrease can be executed
    pub const DelegationBondLessDelay: u32 = 2;
    /// Rounds before the collator leaving the candidates request can be executed
    pub const LeaveCandidatesDelay: u32 = 2;
    /// Rounds before the delegator exit can be executed
    pub const LeaveDelegatorsDelay: u32 = 2;
    /// Maximum bottom delegations per candidate
    pub const MaxBottomDelegationsPerCandidate: u32 = 50;
    /// Maximum delegations per delegator
    pub const MaxDelegationsPerDelegator: u32 = 100;
    /// Maximum top delegations per candidate
    pub const MaxTopDelegationsPerCandidate: u32 = 300;
    /// Minimum round length is 2 minutes
    pub const MinBlocksPerRound: u32 = 2 * BLOCKS_PER_MINUTE as u32;
    /// Minimum stake required to become a collator
    pub const MinCollatorStk: u128 = 64 * BASE;
    /// Minimum stake required to be reserved to be a delegator
    pub const MinDelegatorStk: u128 = BASE / 2;
    /// Minimum collators selected per round, default at genesis and minimum forever after
    pub const MinSelectedCandidates: u32 = 8;
    /// Rounds before the delegator revocation can be executed
    pub const RevokeDelegationDelay: u32 = 2;
    /// Rounds before the reward is paid
    pub const RewardPaymentDelay: u32 = 2;
}

pub type Barrier = (
    TakeWeightCredit,
    AllowTopLevelPaidExecutionFrom<Everything>,
    AllowUnpaidExecutionFrom<ParentOrParentsUnitPlurality>,
);
pub type LocalAssetTransactor =
    CurrencyAdapter<Balances, IsConcrete<RelayChainLocation>, LocationToAccountId, AccountId, ()>;
pub type LocalOriginToLocation = ();
pub type LocationToAccountId = (
    ParentIsDefault<AccountId>,
    SiblingParachainConvertsVia<Sibling, AccountId>,
    AccountId32Aliases<RelayChainNetwork, AccountId>,
);
pub type XcmOriginToTransactDispatchOrigin = (
    SovereignSignedViaLocation<LocationToAccountId, Origin>,
    RelayChainAsNative<RelayChainOrigin, Origin>,
    SiblingParachainAsNative<cumulus_pallet_xcm::Origin, Origin>,
    ParentAsSuperuser<Origin>,
    SignedAccountId32AsNative<RelayChainNetwork, Origin>,
);
pub type XcmRouter = (cumulus_primitives_utility::ParentAsUmp<ParachainSystem, ()>, XcmpQueue);
