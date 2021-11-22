#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

extern crate alloc;

#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

pub mod opaque;
#[cfg(feature = "parachain")]
mod parachain_params;
mod parameters;
#[cfg(feature = "txfilter")]
mod txfilter;
#[cfg(feature = "parachain")]
mod xcm_config;
// #[cfg(feature = "parachain")]
mod txfilter;

pub use parameters::*;

use alloc::{boxed::Box, vec, vec::Vec};
use frame_support::{
    construct_runtime,
    traits::{Contains, Everything},
    weights::{constants::RocksDbWeight, IdentityFee},
};
use frame_system::{EnsureOneOf, EnsureRoot};
//#[cfg(feature = "parachain")]
use parity_scale_codec::Encode;
use sp_api::impl_runtime_apis;
use sp_core::{
    crypto::KeyTypeId,
    u32_trait::{_1, _2},
    OpaqueMetadata,
};
use sp_runtime::{
    create_runtime_str, generic,
    traits::{AccountIdConversion, AccountIdLookup, BlakeTwo256, Block as BlockT},
    transaction_validity::{TransactionSource, TransactionValidity},
    ApplyExtrinsicResult,
};
// #[cfg(feature = "parachain")]
use sp_runtime::{
    traits::{Extrinsic as ExtrinsicT, Verify},
    SaturatedConversion,
};
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;
use substrate_fixed::{types::extra::U33, FixedI128, FixedU128};
use zeitgeist_primitives::{constants::*, types::*};
use zrml_rikiddo::types::{EmaMarketVolume, FeeSigmoid, RikiddoSigmoidMV};
#[cfg(feature = "parachain")]
use {
    nimbus_primitives::{CanAuthor, NimbusId},
    parachain_params::*,
};

#[cfg(feature = "txfilter")]
use {
    parity_scale_codec::Encode,
    sp_runtime::{
        traits::{Extrinsic as ExtrinsicT, Verify},
        SaturatedConversion,
    },
    txfilter::{IsCallable, TransactionCallFilter},
};

pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("zeitgeist"),
    impl_name: create_runtime_str!("zeitgeist"),
    authoring_version: 1,
    spec_version: 26,
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 7,
};

pub type Block = generic::Block<Header, UncheckedExtrinsic>;

type Address = sp_runtime::MultiAddress<AccountId, ()>;
type AdvisoryCommitteeCollectiveInstance = pallet_collective::Instance1;
type AdvisoryCommitteeMembershipInstance = pallet_membership::Instance1;
type EnsureRootOrMoreThanHalfOfAdvisoryCommittee = EnsureOneOf<
    AccountId,
    EnsureRoot<AccountId>,
    pallet_collective::EnsureProportionMoreThan<
        _1,
        _2,
        AccountId,
        AdvisoryCommitteeCollectiveInstance,
    >,
>;
type Executive = frame_executive::Executive<
    Runtime,
    Block,
    frame_system::ChainContext<Runtime>,
    Runtime,
    AllPallets,
>;
type Header = generic::Header<BlockNumber, BlakeTwo256>;
type RikiddoSigmoidFeeMarketVolumeEma = zrml_rikiddo::Instance1;

#[cfg(feature = "txfilter")]
type SignedExtra = (
    TransactionCallFilter<IsCallable, Call>,
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    frame_system::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
    pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);

#[cfg(not(feature = "txfilter"))]
type SignedExtra = (
    txfilter::TransactionCallFilter<txfilter::IsCallable, Call>,
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    frame_system::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
    pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);

#[cfg(feature = "txfilter")]
type SignedPayload = generic::SignedPayload<Call, SignedExtra>;
type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;

// Transaction filtering
/// Submits a transaction with the node's public and signature type. Adheres to the signed extension
/// format of the chain.
#[cfg(feature = "txfilter")]
impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Runtime
where
    Call: From<LocalCall>,
{
    fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
        call: Call,
        public: <Signature as Verify>::Signer,
        account: AccountId,
        nonce: <Runtime as frame_system::Config>::Index,
    ) -> Option<(Call, <UncheckedExtrinsic as ExtrinsicT>::SignaturePayload)> {
        // take the biggest period possible.
        let period =
            BlockHashCount::get().checked_next_power_of_two().map(|c| c / 2).unwrap_or(2) as u64;

        let current_block = System::block_number()
            .saturated_into::<u64>()
            // The `System::block_number` is initialized with `n+1`,
            // so the actual block number is `n`.
            .saturating_sub(1);
        let tip = 0;
        let extra: SignedExtra = (
            <TransactionCallFilter<IsCallable, Call>>::new(),
            <frame_system::CheckSpecVersion<Runtime>>::new(),
            <frame_system::CheckTxVersion<Runtime>>::new(),
            <frame_system::CheckGenesis<Runtime>>::new(),
            <frame_system::CheckEra<Runtime>>::from(generic::Era::mortal(period, current_block)),
            <frame_system::CheckNonce<Runtime>>::from(nonce),
            <frame_system::CheckWeight<Runtime>>::new(),
            <pallet_transaction_payment::ChargeTransactionPayment<Runtime>>::from(tip),
        );
        let raw_payload = SignedPayload::new(call, extra)
            .map_err(|e| {
                log::warn!("Unable to create signed payload: {:?}", e);
            })
            .ok()?;
        let signature = raw_payload.using_encoded(|payload| C::sign(payload, public))?;
        let (call, extra, _) = raw_payload.deconstruct();
        Some((call, (sp_runtime::MultiAddress::Id(account), signature, extra)))
    }
}

#[cfg(feature = "txfilter")]
impl frame_system::offchain::SigningTypes for Runtime {
    type Public = <Signature as Verify>::Signer;
    type Signature = Signature;
}

#[cfg(feature = "txfilter")]
impl<C> frame_system::offchain::SendTransactionTypes<C> for Runtime
where
    Call: From<C>,
{
    type OverarchingCall = Call;
    type Extrinsic = UncheckedExtrinsic;
}

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

                // Money
                Balances: pallet_balances::{Call, Config<T>, Event<T>, Pallet, Storage} = 10,
                TransactionPayment: pallet_transaction_payment::{Pallet, Storage} = 11,
                Treasury: pallet_treasury::{Call, Config, Event<T>, Pallet, Storage} = 12,
                Vesting: pallet_vesting::{Call, Config<T>, Event<T>, Pallet, Storage} = 13,

                // Other Parity pallets
                AdvisoryCommitteeCollective: pallet_collective::<Instance1>::{Call, Config<T>, Event<T>, Origin<T>, Pallet, Storage} = 20,
                AdvisoryCommitteeMembership: pallet_membership::<Instance1>::{Call, Config<T>, Event<T>, Pallet, Storage} = 21,
                Identity: pallet_identity::{Call, Event<T>, Pallet, Storage} = 22,
                Sudo: pallet_sudo::{Call, Config<T>, Event<T>, Pallet, Storage} = 23,
                Utility: pallet_utility::{Call, Event, Pallet, Storage} = 24,

                // Third-party
                Currency: orml_currencies::{Call, Event<T>, Pallet, Storage} = 30,
                Tokens: orml_tokens::{Config<T>, Event<T>, Pallet, Storage} = 31,

                // Zeitgeist
                MarketCommons: zrml_market_commons::{Pallet, Storage} = 40,
                Authorized: zrml_authorized::{Call, Event<T>, Pallet, Storage} = 41,
                Court: zrml_court::{Call, Event<T>, Pallet, Storage} = 42,
                LiquidityMining: zrml_liquidity_mining::{Call, Config<T>, Event<T>, Pallet, Storage} = 43,
                RikiddoSigmoidFeeMarketEma: zrml_rikiddo::<Instance1>::{Pallet, Storage} = 44,
                SimpleDisputes: zrml_simple_disputes::{Event<T>, Pallet, Storage} = 45,
                Swaps: zrml_swaps::{Call, Event<T>, Pallet, Storage} = 46,
                PredictionMarkets: zrml_prediction_markets::{Call, Event<T>, Pallet, Storage} = 47,

                $($additional_pallets)*
            }
        );
    }
}
#[cfg(feature = "parachain")]
create_zeitgeist_runtime!(
    // System
    ParachainSystem: cumulus_pallet_parachain_system::{Call, Config, Event<T>, Inherent, Pallet, Storage, ValidateUnsigned} = 50,
    ParachainInfo: parachain_info::{Config, Pallet, Storage} = 51,

    // Consensus
    ParachainStaking: parachain_staking::{Call, Config<T>, Event<T>, Pallet, Storage} = 60,
    AuthorInherent: pallet_author_inherent::{Call, Inherent, Pallet, Storage} = 61,
    AuthorFilter: pallet_author_slot_filter::{Config, Event, Pallet, Storage} = 62,
    AuthorMapping: pallet_author_mapping::{Call, Config<T>, Event<T>, Pallet, Storage} = 63,

    // XCM
    CumulusXcm: cumulus_pallet_xcm::{Event<T>, Origin, Pallet} = 70,
    DmpQueue: cumulus_pallet_dmp_queue::{Call, Event<T>, Pallet, Storage} = 71,
    PolkadotXcm: pallet_xcm::{Call, Event<T>, Origin, Pallet} = 72,
    XcmpQueue: cumulus_pallet_xcmp_queue::{Call, Event<T>, Pallet, Storage} = 73,

    // Third-party
    Crowdloan: pallet_crowdloan_rewards::{Call, Config<T>, Event<T>, Pallet, Storage} = 80,
);
#[cfg(not(feature = "parachain"))]
create_zeitgeist_runtime!(
    // Consensus
    Aura: pallet_aura::{Config<T>, Pallet, Storage} = 50,
    Grandpa: pallet_grandpa::{Call, Config, Event, Pallet, Storage} = 51,
);

// Configure Pallets
#[cfg(feature = "parachain")]
impl cumulus_pallet_dmp_queue::Config for Runtime {
    type Event = Event;
    type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
    type XcmExecutor = xcm_executor::XcmExecutor<xcm_config::XcmConfig>;
}

#[cfg(feature = "parachain")]
impl cumulus_pallet_parachain_system::Config for Runtime {
    type DmpMessageHandler = DmpQueue;
    type Event = Event;
    type OnValidationData = ();
    type OutboundXcmpMessageSource = XcmpQueue;
    type ReservedDmpWeight = crate::parachain_params::ReservedDmpWeight;
    type ReservedXcmpWeight = crate::parachain_params::ReservedXcmpWeight;
    type SelfParaId = parachain_info::Pallet<Runtime>;
    type XcmpMessageHandler = XcmpQueue;
}

#[cfg(feature = "parachain")]
impl cumulus_pallet_xcm::Config for Runtime {
    type Event = Event;
    type XcmExecutor = xcm_executor::XcmExecutor<xcm_config::XcmConfig>;
}

#[cfg(feature = "parachain")]
impl cumulus_pallet_xcmp_queue::Config for Runtime {
    type ChannelInfo = ParachainSystem;
    type Event = Event;
    type VersionWrapper = ();
    type XcmExecutor = xcm_executor::XcmExecutor<xcm_config::XcmConfig>;
}

impl frame_system::Config for Runtime {
    type AccountData = pallet_balances::AccountData<Balance>;
    type AccountId = AccountId;
    type BaseCallFilter = Everything;
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
    type OnKilledAccount = ();
    type OnNewAccount = ();
    #[cfg(feature = "parachain")]
    type OnSetCode = cumulus_pallet_parachain_system::ParachainSetCode<Self>;
    #[cfg(not(feature = "parachain"))]
    type OnSetCode = ();
    type Origin = Origin;
    type PalletInfo = PalletInfo;
    type SS58Prefix = SS58Prefix;
    type SystemWeightInfo = ();
    type Version = Version;
}

#[cfg(not(feature = "parachain"))]
impl pallet_aura::Config for Runtime {
    type AuthorityId = sp_consensus_aura::sr25519::AuthorityId;
    type DisabledValidators = ();
}

#[cfg(feature = "parachain")]
impl pallet_author_inherent::Config for Runtime {
    type AccountLookup = AuthorMapping;
    type AuthorId = NimbusId;
    type CanAuthor = AuthorFilter;
    type EventHandler = ParachainStaking;
    type SlotBeacon = cumulus_pallet_parachain_system::RelaychainBlockNumberProvider<Self>;
}

#[cfg(feature = "parachain")]
impl pallet_author_mapping::Config for Runtime {
    type AuthorId = NimbusId;
    type DepositAmount = CollatorDeposit;
    type DepositCurrency = Balances;
    type Event = Event;
    type WeightInfo = pallet_author_mapping::weights::SubstrateWeight<Runtime>;
}

#[cfg(feature = "parachain")]
impl pallet_author_slot_filter::Config for Runtime {
    type Event = Event;
    type RandomnessSource = RandomnessCollectiveFlip;
    type PotentialAuthors = ParachainStaking;
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

    type WeightInfo = ();
}

#[cfg(feature = "parachain")]
impl pallet_xcm::Config for Runtime {
    type Event = Event;
    type ExecuteXcmOrigin = xcm_builder::EnsureXcmOrigin<Origin, LocalOriginToLocation>;
    type LocationInverter = xcm_builder::LocationInverter<Ancestry>;
    type SendXcmOrigin = xcm_builder::EnsureXcmOrigin<Origin, LocalOriginToLocation>;
    type Weigher = xcm_builder::FixedWeightBounds<UnitWeightCost, Call>;
    type XcmExecuteFilter = Everything;
    type XcmExecutor = xcm_executor::XcmExecutor<xcm_config::XcmConfig>;
    type XcmReserveTransferFilter = ();
    type XcmRouter = XcmRouter;
    type XcmTeleportFilter = Everything;
}

#[cfg(feature = "parachain")]
impl parachain_staking::Config for Runtime {
    type Currency = Balances;
    type DefaultBlocksPerRound = DefaultBlocksPerRound;
    type DefaultCollatorCommission = DefaultCollatorCommission;
    type DefaultParachainBondReservePercent = DefaultParachainBondReservePercent;
    type Event = Event;
    type LeaveCandidatesDelay = LeaveCandidatesDelay;
    type LeaveNominatorsDelay = LeaveNominatorsDelay;
    type MaxCollatorsPerNominator = MaxCollatorsPerNominator;
    type MaxNominatorsPerCollator = MaxNominatorsPerCollator;
    type MinBlocksPerRound = MinBlocksPerRound;
    type MinCollatorCandidateStk = MinCollatorStake;
    type MinCollatorStk = MinCollatorStake;
    type MinNomination = MinNominatorStake;
    type MinNominatorStk = MinNominatorStake;
    type MinSelectedCandidates = MinSelectedCandidates;
    type MonetaryGovernanceOrigin = EnsureRoot<AccountId>;
    type RevokeNominationDelay = RevokeNominationDelay;
    type RewardPaymentDelay = RewardPaymentDelay;
    type WeightInfo = parachain_staking::weights::SubstrateWeight<Runtime>;
}

impl orml_currencies::Config for Runtime {
    type Event = Event;
    type GetNativeCurrencyId = GetNativeCurrencyId;
    type MultiCurrency = Tokens;
    type NativeCurrency = BasicCurrencyAdapter<Runtime, Balances>;
    type WeightInfo = ();
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
    type WeightInfo = ();
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
    type RewardAddressRelayVoteThreshold = RelaySignaturesThreshold;
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
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
}

impl pallet_collective::Config<AdvisoryCommitteeCollectiveInstance> for Runtime {
    type DefaultVote = pallet_collective::PrimeDefaultVote;
    type Event = Event;
    type MaxMembers = AdvisoryCommitteeMaxMembers;
    type MaxProposals = AdvisoryCommitteeMaxProposals;
    type MotionDuration = AdvisoryCommitteeMotionDuration;
    type Origin = Origin;
    type Proposal = Call;
    type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
}

impl pallet_identity::Config for Runtime {
    type BasicDeposit = BasicDeposit;
    type Currency = Balances;
    type Event = Event;
    type FieldDeposit = FieldDeposit;
    type ForceOrigin = EnsureRoot<AccountId>;
    type MaxAdditionalFields = MaxAdditionalFields;
    type MaxRegistrars = MaxRegistrars;
    type MaxSubAccounts = MaxSubAccounts;
    type RegistrarOrigin = EnsureRoot<AccountId>;
    type Slashed = Treasury;
    type SubAccountDeposit = SubAccountDeposit;
    type WeightInfo = pallet_identity::weights::SubstrateWeight<Runtime>;
}

impl pallet_membership::Config<AdvisoryCommitteeMembershipInstance> for Runtime {
    type AddOrigin = EnsureRootOrMoreThanHalfOfAdvisoryCommittee;
    type Event = Event;
    type MaxMembers = AdvisoryCommitteeMaxMembers;
    type MembershipChanged = AdvisoryCommitteeCollective;
    type MembershipInitialized = AdvisoryCommitteeCollective;
    type PrimeOrigin = EnsureRootOrMoreThanHalfOfAdvisoryCommittee;
    type RemoveOrigin = EnsureRootOrMoreThanHalfOfAdvisoryCommittee;
    type ResetOrigin = EnsureRootOrMoreThanHalfOfAdvisoryCommittee;
    type SwapOrigin = EnsureRootOrMoreThanHalfOfAdvisoryCommittee;
    type WeightInfo = pallet_membership::weights::SubstrateWeight<Runtime>;
}

impl pallet_randomness_collective_flip::Config for Runtime {}

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
    type WeightInfo = pallet_timestamp::weights::SubstrateWeight<Runtime>;
}

impl pallet_transaction_payment::Config for Runtime {
    type FeeMultiplierUpdate = ();
    type OnChargeTransaction = pallet_transaction_payment::CurrencyAdapter<Balances, ()>;
    type TransactionByteFee = TransactionByteFee;
    type WeightToFee = IdentityFee<Balance>;
}

impl pallet_treasury::Config for Runtime {
    type ApproveOrigin = EnsureRoot<AccountId>;
    type Burn = Burn;
    type BurnDestination = ();
    type Currency = Balances;
    type Event = Event;
    type MaxApprovals = MaxApprovals;
    type OnSlash = ();
    type PalletId = TreasuryPalletId;
    type ProposalBond = ProposalBond;
    type ProposalBondMinimum = ProposalBondMinimum;
    type RejectOrigin = EnsureRoot<AccountId>;
    type SpendFunds = ();
    type SpendPeriod = SpendPeriod;
    type WeightInfo = pallet_treasury::weights::SubstrateWeight<Runtime>;
}

impl pallet_utility::Config for Runtime {
    type Event = Event;
    type Call = Call;
    type WeightInfo = pallet_utility::weights::SubstrateWeight<Runtime>;
}

impl pallet_vesting::Config for Runtime {
    type Event = Event;
    type Currency = Balances;
    type BlockNumberToBalance = sp_runtime::traits::ConvertInto;
    type MinVestedTransfer = MinVestedTransfer;
    type WeightInfo = pallet_vesting::weights::SubstrateWeight<Runtime>;
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

impl zrml_prediction_markets::Config for Runtime {
    type AdvisoryBond = AdvisoryBond;
    type ApprovalOrigin = EnsureRootOrMoreThanHalfOfAdvisoryCommittee;
    type Authorized = Authorized;
    type Court = Court;
    type DisputeBond = DisputeBond;
    type DisputeFactor = DisputeFactor;
    type DisputePeriod = DisputePeriod;
    type Event = Event;
    type LiquidityMining = LiquidityMining;
    type MarketCommons = MarketCommons;
    type MaxCategories = MaxCategories;
    type MaxDisputes = MaxDisputes;
    type MaxSubsidyPeriod = MaxSubsidyPeriod;
    type MinCategories = MinCategories;
    type MinSubsidyPeriod = MinSubsidyPeriod;
    type OracleBond = OracleBond;
    type PalletId = PmPalletId;
    type ReportingPeriod = ReportingPeriod;
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
    type LiquidityMining = LiquidityMining;
    type MarketId = MarketId;
    type MinAssets = MinAssets;
    type MaxAssets = MaxAssets;
    type MaxInRatio = MaxInRatio;
    type MaxOutRatio = MaxOutRatio;
    type MaxTotalWeight = MaxTotalWeight;
    type MaxWeight = MaxWeight;
    type MinLiquidity = MinLiquidity;
    type MinSubsidy = MinSubsidy;
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
        fn collect_collation_info() -> cumulus_primitives_core::CollationInfo {
            ParachainSystem::collect_collation_info()
        }
    }

    #[cfg(feature = "parachain")]
    impl nimbus_primitives::AuthorFilterAPI<Block, NimbusId> for Runtime {
        fn can_author(author: NimbusId, slot: u32, parent_header: &<Block as BlockT>::Header) -> bool {
            // The Moonbeam runtimes use an entropy source that needs to do some accounting
            // work during block initialization. Therefore we initialize it here to match
            // the state it will be in when the next block is being executed.
            use frame_support::traits::OnInitialize;
            System::initialize(
                &parent_header.number.saturating_add(1),
                &parent_header.hash(),
                &parent_header.digest,
                frame_system::InitKind::Inspection
            );
            RandomnessCollectiveFlip::on_initialize(System::block_number());

            // And now the actual prediction call
            AuthorInherent::can_author(&author, &slot)
        }
    }

    #[cfg(feature = "runtime-benchmarks")]
    impl frame_benchmarking::Benchmark<Block> for Runtime {
        fn benchmark_metadata(extra: bool) -> (
            Vec<frame_benchmarking::BenchmarkList>,
            Vec<frame_support::traits::StorageInfo>,
        ) {
            use frame_benchmarking::{list_benchmark, Benchmarking, BenchmarkList};
            use frame_support::traits::StorageInfoTrait;
            use frame_system_benchmarking::Pallet as SystemBench;

            let mut list = Vec::<BenchmarkList>::new();

            list_benchmark!(list, extra, frame_system, SystemBench::<Runtime>);
            list_benchmark!(list, extra, pallet_balances, Balances);
            list_benchmark!(list, extra, pallet_collective, AdvisoryCommitteeCollective);
            list_benchmark!(list, extra, pallet_membership, AdvisoryCommitteeMembership);
            list_benchmark!(list, extra, pallet_timestamp, Timestamp);
            list_benchmark!(list, extra, pallet_utility, Utility);
            list_benchmark!(list, extra, pallet_vesting, Vesting);
            list_benchmark!(list, extra, zrml_swaps, Swaps);
            list_benchmark!(list, extra, zrml_authorized, Authorized);
            list_benchmark!(list, extra, zrml_court, Court);
            list_benchmark!(list, extra, zrml_prediction_markets, PredictionMarkets);
            list_benchmark!(list, extra, zrml_liquidity_mining, LiquidityMining);

            (list, AllPalletsWithSystem::storage_info())
        }

        fn dispatch_benchmark(
            config: frame_benchmarking::BenchmarkConfig,
        ) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
            use frame_benchmarking::{
                add_benchmark, vec, BenchmarkBatch, Benchmarking, TrackedStorageKey, Vec
            };
            use frame_system_benchmarking::Pallet as SystemBench;

            impl frame_system_benchmarking::Config for Runtime {}

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
            ];

            let mut batches = Vec::<BenchmarkBatch>::new();
            let params = (&config, &whitelist);

            add_benchmark!(params, batches, frame_system, SystemBench::<Runtime>);
            add_benchmark!(params, batches, pallet_balances, Balances);
            add_benchmark!(params, batches, pallet_collective, AdvisoryCommitteeCollective);
            add_benchmark!(params, batches, pallet_membership, AdvisoryCommitteeMembership);
            add_benchmark!(params, batches, pallet_timestamp, Timestamp);
            add_benchmark!(params, batches, pallet_utility, Utility);
            add_benchmark!(params, batches, pallet_vesting, Vesting);
            add_benchmark!(params, batches, zrml_swaps, Swaps);
            add_benchmark!(params, batches, zrml_authorized, Authorized);
            add_benchmark!(params, batches, zrml_court, Court);
            add_benchmark!(params, batches, zrml_prediction_markets, PredictionMarkets);
            add_benchmark!(params, batches, zrml_liquidity_mining, LiquidityMining);

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
            Runtime::metadata().into()
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
            Aura::authorities()
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
