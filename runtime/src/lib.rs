#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

extern crate alloc;

#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod opaque;
#[cfg(feature = "parachain")]
mod parachain_params;
mod parameters;
mod weights;
#[cfg(feature = "parachain")]
mod xcm_config;

#[cfg(feature = "parachain")]
pub use parachain_params::*;
pub use parameters::*;

use alloc::{boxed::Box, vec, vec::Vec};
use frame_support::{
    construct_runtime,
    traits::{ConstU16, ConstU32, Contains, EnsureOneOf, EqualPrivilegeOnly, InstanceFilter},
    weights::{constants::RocksDbWeight, IdentityFee},
};
use frame_system::EnsureRoot;
use pallet_collective::{EnsureProportionAtLeast, PrimeDefaultVote};
use sp_api::impl_runtime_apis;
use sp_core::{
    crypto::KeyTypeId,
    u32_trait::{_1, _2, _3, _4},
    OpaqueMetadata,
};
use sp_runtime::{
    create_runtime_str, generic,
    traits::{AccountIdConversion, AccountIdLookup, BlakeTwo256, Block as BlockT},
    transaction_validity::{TransactionSource, TransactionValidity},
    ApplyExtrinsicResult,
};
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;
use substrate_fixed::{types::extra::U33, FixedI128, FixedU128};
use zeitgeist_primitives::{constants::*, types::*};
use zrml_rikiddo::types::{EmaMarketVolume, FeeSigmoid, RikiddoSigmoidMV};
use zrml_swaps::migrations::MigratePoolBaseAsset;
#[cfg(feature = "parachain")]
use {
    frame_support::traits::{Everything, Nothing},
    frame_system::EnsureSigned,
    nimbus_primitives::{CanAuthor, NimbusId},
    xcm_builder::{EnsureXcmOrigin, FixedWeightBounds, LocationInverter},
    xcm_config::XcmConfig,
};

pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("zeitgeist"),
    impl_name: create_runtime_str!("zeitgeist"),
    authoring_version: 1,
    spec_version: 36,
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 13,
    state_version: 1,
};

pub type Block = generic::Block<Header, UncheckedExtrinsic>;

type Address = sp_runtime::MultiAddress<AccountId, ()>;

type Executive = frame_executive::Executive<
    Runtime,
    Block,
    frame_system::ChainContext<Runtime>,
    Runtime,
    AllPalletsWithSystem,
    MigratePoolBaseAsset<Runtime>,
>;

type Header = generic::Header<BlockNumber, BlakeTwo256>;
type RikiddoSigmoidFeeMarketVolumeEma = zrml_rikiddo::Instance1;
type SignedExtra = (
    frame_system::CheckNonZeroSender<Runtime>,
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    frame_system::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
    pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);
type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;

// Governance
type AdvisoryCommitteeInstance = pallet_collective::Instance1;
type AdvisoryCommitteeMembershipInstance = pallet_membership::Instance1;
type CouncilInstance = pallet_collective::Instance2;
type CouncilMembershipInstance = pallet_membership::Instance2;
type TechnicalCommitteeInstance = pallet_collective::Instance3;
type TechnicalCommitteeMembershipInstance = pallet_membership::Instance3;

// Council vote proportions
// At least 50%
type EnsureRootOrHalfCouncil =
    EnsureOneOf<EnsureRoot<AccountId>, EnsureProportionAtLeast<_1, _2, AccountId, CouncilInstance>>;

// At least 66%
type EnsureRootOrTwoThirdsCouncil =
    EnsureOneOf<EnsureRoot<AccountId>, EnsureProportionAtLeast<_2, _3, AccountId, CouncilInstance>>;

// At least 75%
type EnsureRootOrThreeFourthsCouncil =
    EnsureOneOf<EnsureRoot<AccountId>, EnsureProportionAtLeast<_3, _4, AccountId, CouncilInstance>>;

// At least 100%
type EnsureRootOrAllCouncil =
    EnsureOneOf<EnsureRoot<AccountId>, EnsureProportionAtLeast<_1, _1, AccountId, CouncilInstance>>;

// Technical committee vote proportions
// At least 50%
#[cfg(feature = "parachain")]
type EnsureRootOrHalfTechnicalCommittee = EnsureOneOf<
    EnsureRoot<AccountId>,
    EnsureProportionAtLeast<_1, _2, AccountId, TechnicalCommitteeInstance>,
>;

// At least 66%
type EnsureRootOrTwoThirdsTechnicalCommittee = EnsureOneOf<
    EnsureRoot<AccountId>,
    EnsureProportionAtLeast<_2, _3, AccountId, TechnicalCommitteeInstance>,
>;

// At least 100%
type EnsureRootOrAllTechnicalCommittee = EnsureOneOf<
    EnsureRoot<AccountId>,
    EnsureProportionAtLeast<_1, _1, AccountId, TechnicalCommitteeInstance>,
>;

// Advisory committee vote proportions
// At least 50%
type EnsureRootOrHalfAdvisoryCommittee = EnsureOneOf<
    EnsureRoot<AccountId>,
    EnsureProportionAtLeast<_1, _2, AccountId, AdvisoryCommitteeInstance>,
>;

// Technical committee vote proportions
// At least 66%
type EnsureRootOrTwoThirdsAdvisoryCommittee = EnsureOneOf<
    EnsureRoot<AccountId>,
    EnsureProportionAtLeast<_2, _3, AccountId, AdvisoryCommitteeInstance>,
>;

// At least 100%
type EnsureRootOrAllAdvisoryCommittee = EnsureOneOf<
    EnsureRoot<AccountId>,
    EnsureProportionAtLeast<_1, _1, AccountId, AdvisoryCommitteeInstance>,
>;

// Construct runtime
macro_rules! create_zeitgeist_runtime {
    ($($additional_pallets:tt)*) => {
        // Pallets are enumerated based on the dependency graph.
        //
        // For example, `PredictionMarkets` is pĺaced after `SimpleDisputes` because
        // `PredictionMarkets` depends on `SimpleDisputes`.
        construct_runtime!(
            pub enum Runtime where
                Block = Block,
                NodeBlock = generic::Block<Header, sp_runtime::OpaqueExtrinsic>,
                UncheckedExtrinsic = UncheckedExtrinsic,
            {
                // System
                System: frame_system::{Call, Config, Event<T>, Pallet, Storage} = 0,
                Timestamp: pallet_timestamp::{Call, Pallet, Storage, Inherent} = 1,
                RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Pallet, Storage} = 2,
                Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>} = 3,
                Preimage: pallet_preimage::{Pallet, Call, Storage, Event<T>} = 4,

                // Money
                Balances: pallet_balances::{Call, Config<T>, Event<T>, Pallet, Storage} = 10,
                TransactionPayment: pallet_transaction_payment::{Config, Pallet, Storage} = 11,
                Treasury: pallet_treasury::{Call, Config, Event<T>, Pallet, Storage} = 12,
                Vesting: pallet_vesting::{Call, Config<T>, Event<T>, Pallet, Storage} = 13,
                MultiSig: pallet_multisig::{Call, Event<T>, Pallet, Storage} = 14,

                // Governance
                Democracy: pallet_democracy::{Pallet, Call, Storage, Config<T>, Event<T>} = 20,
                AdvisoryCommittee: pallet_collective::<Instance1>::{Call, Config<T>, Event<T>, Origin<T>, Pallet, Storage} = 21,
                AdvisoryCommitteeMembership: pallet_membership::<Instance1>::{Call, Config<T>, Event<T>, Pallet, Storage} = 22,
                Council: pallet_collective::<Instance2>::{Call, Config<T>, Event<T>, Origin<T>, Pallet, Storage} = 23,
                CouncilMembership: pallet_membership::<Instance2>::{Call, Config<T>, Event<T>, Pallet, Storage} = 24,
                TechnicalCommittee: pallet_collective::<Instance3>::{Call, Config<T>, Event<T>, Origin<T>, Pallet, Storage} = 25,
                TechnicalCommitteeMembership: pallet_membership::<Instance3>::{Call, Config<T>, Event<T>, Pallet, Storage} = 26,

                // Other Parity pallets
                Identity: pallet_identity::{Call, Event<T>, Pallet, Storage} = 30,
                Sudo: pallet_sudo::{Call, Config<T>, Event<T>, Pallet, Storage} = 31,
                Utility: pallet_utility::{Call, Event, Pallet, Storage} = 32,
                Proxy: pallet_proxy::{Call, Event<T>, Pallet, Storage} = 33,

                // Third-party
                Currency: orml_currencies::{Call, Event<T>, Pallet, Storage} = 40,
                Tokens: orml_tokens::{Config<T>, Event<T>, Pallet, Storage} = 41,

                // Zeitgeist
                MarketCommons: zrml_market_commons::{Pallet, Storage} = 50,
                Authorized: zrml_authorized::{Call, Event<T>, Pallet, Storage} = 51,
                Court: zrml_court::{Call, Event<T>, Pallet, Storage} = 52,
                LiquidityMining: zrml_liquidity_mining::{Call, Config<T>, Event<T>, Pallet, Storage} = 53,
                RikiddoSigmoidFeeMarketEma: zrml_rikiddo::<Instance1>::{Pallet, Storage} = 54,
                SimpleDisputes: zrml_simple_disputes::{Event<T>, Pallet, Storage} = 55,
                Swaps: zrml_swaps::{Call, Event<T>, Pallet, Storage} = 56,
                PredictionMarkets: zrml_prediction_markets::{Call, Event<T>, Pallet, Storage} = 57,

                $($additional_pallets)*
            }
        );
    }
}

#[cfg(feature = "parachain")]
create_zeitgeist_runtime!(
    // System
    ParachainSystem: cumulus_pallet_parachain_system::{Call, Config, Event<T>, Inherent, Pallet, Storage, ValidateUnsigned} = 100,
    ParachainInfo: parachain_info::{Config, Pallet, Storage} = 101,

    // Consensus
    ParachainStaking: parachain_staking::{Call, Config<T>, Event<T>, Pallet, Storage} = 110,
    AuthorInherent: pallet_author_inherent::{Call, Inherent, Pallet, Storage} = 111,
    AuthorFilter: pallet_author_slot_filter::{Config, Event, Pallet, Storage} = 112,
    AuthorMapping: pallet_author_mapping::{Call, Config<T>, Event<T>, Pallet, Storage} = 113,

    // XCM
    CumulusXcm: cumulus_pallet_xcm::{Event<T>, Origin, Pallet} = 120,
    DmpQueue: cumulus_pallet_dmp_queue::{Call, Event<T>, Pallet, Storage} = 121,
    PolkadotXcm: pallet_xcm::{Call, Config, Event<T>, Origin, Pallet, Storage} = 122,
    XcmpQueue: cumulus_pallet_xcmp_queue::{Call, Event<T>, Pallet, Storage} = 123,

    // Third-party
    Crowdloan: pallet_crowdloan_rewards::{Call, Config<T>, Event<T>, Pallet, Storage} = 130,
);

#[cfg(not(feature = "parachain"))]
create_zeitgeist_runtime!(
    // Consensus
    Aura: pallet_aura::{Config<T>, Pallet, Storage} = 100,
    Grandpa: pallet_grandpa::{Call, Config, Event, Pallet, Storage} = 101,
);

// Configure Pallets
#[cfg(feature = "parachain")]
impl cumulus_pallet_dmp_queue::Config for Runtime {
    type Event = Event;
    type ExecuteOverweightOrigin = EnsureRootOrHalfTechnicalCommittee;
    type XcmExecutor = xcm_executor::XcmExecutor<XcmConfig>;
}

#[cfg(feature = "parachain")]
impl cumulus_pallet_parachain_system::Config for Runtime {
    type DmpMessageHandler = DmpQueue;
    type Event = Event;
    type OnSystemEvent = ();
    type OutboundXcmpMessageSource = XcmpQueue;
    type ReservedDmpWeight = crate::parachain_params::ReservedDmpWeight;
    type ReservedXcmpWeight = crate::parachain_params::ReservedXcmpWeight;
    type SelfParaId = parachain_info::Pallet<Runtime>;
    type XcmpMessageHandler = XcmpQueue;
}

#[cfg(feature = "parachain")]
impl cumulus_pallet_xcm::Config for Runtime {
    type Event = Event;
    type XcmExecutor = xcm_executor::XcmExecutor<XcmConfig>;
}

#[cfg(feature = "parachain")]
impl cumulus_pallet_xcmp_queue::Config for Runtime {
    type ChannelInfo = ParachainSystem;
    type ControllerOrigin = EnsureRootOrTwoThirdsTechnicalCommittee;
    type ControllerOriginConverter = XcmOriginToTransactDispatchOrigin;
    type Event = Event;
    type ExecuteOverweightOrigin = EnsureRootOrHalfTechnicalCommittee;
    type VersionWrapper = ();
    type XcmExecutor = xcm_executor::XcmExecutor<XcmConfig>;
}

#[derive(scale_info::TypeInfo)]
pub struct IsCallable;

cfg_if::cfg_if! {
    if #[cfg(all(feature = "parachain", feature = "txfilter"))] {
        // Restricted parachain.
        impl Contains<Call> for IsCallable {
            fn contains(call: &Call) -> bool {
                match call {
                    // Allowed calls:
                    Call::System(_)
                    | Call::Sudo(_)
                    | Call::Timestamp(_)
                    | Call::AuthorInherent(_)
                    | Call::AuthorMapping(_)
                    | Call::DmpQueue(_)
                    | Call::ParachainSystem(_)
                    | Call::PolkadotXcm(_)
                    | Call::XcmpQueue(_) => true,

                    // Prohibited calls:
                    Call::ParachainStaking(_)
                    | Call::Crowdloan(_)
                    | Call::Balances(_)
                    | Call::Treasury(_)
                    | Call::AdvisoryCommittee(_)
                    | Call::AdvisoryCommitteeMembership(_)
                    | Call::Council(_)
                    | Call::CouncilMembership(_)
                    | Call::TechnicalCommittee(_)
                    | Call::TechnicalCommitteeMembership(_)
                    | Call::MultiSig(_)
                    | Call::Democracy(_)
                    | Call::Scheduler(_)
                    | Call::Preimage(_)
                    | Call::Identity(_)
                    | Call::Utility(_)
                    | Call::Proxy(_)
                    | Call::Currency(_)
                    | Call::Authorized(_)
                    | Call::Court(_)
                    | Call::LiquidityMining(_)
                    | Call::Swaps(_)
                    | Call::PredictionMarkets(_)
                    | Call::Vesting(_) => false,
                }
            }
        }
    // Restricted standalone chain.
    } else if #[cfg(all(feature = "txfilter", not(feature = "parachain")))] {
        impl Contains<Call> for IsCallable {
            fn contains(call: &Call) -> bool {
                match call {
                    // Allowed calls:
                    Call::System(_) | Call::Grandpa(_) | Call::Sudo(_) | Call::Timestamp(_) => true,

                    // Prohibited calls:
                    Call::Balances(_)
                    | Call::Treasury(_)
                    | Call::AdvisoryCommittee(_)
                    | Call::AdvisoryCommitteeMembership(_)
                    | Call::Council(_)
                    | Call::CouncilMembership(_)
                    | Call::TechnicalCommittee(_)
                    | Call::TechnicalCommitteeMembership(_)
                    | Call::MultiSig(_)
                    | Call::Democracy(_)
                    | Call::Scheduler(_)
                    | Call::Preimage(_)
                    | Call::Identity(_)
                    | Call::Utility(_)
                    | Call::Proxy(_)
                    | Call::Currency(_)
                    | Call::Authorized(_)
                    | Call::Court(_)
                    | Call::LiquidityMining(_)
                    | Call::Swaps(_)
                    | Call::PredictionMarkets(_)
                    | Call::Vesting(_) => false,
                }
            }
        }
    // Unrestricted (no "txfilter" feature) chains.
    // Currently disables Rikiddo and Court markets as well as LiquidityMining.
    // Will be relaxed for testnet once runtimes are separated.
    } else {
        impl Contains<Call> for IsCallable {
            fn contains(call: &Call) -> bool {
                use zrml_prediction_markets::Call::{create_categorical_market, create_cpmm_market_and_deploy_assets, create_scalar_market};

                match call {
                    Call::PredictionMarkets(inner_call) => {
                        match inner_call {
                            // Disable Rikiddo markets
                            create_categorical_market { scoring_rule: ScoringRule::RikiddoSigmoidFeeMarketEma, .. } => false,
                            create_scalar_market { scoring_rule: ScoringRule::RikiddoSigmoidFeeMarketEma, .. } => false,
                            // Disable Court dispute resolution mechanism
                            create_categorical_market { mdm: MarketDisputeMechanism::Court, .. } => false,
                            create_scalar_market { mdm: MarketDisputeMechanism::Court, .. } => false,
                            create_cpmm_market_and_deploy_assets { mdm: MarketDisputeMechanism::Court, .. } => false,
                            _ => true
                        }
                    }
                    Call::LiquidityMining(_) => false,
                    _ => true
                }
            }
        }
    }
}

impl frame_system::Config for Runtime {
    type AccountData = pallet_balances::AccountData<Balance>;
    type AccountId = AccountId;
    type BaseCallFilter = IsCallable;
    type BlockHashCount = BlockHashCount;
    type BlockLength = RuntimeBlockLength;
    type BlockNumber = BlockNumber;
    type BlockWeights = RuntimeBlockWeights;
    type Call = Call;
    type DbWeight = RocksDbWeight;
    type Event = Event;
    type Hash = Hash;
    type Hashing = BlakeTwo256;
    type Header = generic::Header<BlockNumber, BlakeTwo256>;
    type Index = Index;
    type Lookup = AccountIdLookup<AccountId, ()>;
    type MaxConsumers = ConstU32<16>;
    type OnKilledAccount = ();
    type OnNewAccount = ();
    #[cfg(feature = "parachain")]
    type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
    #[cfg(not(feature = "parachain"))]
    type OnSetCode = ();
    type Origin = Origin;
    type PalletInfo = PalletInfo;
    type SS58Prefix = SS58Prefix;
    type SystemWeightInfo = weights::frame_system::WeightInfo<Runtime>;
    type Version = Version;
}

#[cfg(not(feature = "parachain"))]
impl pallet_aura::Config for Runtime {
    type AuthorityId = sp_consensus_aura::sr25519::AuthorityId;
    type DisabledValidators = ();
    type MaxAuthorities = MaxAuthorities;
}

#[cfg(feature = "parachain")]
impl pallet_author_inherent::Config for Runtime {
    type AccountLookup = AuthorMapping;
    type CanAuthor = AuthorFilter;
    type EventHandler = ParachainStaking;
    type SlotBeacon = cumulus_pallet_parachain_system::RelaychainBlockNumberProvider<Self>;
}

#[cfg(feature = "parachain")]
impl pallet_author_mapping::Config for Runtime {
    type DepositAmount = CollatorDeposit;
    type DepositCurrency = Balances;
    type Event = Event;
    type WeightInfo = weights::pallet_author_mapping::WeightInfo<Runtime>;
}

#[cfg(feature = "parachain")]
impl pallet_author_slot_filter::Config for Runtime {
    type Event = Event;
    type RandomnessSource = RandomnessCollectiveFlip;
    type PotentialAuthors = ParachainStaking;
    type WeightInfo = weights::pallet_author_slot_filter::WeightInfo<Runtime>;
}

#[cfg(not(feature = "parachain"))]
impl pallet_grandpa::Config for Runtime {
    type Event = Event;
    type Call = Call;
    type KeyOwnerProofSystem = ();
    type KeyOwnerProof =
        <Self::KeyOwnerProofSystem as frame_support::traits::KeyOwnerProofSystem<(
            KeyTypeId,
            pallet_grandpa::AuthorityId,
        )>>::Proof;
    type KeyOwnerIdentification =
        <Self::KeyOwnerProofSystem as frame_support::traits::KeyOwnerProofSystem<(
            KeyTypeId,
            pallet_grandpa::AuthorityId,
        )>>::IdentificationTuple;
    type HandleEquivocation = ();
    type MaxAuthorities = MaxAuthorities;
    // Currently the benchmark does yield an invalid weight implementation
    // type WeightInfo = weights::pallet_grandpa::WeightInfo<Runtime>;
    type WeightInfo = ();
}

#[cfg(feature = "parachain")]
impl pallet_xcm::Config for Runtime {
    type Event = Event;
    type SendXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
    type XcmRouter = XcmRouter;
    type ExecuteXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
    type XcmExecuteFilter = Nothing;
    // ^ Disable dispatchable execute on the XCM pallet.
    // Needs to be `Everything` for local testing.
    type XcmExecutor = xcm_executor::XcmExecutor<XcmConfig>;
    type XcmTeleportFilter = Everything;
    type XcmReserveTransferFilter = Nothing;
    type Weigher = FixedWeightBounds<UnitWeightCost, Call, MaxInstructions>;
    type LocationInverter = LocationInverter<Ancestry>;
    type Origin = Origin;
    type Call = Call;

    const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
    // ^ Override for AdvertisedXcmVersion default
    type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
}

#[cfg(feature = "parachain")]
impl parachain_staking::Config for Runtime {
    type CandidateBondLessDelay = CandidateBondLessDelay;
    type Currency = Balances;
    type DefaultBlocksPerRound = DefaultBlocksPerRound;
    type DefaultCollatorCommission = DefaultCollatorCommission;
    type DefaultParachainBondReservePercent = DefaultParachainBondReservePercent;
    type DelegationBondLessDelay = DelegationBondLessDelay;
    type Event = Event;
    type LeaveCandidatesDelay = LeaveCandidatesDelay;
    type LeaveDelegatorsDelay = LeaveDelegatorsDelay;
    type MaxBottomDelegationsPerCandidate = MaxBottomDelegationsPerCandidate;
    type MaxTopDelegationsPerCandidate = MaxTopDelegationsPerCandidate;
    type MaxDelegationsPerDelegator = MaxDelegationsPerDelegator;
    type MinBlocksPerRound = MinBlocksPerRound;
    type MinCandidateStk = MinCollatorStk;
    type MinCollatorStk = MinCollatorStk;
    type MinDelegation = MinDelegatorStk;
    type MinDelegatorStk = MinDelegatorStk;
    type MinSelectedCandidates = MinSelectedCandidates;
    type MonetaryGovernanceOrigin = EnsureRoot<AccountId>;
    type RevokeDelegationDelay = RevokeDelegationDelay;
    type RewardPaymentDelay = RewardPaymentDelay;
    type WeightInfo = parachain_staking::weights::SubstrateWeight<Runtime>;
}

impl orml_currencies::Config for Runtime {
    type Event = Event;
    type GetNativeCurrencyId = GetNativeCurrencyId;
    type MultiCurrency = Tokens;
    type NativeCurrency = BasicCurrencyAdapter<Runtime, Balances>;
    type WeightInfo = weights::orml_currencies::WeightInfo<Runtime>;
}

impl orml_tokens::Config for Runtime {
    type Amount = Amount;
    type Balance = Balance;
    type CurrencyId = CurrencyId;
    type DustRemovalWhitelist = DustRemovalWhitelist;
    type Event = Event;
    type ExistentialDeposits = ExistentialDeposits;
    type MaxLocks = MaxLocks;
    type OnDust = orml_tokens::TransferDust<Runtime, DustAccount>;
    type WeightInfo = weights::orml_tokens::WeightInfo<Runtime>;
}

#[cfg(feature = "parachain")]
impl pallet_crowdloan_rewards::Config for Runtime {
    type Event = Event;
    type InitializationPayment = InitializationPayment;
    type Initialized = Initialized;
    type MaxInitContributors = MaxInitContributorsBatchSizes;
    type MinimumReward = MinimumReward;
    type RelayChainAccountId = AccountId;
    type RewardCurrency = Balances;
    type RewardAddressAssociateOrigin = EnsureSigned<Self::AccountId>;
    type RewardAddressChangeOrigin = frame_system::EnsureSigned<Self::AccountId>;
    type RewardAddressRelayVoteThreshold = RelaySignaturesThreshold;
    type SignatureNetworkIdentifier = SignatureNetworkIdentifier;
    type VestingBlockNumber = cumulus_primitives_core::relay_chain::BlockNumber;
    type VestingBlockProvider =
        cumulus_pallet_parachain_system::RelaychainBlockNumberProvider<Self>;
    type WeightInfo = pallet_crowdloan_rewards::weights::SubstrateWeight<Runtime>;
}

impl pallet_balances::Config for Runtime {
    type AccountStore = System;
    type Balance = Balance;
    type DustRemoval = ();
    type Event = Event;
    type ExistentialDeposit = ExistentialDeposit;
    type MaxLocks = MaxLocks;
    type MaxReserves = MaxReserves;
    type ReserveIdentifier = [u8; 8];
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>; // weights::pallet_balances::WeightInfo<Runtime>;
}

impl pallet_collective::Config<AdvisoryCommitteeInstance> for Runtime {
    type DefaultVote = PrimeDefaultVote;
    type Event = Event;
    type MaxMembers = AdvisoryCommitteeMaxMembers;
    type MaxProposals = AdvisoryCommitteeMaxProposals;
    type MotionDuration = AdvisoryCommitteeMotionDuration;
    type Origin = Origin;
    type Proposal = Call;
    type WeightInfo = weights::pallet_collective::WeightInfo<Runtime>;
}

impl pallet_collective::Config<CouncilInstance> for Runtime {
    type DefaultVote = PrimeDefaultVote;
    type Event = Event;
    type MaxMembers = CouncilMaxMembers;
    type MaxProposals = CouncilMaxProposals;
    type MotionDuration = CouncilMotionDuration;
    type Origin = Origin;
    type Proposal = Call;
    type WeightInfo = weights::pallet_collective::WeightInfo<Runtime>;
}

impl pallet_collective::Config<TechnicalCommitteeInstance> for Runtime {
    type DefaultVote = PrimeDefaultVote;
    type Event = Event;
    type MaxMembers = TechnicalCommitteeMaxMembers;
    type MaxProposals = TechnicalCommitteeMaxProposals;
    type MotionDuration = TechnicalCommitteeMotionDuration;
    type Origin = Origin;
    type Proposal = Call;
    type WeightInfo = weights::pallet_collective::WeightInfo<Runtime>;
}

impl pallet_democracy::Config for Runtime {
    type Proposal = Call;
    type Event = Event;
    type Currency = Balances;
    type EnactmentPeriod = EnactmentPeriod;
    type LaunchPeriod = LaunchPeriod;
    type VotingPeriod = VotingPeriod;
    type VoteLockingPeriod = VoteLockingPeriod;
    type MinimumDeposit = MinimumDeposit;
    /// Origin that can decide what their next motion is.
    type ExternalOrigin = EnsureRootOrHalfCouncil;
    /// Origin that can have the next scheduled referendum be a straight majority-carries vote.
    type ExternalMajorityOrigin = EnsureRootOrHalfCouncil;
    /// Origina that can have the next scheduled referendum be a straight default-carries
    /// (NTB) vote.
    type ExternalDefaultOrigin = EnsureRootOrAllCouncil;
    /// Origin that can have an ExternalMajority/ExternalDefault vote
    /// be tabled immediately and with a shorter voting/enactment period.
    type FastTrackOrigin = EnsureRootOrTwoThirdsTechnicalCommittee;
    /// Origin from which the next majority-carries (or more permissive) referendum may be tabled
    /// to vote immediately and asynchronously in a similar manner to the emergency origin.
    type InstantOrigin = EnsureRootOrAllTechnicalCommittee;
    type InstantAllowed = InstantAllowed;
    type FastTrackVotingPeriod = FastTrackVotingPeriod;
    /// Origin from which any referendum may be cancelled in an emergency.
    type CancellationOrigin = EnsureRootOrThreeFourthsCouncil;
    /// Origin from which proposals may be blacklisted.
    type BlacklistOrigin = EnsureRootOrAllCouncil;
    /// Origin from which a proposal may be cancelled and its backers slashed.
    type CancelProposalOrigin = EnsureRootOrAllTechnicalCommittee;
    /// Origin for anyone able to veto proposals.
    type VetoOrigin = pallet_collective::EnsureMember<AccountId, TechnicalCommitteeInstance>;
    type CooloffPeriod = CooloffPeriod;
    type PreimageByteDeposit = PreimageByteDeposit;
    type OperationalPreimageOrigin = pallet_collective::EnsureMember<AccountId, CouncilInstance>;
    type Slash = Treasury;
    type Scheduler = Scheduler;
    type PalletsOrigin = OriginCaller;
    type MaxVotes = MaxVotes;
    type WeightInfo = weights::pallet_democracy::WeightInfo<Runtime>;
    type MaxProposals = MaxProposals;
}

impl pallet_identity::Config for Runtime {
    type BasicDeposit = BasicDeposit;
    type Currency = Balances;
    type Event = Event;
    type FieldDeposit = FieldDeposit;
    type ForceOrigin = EnsureRootOrTwoThirdsAdvisoryCommittee;
    type MaxAdditionalFields = MaxAdditionalFields;
    type MaxRegistrars = MaxRegistrars;
    type MaxSubAccounts = MaxSubAccounts;
    type RegistrarOrigin = EnsureRootOrHalfCouncil;
    type Slashed = Treasury;
    type SubAccountDeposit = SubAccountDeposit;
    type WeightInfo = weights::pallet_identity::WeightInfo<Runtime>;
}

impl pallet_membership::Config<AdvisoryCommitteeMembershipInstance> for Runtime {
    type AddOrigin = EnsureRootOrTwoThirdsCouncil;
    type Event = Event;
    type MaxMembers = AdvisoryCommitteeMaxMembers;
    type MembershipChanged = AdvisoryCommittee;
    type MembershipInitialized = AdvisoryCommittee;
    type PrimeOrigin = EnsureRootOrTwoThirdsCouncil;
    type RemoveOrigin = EnsureRootOrTwoThirdsCouncil;
    type ResetOrigin = EnsureRootOrTwoThirdsCouncil;
    type SwapOrigin = EnsureRootOrTwoThirdsCouncil;
    type WeightInfo = weights::pallet_membership::WeightInfo<Runtime>;
}

impl pallet_membership::Config<CouncilMembershipInstance> for Runtime {
    type AddOrigin = EnsureRootOrThreeFourthsCouncil;
    type Event = Event;
    type MaxMembers = CouncilMaxMembers;
    type MembershipChanged = AdvisoryCommittee;
    type MembershipInitialized = AdvisoryCommittee;
    type PrimeOrigin = EnsureRootOrThreeFourthsCouncil;
    type RemoveOrigin = EnsureRootOrThreeFourthsCouncil;
    type ResetOrigin = EnsureRootOrThreeFourthsCouncil;
    type SwapOrigin = EnsureRootOrThreeFourthsCouncil;
    type WeightInfo = weights::pallet_membership::WeightInfo<Runtime>;
}

impl pallet_membership::Config<TechnicalCommitteeMembershipInstance> for Runtime {
    type AddOrigin = EnsureRootOrTwoThirdsCouncil;
    type Event = Event;
    type MaxMembers = TechnicalCommitteeMaxMembers;
    type MembershipChanged = TechnicalCommittee;
    type MembershipInitialized = TechnicalCommittee;
    type PrimeOrigin = EnsureRootOrTwoThirdsCouncil;
    type RemoveOrigin = EnsureRootOrTwoThirdsCouncil;
    type ResetOrigin = EnsureRootOrTwoThirdsCouncil;
    type SwapOrigin = EnsureRootOrTwoThirdsCouncil;
    type WeightInfo = weights::pallet_membership::WeightInfo<Runtime>;
}

impl pallet_multisig::Config for Runtime {
    type Event = Event;
    type Call = Call;
    type Currency = Balances;
    type DepositBase = DepositBase;
    type DepositFactor = DepositFactor;
    type MaxSignatories = ConstU16<100>;
    type WeightInfo = weights::pallet_multisig::WeightInfo<Runtime>;
}

impl pallet_preimage::Config for Runtime {
    type WeightInfo = weights::pallet_preimage::WeightInfo<Runtime>;
    type Event = Event;
    type Currency = Balances;
    type ManagerOrigin = EnsureRoot<AccountId>;
    type MaxSize = PreimageMaxSize;
    type BaseDeposit = PreimageBaseDeposit;
    type ByteDeposit = PreimageByteDeposit;
}

impl InstanceFilter<Call> for ProxyType {
    fn filter(&self, c: &Call) -> bool {
        match self {
            ProxyType::Any => true,
            ProxyType::CancelProxy => {
                matches!(c, Call::Proxy(pallet_proxy::Call::reject_announcement { .. }))
            }
            ProxyType::Governance => matches!(
                c,
                Call::Democracy(..)
                    | Call::Council(..)
                    | Call::TechnicalCommittee(..)
                    | Call::AdvisoryCommittee(..)
                    | Call::Treasury(..)
            ),
            #[cfg(feature = "parachain")]
            ProxyType::Staking => matches!(c, Call::ParachainStaking(..)),
            #[cfg(not(feature = "parachain"))]
            ProxyType::Staking => false,
        }
    }

    fn is_superset(&self, o: &Self) -> bool {
        match (self, o) {
            (x, y) if x == y => true,
            (ProxyType::Any, _) => true,
            (_, ProxyType::Any) => false,
            _ => false,
        }
    }
}

impl pallet_proxy::Config for Runtime {
    type Event = Event;
    type Call = Call;
    type Currency = Balances;
    type ProxyType = ProxyType;
    type ProxyDepositBase = ProxyDepositBase;
    type ProxyDepositFactor = ProxyDepositFactor;
    type MaxProxies = ConstU32<32>;
    type WeightInfo = weights::pallet_proxy::WeightInfo<Runtime>;
    type MaxPending = ConstU32<32>;
    type CallHasher = BlakeTwo256;
    type AnnouncementDepositBase = AnnouncementDepositBase;
    type AnnouncementDepositFactor = AnnouncementDepositFactor;
}

impl pallet_randomness_collective_flip::Config for Runtime {}

impl pallet_scheduler::Config for Runtime {
    type Event = Event;
    type Origin = Origin;
    type PalletsOrigin = OriginCaller;
    type Call = Call;
    type MaximumWeight = MaximumSchedulerWeight;
    type ScheduleOrigin = EnsureRoot<AccountId>;
    type MaxScheduledPerBlock = MaxScheduledPerBlock;
    type WeightInfo = weights::pallet_scheduler::WeightInfo<Runtime>;
    type OriginPrivilegeCmp = EqualPrivilegeOnly;
    type PreimageProvider = Preimage;
    type NoPreimagePostponement = NoPreimagePostponement;
}

impl pallet_sudo::Config for Runtime {
    type Call = Call;
    type Event = Event;
}

impl pallet_timestamp::Config for Runtime {
    type MinimumPeriod = MinimumPeriod;
    type Moment = u64;
    #[cfg(feature = "parachain")]
    type OnTimestampSet = ();
    #[cfg(not(feature = "parachain"))]
    type OnTimestampSet = Aura;
    type WeightInfo = weights::pallet_timestamp::WeightInfo<Runtime>;
}

impl pallet_transaction_payment::Config for Runtime {
    type FeeMultiplierUpdate = ();
    type OnChargeTransaction = pallet_transaction_payment::CurrencyAdapter<Balances, ()>;
    type OperationalFeeMultiplier = OperationalFeeMultiplier;
    type TransactionByteFee = TransactionByteFee;
    type WeightToFee = IdentityFee<Balance>;
}

impl pallet_treasury::Config for Runtime {
    type ApproveOrigin = EnsureRootOrTwoThirdsCouncil;
    type Burn = Burn;
    type BurnDestination = ();
    type Currency = Balances;
    type Event = Event;
    type MaxApprovals = MaxApprovals;
    type OnSlash = ();
    type PalletId = TreasuryPalletId;
    type ProposalBond = ProposalBond;
    type ProposalBondMinimum = ProposalBondMinimum;
    type ProposalBondMaximum = ProposalBondMaximum;
    type RejectOrigin = EnsureRootOrTwoThirdsCouncil;
    type SpendFunds = ();
    type SpendPeriod = SpendPeriod;
    type WeightInfo = weights::pallet_treasury::WeightInfo<Runtime>;
}

impl pallet_utility::Config for Runtime {
    type Event = Event;
    type Call = Call;
    type PalletsOrigin = OriginCaller;
    type WeightInfo = weights::pallet_utility::WeightInfo<Runtime>;
}

impl pallet_vesting::Config for Runtime {
    type Event = Event;
    type Currency = Balances;
    type BlockNumberToBalance = sp_runtime::traits::ConvertInto;
    type MinVestedTransfer = MinVestedTransfer;
    type WeightInfo = pallet_vesting::weights::SubstrateWeight<Runtime>; // weights::pallet_vesting::WeightInfo<Runtime>;

    // `VestingInfo` encode length is 36bytes. 28 schedules gets encoded as 1009 bytes, which is the
    // highest number of schedules that encodes less than 2^10.
    const MAX_VESTING_SCHEDULES: u32 = 28;
}

#[cfg(feature = "parachain")]
impl parachain_info::Config for Runtime {}

impl zrml_authorized::Config for Runtime {
    type Event = Event;
    type MarketCommons = MarketCommons;
    type PalletId = AuthorizedPalletId;
    type WeightInfo = zrml_authorized::weights::WeightInfo<Runtime>;
}

impl zrml_court::Config for Runtime {
    type CourtCaseDuration = CourtCaseDuration;
    type Event = Event;
    type MarketCommons = MarketCommons;
    type PalletId = CourtPalletId;
    type Random = RandomnessCollectiveFlip;
    type StakeWeight = StakeWeight;
    type TreasuryPalletId = TreasuryPalletId;
    type WeightInfo = zrml_court::weights::WeightInfo<Runtime>;
}

impl zrml_liquidity_mining::Config for Runtime {
    type Event = Event;
    type MarketCommons = MarketCommons;
    type MarketId = MarketId;
    type PalletId = LiquidityMiningPalletId;
    type WeightInfo = zrml_liquidity_mining::weights::WeightInfo<Runtime>;
}

impl zrml_market_commons::Config for Runtime {
    type Currency = Balances;
    type MarketId = MarketId;
    type Timestamp = Timestamp;
}

// NoopLiquidityMining implements LiquidityMiningPalletApi with no-ops.
// Has to be public because it will be exposed by Runtime.
pub struct NoopLiquidityMining;

impl zrml_liquidity_mining::LiquidityMiningPalletApi for NoopLiquidityMining {
    type AccountId = AccountId;
    type Balance = Balance;
    type BlockNumber = BlockNumber;
    type MarketId = MarketId;

    fn add_shares(_: Self::AccountId, _: Self::MarketId, _: Self::Balance) {}

    fn distribute_market_incentives(
        _: &Self::MarketId,
    ) -> frame_support::pallet_prelude::DispatchResult {
        Ok(())
    }

    fn remove_shares(_: &Self::AccountId, _: &Self::MarketId, _: Self::Balance) {}
}

impl zrml_prediction_markets::Config for Runtime {
    type AdvisoryBond = AdvisoryBond;
    type ApprovalOrigin = EnsureRootOrHalfAdvisoryCommittee;
    type Authorized = Authorized;
    type Court = Court;
    type CloseOrigin = EnsureRootOrTwoThirdsAdvisoryCommittee;
    type DestroyOrigin = EnsureRootOrAllAdvisoryCommittee;
    type DisputeBond = DisputeBond;
    type DisputeFactor = DisputeFactor;
    type DisputePeriod = DisputePeriod;
    type Event = Event;
    // LiquidityMining is currently unstable.
    // NoopLiquidityMining will be applied only to mainnet once runtimes are separated.
    type LiquidityMining = NoopLiquidityMining;
    // type LiquidityMining = LiquidityMining;
    type MarketCommons = MarketCommons;
    type MaxCategories = MaxCategories;
    type MaxDisputes = MaxDisputes;
    type MaxSubsidyPeriod = MaxSubsidyPeriod;
    type MaxMarketPeriod = MaxMarketPeriod;
    type MinCategories = MinCategories;
    type MinSubsidyPeriod = MinSubsidyPeriod;
    type OracleBond = OracleBond;
    type PalletId = PmPalletId;
    type ReportingPeriod = ReportingPeriod;
    type ResolveOrigin = EnsureRoot<AccountId>;
    type Shares = Tokens;
    type SimpleDisputes = SimpleDisputes;
    type Slash = ();
    type Swaps = Swaps;
    type ValidityBond = ValidityBond;
    type WeightInfo = zrml_prediction_markets::weights::WeightInfo<Runtime>;
}

impl zrml_rikiddo::Config<RikiddoSigmoidFeeMarketVolumeEma> for Runtime {
    type Timestamp = Timestamp;
    type Balance = Balance;
    type FixedTypeU = FixedU128<U33>;
    type FixedTypeS = FixedI128<U33>;
    type BalanceFractionalDecimals = BalanceFractionalDecimals;
    type PoolId = PoolId;
    type Rikiddo = RikiddoSigmoidMV<
        Self::FixedTypeU,
        Self::FixedTypeS,
        FeeSigmoid<Self::FixedTypeS>,
        EmaMarketVolume<Self::FixedTypeU>,
    >;
}

impl zrml_simple_disputes::Config for Runtime {
    type Event = Event;
    type MarketCommons = MarketCommons;
    type PalletId = SimpleDisputesPalletId;
}

impl zrml_swaps::Config for Runtime {
    type Event = Event;
    type ExitFee = ExitFee;
    type FixedTypeU = FixedU128<U33>;
    type FixedTypeS = FixedI128<U33>;
    // LiquidityMining is currently unstable.
    // NoopLiquidityMining will be applied only to mainnet once runtimes are separated.
    type LiquidityMining = NoopLiquidityMining;
    // type LiquidityMining = LiquidityMining;
    type MarketCommons = MarketCommons;
    type MarketId = MarketId;
    type MinAssets = MinAssets;
    type MaxAssets = MaxAssets;
    type MaxInRatio = MaxInRatio;
    type MaxOutRatio = MaxOutRatio;
    type MaxTotalWeight = MaxTotalWeight;
    type MaxWeight = MaxWeight;
    type MinLiquidity = MinLiquidity;
    type MinSubsidy = MinSubsidy;
    type MinSubsidyPerAccount = MinSubsidyPerAccount;
    type MinWeight = MinWeight;
    type PalletId = SwapsPalletId;
    type RikiddoSigmoidFeeMarketEma = RikiddoSigmoidFeeMarketEma;
    type Shares = Currency;
    type WeightInfo = zrml_swaps::weights::WeightInfo<Runtime>;
}

// Implementation of runtime's apis
impl_runtime_apis! {
    #[cfg(feature = "parachain")]
    impl cumulus_primitives_core::CollectCollationInfo<Block> for Runtime {
        fn collect_collation_info(
            header: &<Block as BlockT>::Header
        ) -> cumulus_primitives_core::CollationInfo {
            ParachainSystem::collect_collation_info(header)
        }
    }

    #[cfg(feature = "parachain")]
    // Required to satisify trait bounds at the client implementation.
    impl nimbus_primitives::AuthorFilterAPI<Block, NimbusId> for Runtime {
        fn can_author(_: NimbusId, _: u32, _: &<Block as BlockT>::Header) -> bool {
            panic!("AuthorFilterAPI is no longer supported. Please update your client.")
        }
    }

    #[cfg(feature = "parachain")]
    impl nimbus_primitives::NimbusApi<Block> for Runtime {
        fn can_author(
            author: nimbus_primitives::NimbusId,
            slot: u32,
            parent_header: &<Block as BlockT>::Header
        ) -> bool {

            // Ensure that an update is enforced when we are close to maximum block number
            let block_number = if let Some(bn) = parent_header.number.checked_add(1) {
                bn
            } else {
                log::error!("ERROR: No block numbers left");
                return false;
            };

            use frame_support::traits::OnInitialize;
            System::initialize(
                &block_number,
                &parent_header.hash(),
                &parent_header.digest,
            );
            RandomnessCollectiveFlip::on_initialize(block_number);

            // Because the staking solution calculates the next staking set at the beginning
            // of the first block in the new round, the only way to accurately predict the
            // authors is to compute the selection during prediction.
            if parachain_staking::Pallet::<Self>::round().should_update(block_number) {
                // get author account id
                use nimbus_primitives::AccountLookup;
                let author_account_id = if let Some(account) =
                    pallet_author_mapping::Pallet::<Self>::lookup_account(&author) {
                    account
                } else {
                    // return false if author mapping not registered like in can_author impl
                    return false
                };
                // predict eligibility post-selection by computing selection results now
                let (eligible, _) =
                    pallet_author_slot_filter::compute_pseudo_random_subset::<Self>(
                        parachain_staking::Pallet::<Self>::compute_top_candidates(),
                        &slot
                    );
                eligible.contains(&author_account_id)
            } else {
                AuthorInherent::can_author(&author, &slot)
            }
        }
    }

    #[cfg(feature = "runtime-benchmarks")]
    impl frame_benchmarking::Benchmark<Block> for Runtime {
        fn benchmark_metadata(extra: bool) -> (
            Vec<frame_benchmarking::BenchmarkList>,
            Vec<frame_support::traits::StorageInfo>,
        ) {
            use frame_benchmarking::{list_benchmark, baseline::Pallet as BaselineBench, Benchmarking, BenchmarkList};
            use frame_support::traits::StorageInfoTrait;
            use frame_system_benchmarking::Pallet as SystemBench;
            use orml_benchmarking::list_benchmark as orml_list_benchmark;

            let mut list = Vec::<BenchmarkList>::new();

            list_benchmark!(list, extra, frame_benchmarking, BaselineBench::<Runtime>);
            list_benchmark!(list, extra, frame_system, SystemBench::<Runtime>);
            orml_list_benchmark!(list, extra, orml_currencies, benchmarking::currencies);
            orml_list_benchmark!(list, extra, orml_tokens, benchmarking::tokens);
            list_benchmark!(list, extra, pallet_balances, Balances);
            list_benchmark!(list, extra, pallet_collective, AdvisoryCommittee);
            list_benchmark!(list, extra, pallet_democracy, Democracy);
            list_benchmark!(list, extra, pallet_identity, Identity);
            list_benchmark!(list, extra, pallet_membership, AdvisoryCommitteeMembership);
            list_benchmark!(list, extra, pallet_multisig, MultiSig);
            list_benchmark!(list, extra, pallet_preimage, Preimage);
            list_benchmark!(list, extra, pallet_proxy, Proxy);
            list_benchmark!(list, extra, pallet_scheduler, Scheduler);
            list_benchmark!(list, extra, pallet_timestamp, Timestamp);
            list_benchmark!(list, extra, pallet_treasury, Treasury);
            list_benchmark!(list, extra, pallet_utility, Utility);
            list_benchmark!(list, extra, pallet_vesting, Vesting);
            list_benchmark!(list, extra, zrml_swaps, Swaps);
            list_benchmark!(list, extra, zrml_authorized, Authorized);
            list_benchmark!(list, extra, zrml_court, Court);
            list_benchmark!(list, extra, zrml_prediction_markets, PredictionMarkets);
            list_benchmark!(list, extra, zrml_liquidity_mining, LiquidityMining);

            cfg_if::cfg_if! {
                if #[cfg(feature = "parachain")] {
                    list_benchmark!(list, extra, pallet_author_mapping, AuthorMapping);
                    list_benchmark!(list, extra, pallet_author_slot_filter, AuthorFilter);
                    list_benchmark!(list, extra, parachain_staking, ParachainStaking);
                    list_benchmark!(list, extra, pallet_crowdloan_rewards, Crowdloan);
                } else {
                    list_benchmark!(list, extra, pallet_grandpa, Grandpa);
                }
            }

            (list, AllPalletsWithSystem::storage_info())
        }

        fn dispatch_benchmark(
            config: frame_benchmarking::BenchmarkConfig,
        ) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
            use frame_benchmarking::{
                add_benchmark, baseline::{Pallet as BaselineBench, Config as BaselineConfig}, vec, BenchmarkBatch, Benchmarking, TrackedStorageKey, Vec
            };
            use frame_system_benchmarking::Pallet as SystemBench;
            use orml_benchmarking::{add_benchmark as orml_add_benchmark};

            impl frame_system_benchmarking::Config for Runtime {}
            impl BaselineConfig for Runtime {}

            let whitelist: Vec<TrackedStorageKey> = vec![
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac")
                    .to_vec()
                    .into(),
                hex_literal::hex!("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80")
                    .to_vec()
                    .into(),
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a")
                    .to_vec()
                    .into(),
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850")
                    .to_vec()
                    .into(),
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7")
                    .to_vec()
                    .into(),
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da946c154ffd9992e395af90b5b13cc6f295c77033fce8a9045824a6690bbf99c6db269502f0a8d1d2a008542d5690a0749").to_vec().into(),
                hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da95ecffd7b6c0f78751baa9d281e0bfa3a6d6f646c70792f74727372790000000000000000000000000000000000000000").to_vec().into(),
            ];

            let mut batches = Vec::<BenchmarkBatch>::new();
            let params = (&config, &whitelist);

            add_benchmark!(params, batches, frame_benchmarking, BaselineBench::<Runtime>);
            add_benchmark!(params, batches, frame_system, SystemBench::<Runtime>);
            orml_add_benchmark!(params, batches, orml_currencies, benchmarking::currencies);
            orml_add_benchmark!(params, batches, orml_tokens, benchmarking::tokens);
            add_benchmark!(params, batches, pallet_balances, Balances);
            add_benchmark!(params, batches, pallet_collective, AdvisoryCommittee);
            add_benchmark!(params, batches, pallet_democracy, Democracy);
            add_benchmark!(params, batches, pallet_identity, Identity);
            add_benchmark!(params, batches, pallet_membership, AdvisoryCommitteeMembership);
            add_benchmark!(params, batches, pallet_multisig, MultiSig);
            add_benchmark!(params, batches, pallet_preimage, Preimage);
            add_benchmark!(params, batches, pallet_proxy, Proxy);
            add_benchmark!(params, batches, pallet_scheduler, Scheduler);
            add_benchmark!(params, batches, pallet_timestamp, Timestamp);
            add_benchmark!(params, batches, pallet_treasury, Treasury);
            add_benchmark!(params, batches, pallet_utility, Utility);
            add_benchmark!(params, batches, pallet_vesting, Vesting);
            add_benchmark!(params, batches, zrml_swaps, Swaps);
            add_benchmark!(params, batches, zrml_authorized, Authorized);
            add_benchmark!(params, batches, zrml_court, Court);
            add_benchmark!(params, batches, zrml_prediction_markets, PredictionMarkets);
            add_benchmark!(params, batches, zrml_liquidity_mining, LiquidityMining);


            cfg_if::cfg_if! {
                if #[cfg(feature = "parachain")] {
                    add_benchmark!(params, batches, pallet_author_mapping, AuthorMapping);
                    add_benchmark!(params, batches, pallet_author_slot_filter, AuthorFilter);
                    add_benchmark!(params, batches, parachain_staking, ParachainStaking);
                    add_benchmark!(params, batches, pallet_crowdloan_rewards, Crowdloan);
                } else {
                    add_benchmark!(params, batches, pallet_grandpa, Grandpa);
                }
            }

            if batches.is_empty() {
                return Err("Benchmark not found for this pallet.".into());
            }
            Ok(batches)
        }
    }

    impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
        fn account_nonce(account: AccountId) -> Index {
            System::account_nonce(account)
        }
    }

    impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
        fn query_fee_details(
            uxt: <Block as BlockT>::Extrinsic,
            len: u32,
        ) -> pallet_transaction_payment::FeeDetails<Balance> {
            TransactionPayment::query_fee_details(uxt, len)
        }

        fn query_info(
            uxt: <Block as BlockT>::Extrinsic,
            len: u32,
        ) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
            TransactionPayment::query_info(uxt, len)
        }
    }

    impl sp_api::Core<Block> for Runtime {
        fn execute_block(block: Block) {
            Executive::execute_block(block)
        }

        fn initialize_block(header: &<Block as BlockT>::Header) {
            Executive::initialize_block(header)
        }

        fn version() -> RuntimeVersion {
            VERSION
        }
    }

    impl sp_api::Metadata<Block> for Runtime {
        fn metadata() -> OpaqueMetadata {
            OpaqueMetadata::new(Runtime::metadata().into())
        }
    }

    impl sp_block_builder::BlockBuilder<Block> for Runtime {
        fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
            Executive::apply_extrinsic(extrinsic)
        }

        fn check_inherents(
            block: Block,
            data: sp_inherents::InherentData,
        ) -> sp_inherents::CheckInherentsResult {
            data.check_extrinsics(&block)
        }

        fn finalize_block() -> <Block as BlockT>::Header {
            Executive::finalize_block()
        }

        fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
            data.create_extrinsics()
        }
    }

    #[cfg(not(feature = "parachain"))]
    impl sp_consensus_aura::AuraApi<Block, sp_consensus_aura::sr25519::AuthorityId> for Runtime {
        fn authorities() -> Vec<sp_consensus_aura::sr25519::AuthorityId> {
            Aura::authorities().into_inner()
        }

        fn slot_duration() -> sp_consensus_aura::SlotDuration {
            sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
        }
    }

    #[cfg(not(feature = "parachain"))]
    impl sp_finality_grandpa::GrandpaApi<Block> for Runtime {
        fn current_set_id() -> pallet_grandpa::fg_primitives::SetId {
            Grandpa::current_set_id()
        }

        fn generate_key_ownership_proof(
            _set_id: pallet_grandpa::fg_primitives::SetId,
            _authority_id: pallet_grandpa::AuthorityId,
        ) -> Option<pallet_grandpa::fg_primitives::OpaqueKeyOwnershipProof> {
            None
        }

        fn grandpa_authorities() -> pallet_grandpa::AuthorityList {
            Grandpa::grandpa_authorities()
        }

        fn submit_report_equivocation_unsigned_extrinsic(
            _equivocation_proof: pallet_grandpa::fg_primitives::EquivocationProof<
                <Block as BlockT>::Hash,
                sp_runtime::traits::NumberFor<Block>,
            >,
            _key_owner_proof: pallet_grandpa::fg_primitives::OpaqueKeyOwnershipProof,
        ) -> Option<()> {
            None
        }
    }

    impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
        fn offchain_worker(header: &<Block as BlockT>::Header) {
            Executive::offchain_worker(header)
        }
    }

    impl sp_session::SessionKeys<Block> for Runtime {
        fn decode_session_keys(encoded: Vec<u8>) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
            opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
        }

        fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
            opaque::SessionKeys::generate(seed)
        }
    }

    impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
        fn validate_transaction(
            source: TransactionSource,
            tx: <Block as BlockT>::Extrinsic,
            block_hash: <Block as BlockT>::Hash,
        ) -> TransactionValidity {
            Executive::validate_transaction(source, tx, block_hash)
        }
    }

    impl zrml_swaps_runtime_api::SwapsApi<Block, PoolId, AccountId, Balance, MarketId>
      for Runtime
    {
        fn get_spot_price(
            pool_id: PoolId,
            asset_in: Asset<MarketId>,
            asset_out: Asset<MarketId>,
        ) -> SerdeWrapper<Balance> {
            SerdeWrapper(Swaps::get_spot_price(pool_id, asset_in, asset_out).ok().unwrap_or(0))
        }

        fn pool_account_id(pool_id: PoolId) -> AccountId {
            Swaps::pool_account_id(pool_id)
        }

        fn pool_shares_id(pool_id: PoolId) -> Asset<SerdeWrapper<MarketId>> {
            Asset::PoolShare(SerdeWrapper(pool_id))
        }
    }
}

// Check the timestamp and parachain inherents
#[cfg(feature = "parachain")]
struct CheckInherents;

#[cfg(feature = "parachain")]
impl cumulus_pallet_parachain_system::CheckInherents<Block> for CheckInherents {
    fn check_inherents(
        block: &Block,
        relay_state_proof: &cumulus_pallet_parachain_system::RelayChainStateProof,
    ) -> sp_inherents::CheckInherentsResult {
        let relay_chain_slot = relay_state_proof
            .read_slot()
            .expect("Could not read the relay chain slot from the proof");

        let inherent_data =
            cumulus_primitives_timestamp::InherentDataProvider::from_relay_chain_slot_and_duration(
                relay_chain_slot,
                core::time::Duration::from_secs(6),
            )
            .create_inherent_data()
            .expect("Could not create the timestamp inherent data");

        inherent_data.check_extrinsics(block)
    }
}

// Nimbus's Executive wrapper allows relay validators to verify the seal digest
#[cfg(feature = "parachain")]
cumulus_pallet_parachain_system::register_validate_block! {
    Runtime = Runtime,
    BlockExecutor = pallet_author_inherent::BlockExecutor::<Runtime, Executive>,
    CheckInherents = CheckInherents,
}

#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
    NativeVersion { runtime_version: VERSION, can_author_with: Default::default() }
}

// Accounts protected from being deleted due to a too low amount of funds.
pub struct DustRemovalWhitelist;

impl Contains<AccountId> for DustRemovalWhitelist
where
    frame_support::PalletId: AccountIdConversion<AccountId>,
{
    fn contains(ai: &AccountId) -> bool {
        let pallets = vec![
            AuthorizedPalletId::get(),
            CourtPalletId::get(),
            LiquidityMiningPalletId::get(),
            PmPalletId::get(),
            SimpleDisputesPalletId::get(),
            SwapsPalletId::get(),
        ];

        if let Some(pallet_id) = frame_support::PalletId::try_from_sub_account::<u128>(ai) {
            return pallets.contains(&pallet_id.0);
        }

        for pallet_id in pallets {
            let pallet_acc: AccountId = pallet_id.into_account();

            if pallet_acc == *ai {
                return true;
            }
        }

        false
    }
}
