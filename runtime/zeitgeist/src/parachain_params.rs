#![allow(
    // Constants parameters inside `parameter_types!` already check
    // arithmetic operations at compile time
    clippy::integer_arithmetic
)]
#![cfg(feature = "parachain")]

use super::{
    parameters::MAXIMUM_BLOCK_WEIGHT, AccountId, Balances, Origin, ParachainInfo, ParachainSystem,
    XcmpQueue,
};
use frame_support::{match_types, parameter_types, traits::Everything, weights::Weight};
use pallet_xcm::XcmPassthrough;
use polkadot_parachain::primitives::Sibling;
use sp_runtime::{Perbill, Percent, SaturatedConversion};
use xcm::latest::{BodyId, Junction, Junctions, MultiLocation, NetworkId};
use xcm_builder::{
    AccountId32Aliases, AllowTopLevelPaidExecutionFrom, AllowUnpaidExecutionFrom, CurrencyAdapter,
    IsConcrete, ParentIsPreset, RelayChainAsNative, SiblingParachainAsNative,
    SiblingParachainConvertsVia, SignedAccountId32AsNative, SignedToAccountId32,
    SovereignSignedViaLocation, TakeWeightCredit,
};
use zeitgeist_primitives::{
    constants::{BASE, BLOCKS_PER_MINUTE, MICRO},
    types::Balance,
};

match_types! {
    pub type ParentOrParentsUnitPlurality: impl Contains<MultiLocation> = {
        MultiLocation { parents: 1, interior: Junctions::Here } |
        // Potentially change "Unit" to "Executive" for mainnet once we have separate runtimes
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
    pub const RelayLocation: MultiLocation = MultiLocation::parent();
    // Have to change "Any" to "Kusama" for mainnet once we have separate runtimes
    pub const RelayNetwork: NetworkId = NetworkId::Any;
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

    // XCM
    pub const MaxInstructions: u32 = 100;
}

pub type Barrier = (
    TakeWeightCredit,
    AllowTopLevelPaidExecutionFrom<Everything>,
    AllowUnpaidExecutionFrom<ParentOrParentsUnitPlurality>,
    // ^^^ Parent and its exec plurality get free execution
);

/// Means for transacting assets on this chain.
pub type LocalAssetTransactor = CurrencyAdapter<
    // Use this currency:
    Balances,
    // Use this currency when it is a fungible asset matching the given location or name:
    IsConcrete<RelayLocation>,
    // Do a simple punn to convert an AccountId32 MultiLocation into a native chain account ID:
    LocationToAccountId,
    // Our chain's account ID type (we can't get away without mentioning it explicitly):
    AccountId,
    // We don't track any teleports.
    (),
>;

/// No local origins on this chain are allowed to dispatch XCM sends/executions.
pub type LocalOriginToLocation = SignedToAccountId32<Origin, AccountId, RelayNetwork>;

/// Type for specifying how a `MultiLocation` can be converted into an `AccountId`. This is used
/// when determining ownership of accounts for asset transacting and when attempting to use XCM
/// `Transact` in order to determine the dispatch Origin.
pub type LocationToAccountId = (
    // The parent (Relay-chain) origin converts to the parent `AccountId`.
    ParentIsPreset<AccountId>,
    // Sibling parachain origins convert to AccountId via the `ParaId::into`.
    SiblingParachainConvertsVia<Sibling, AccountId>,
    // Straight up local `AccountId32` origins just alias directly to `AccountId`.
    AccountId32Aliases<RelayNetwork, AccountId>,
);

/// This is the type we use to convert an (incoming) XCM origin into a local `Origin` instance,
/// ready for dispatching a transaction with Xcm's `Transact`. There is an `OriginKind` which can
/// biases the kind of local `Origin` it will become.
pub type XcmOriginToTransactDispatchOrigin = (
    // Sovereign account converter; this attempts to derive an `AccountId` from the origin location
    // using `LocationToAccountId` and then turn that into the usual `Signed` origin. Useful for
    // foreign chains who want to have a local sovereign account on this chain which they control.
    SovereignSignedViaLocation<LocationToAccountId, Origin>,
    // Native converter for Relay-chain (Parent) location; will converts to a `Relay` origin when
    // recognized.
    RelayChainAsNative<RelayChainOrigin, Origin>,
    // Native converter for sibling Parachains; will convert to a `SiblingPara` origin when
    // recognized.
    SiblingParachainAsNative<cumulus_pallet_xcm::Origin, Origin>,
    // Native signed account converter; this just converts an `AccountId32` origin into a normal
    // `Origin::Signed` origin of the same 32-byte value.
    SignedAccountId32AsNative<RelayNetwork, Origin>,
    // Xcm origins can be represented natively under the Xcm pallet's Xcm origin.
    XcmPassthrough<Origin>,
);

/// The means for routing XCM messages which are not for local execution into the right message
/// queues.
pub type XcmRouter = (
    // Two routers - use UMP to communicate with the relay chain:
    cumulus_primitives_utility::ParentAsUmp<ParachainSystem, ()>,
    // ..and XCMP to communicate with the sibling chains.
    XcmpQueue,
);
