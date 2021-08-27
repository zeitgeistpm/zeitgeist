#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

extern crate alloc;

#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

pub mod opaque;
#[cfg(feature = "parachain")]
mod parachain_params;
#[cfg(feature = "parachain")]
mod xcm_config;
#[cfg(feature = "parachain")]
mod xcmp_message;

#[cfg(feature = "parachain")]
pub use xcmp_message::XCMPMessage;

use alloc::{boxed::Box, vec::Vec};
use frame_support::{
    construct_runtime, parameter_types,
    weights::{
        constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_PER_SECOND},
        DispatchClass, IdentityFee, Weight,
    },
    PalletId,
};
use frame_system::{
    limits::{BlockLength, BlockWeights},
    EnsureRoot,
};
use orml_traits::parameter_type_with_key;
use sp_api::impl_runtime_apis;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
use sp_runtime::{
    create_runtime_str, generic,
    traits::{AccountIdConversion, AccountIdLookup, BlakeTwo256, Block as BlockT},
    transaction_validity::{TransactionSource, TransactionValidity},
    ApplyExtrinsicResult, Perbill, Percent,
};
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;
use zeitgeist_primitives::{constants::*, types::*};
#[cfg(feature = "parachain")]
use {
    frame_support::traits::All,
    nimbus_primitives::{CanAuthor, NimbusId},
    parachain_params::*,
    xcm::v0::{MultiAsset, MultiLocation, Xcm},
};

pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("zeitgeist"),
    impl_name: create_runtime_str!("zeitgeist"),
    authoring_version: 1,
    // major_minor_patch_spec-version
    spec_version: 22,
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 5,
};

const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(10);
const MAXIMUM_BLOCK_WEIGHT: Weight = WEIGHT_PER_SECOND / 2;
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

pub type AdaptedBasicCurrency =
    orml_currencies::BasicCurrencyAdapter<Runtime, Balances, Amount, Balance>;
pub type Address = sp_runtime::MultiAddress<AccountId, ()>;
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
pub type BlockId = generic::BlockId<Block>;
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>;
pub type Executive = frame_executive::Executive<
    Runtime,
    Block,
    frame_system::ChainContext<Runtime>,
    Runtime,
    AllPallets,
>;
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
pub type SignedBlock = generic::SignedBlock<Block>;
pub type SignedExtra = (
    frame_system::CheckSpecVersion<Runtime>,
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    frame_system::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
    pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;

parameter_types! {
  pub const BondDuration: u32 = BLOCKS_PER_DAY as u32;
  pub const CollatorDeposit: Balance = 2 * BASE;
  pub const DefaultCollatorCommission: Perbill = Perbill::from_percent(20);
  pub const DefaultParachainBondReservePercent: Percent = Percent::from_percent(30);
  pub const GetNativeCurrencyId: Asset<MarketId> = Asset::Ztg;
  pub const MaxCollatorsPerNominator: u32 = 16;
  pub const MaxLocks: u32 = 50;
  pub const MaxNominatorsPerCollator: u32 = 32;
  pub const MinBlocksPerRound: u32 = (BLOCKS_PER_DAY / 6) as _;
  pub const MinCollatorStake: u128 = 64 * BASE;
  pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
  pub const MinNominatorStake: u128 = BASE / 2;
  pub const MinSelectedCandidates: u32 = 1;
  pub const SS58Prefix: u8 = 73;
  pub const TransactionByteFee: Balance = 100 * MICRO;
  pub const Version: RuntimeVersion = VERSION;
  pub DustAccount: AccountId = PalletId(*b"orml/dst").into_account();
  pub RuntimeBlockLength: BlockLength = BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
  pub RuntimeBlockWeights: BlockWeights = BlockWeights::builder()
    .base_block(BlockExecutionWeight::get())
    .for_class(DispatchClass::all(), |weights| {
      weights.base_extrinsic = ExtrinsicBaseWeight::get();
    })
    .for_class(DispatchClass::Normal, |weights| {
      weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
    })
    .for_class(DispatchClass::Operational, |weights| {
      weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
      weights.reserved = Some(
        MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT
      );
    })
    .avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
    .build_or_panic();
}

parameter_type_with_key! {
    pub ExistentialDeposits: |_currency_id: CurrencyId| -> Balance {
      Default::default()
    };
}

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

                // Other Parity pallets
                Sudo: pallet_sudo::{Call, Config<T>, Event<T>, Pallet, Storage} = 20,
                Utility: pallet_utility::{Call, Event, Pallet, Storage} = 21,

                // Third-party
                Currency: orml_currencies::{Call, Event<T>, Pallet, Storage} = 30,
                Tokens: orml_tokens::{Config<T>, Event<T>, Pallet, Storage} = 31,

                // Zeitgeist
                LiquidityMining: zrml_liquidity_mining::{Call, Config<T>, Event<T>, Pallet, Storage} = 40,
                Orderbook: zrml_orderbook_v1::{Call, Event<T>, Pallet, Storage} = 41,

                MarketCommons: zrml_market_commons::{Pallet, Storage} = 42,
                Swaps: zrml_swaps::{Call, Event<T>, Pallet, Storage} = 43,
                SimpleDisputes: zrml_simple_disputes::{Event<T>, Pallet, Storage} = 44,
                PredictionMarkets: zrml_prediction_markets::{Call, Event<T>, Pallet, Storage} = 45,

                $($additional_pallets)*
            }
        );
    }
}
#[cfg(feature = "parachain")]
create_zeitgeist_runtime!(
    // System
    ParachainSystem: cumulus_pallet_parachain_system::{Call, Pallet, Storage, Inherent, Event<T>} = 50,
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
    CumulusPing: cumulus_ping::{Call, Event<T>, Pallet, Storage} = 99,
);
#[cfg(not(feature = "parachain"))]
create_zeitgeist_runtime!(
    // Consensus
    Aura: pallet_aura::{Config<T>, Pallet, Storage} = 50,
    Grandpa: pallet_grandpa::{Call, Config, Event, Pallet, Storage} = 51,
);

#[cfg(feature = "parachain")]
impl cumulus_pallet_dmp_queue::Config for Runtime {
    type Event = Event;
    type XcmExecutor = xcm_executor::XcmExecutor<xcm_config::XcmConfig>;
    type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
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
    type XcmExecutor = xcm_executor::XcmExecutor<xcm_config::XcmConfig>;
}

#[cfg(feature = "parachain")]
impl cumulus_ping::Config for Runtime {
    type Call = Call;
    type Event = Event;
    type Origin = Origin;
    type XcmSender = XcmRouter;
}

impl frame_system::Config for Runtime {
    type AccountData = pallet_balances::AccountData<Balance>;
    type AccountId = AccountId;
    type BaseCallFilter = ();
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
    type SendXcmOrigin = xcm_builder::EnsureXcmOrigin<Origin, LocalOriginToLocation>;
    type Weigher = xcm_builder::FixedWeightBounds<UnitWeightCost, Call>;
    type XcmExecuteFilter = All<(MultiLocation, Xcm<Call>)>;
    type XcmExecutor = xcm_executor::XcmExecutor<xcm_config::XcmConfig>;
    type XcmReserveTransferFilter = ();
    type XcmRouter = XcmRouter;
    type XcmTeleportFilter = All<(MultiLocation, Vec<MultiAsset>)>;
}

#[cfg(feature = "parachain")]
impl parachain_staking::Config for Runtime {
    type BondDuration = BondDuration;
    type Currency = Balances;
    type DefaultBlocksPerRound = DefaultBlocksPerRound;
    type DefaultCollatorCommission = DefaultCollatorCommission;
    type DefaultParachainBondReservePercent = DefaultParachainBondReservePercent;
    type Event = Event;
    type MaxCollatorsPerNominator = MaxCollatorsPerNominator;
    type MaxNominatorsPerCollator = MaxNominatorsPerCollator;
    type MinBlocksPerRound = MinBlocksPerRound;
    type MinCollatorCandidateStk = MinCollatorStake;
    type MinCollatorStk = MinCollatorStake;
    type MinNomination = MinNominatorStake;
    type MinNominatorStk = MinNominatorStake;
    type MinSelectedCandidates = MinSelectedCandidates;
    type MonetaryGovernanceOrigin = EnsureRoot<AccountId>;
    type WeightInfo = ();
}

impl orml_currencies::Config for Runtime {
    type Event = Event;
    type GetNativeCurrencyId = GetNativeCurrencyId;
    type MultiCurrency = Tokens;
    type NativeCurrency = AdaptedBasicCurrency;
    type WeightInfo = ();
}

impl orml_tokens::Config for Runtime {
    type Amount = Amount;
    type Balance = Balance;
    type CurrencyId = CurrencyId;
    type Event = Event;
    type ExistentialDeposits = ExistentialDeposits;
    type MaxLocks = MaxLocks;
    type OnDust = ();
    type WeightInfo = ();
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
    type WeightInfo = ();
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
    type WeightInfo = ();
}

#[cfg(feature = "parachain")]
impl parachain_info::Config for Runtime {}

impl zrml_liquidity_mining::Config for Runtime {
    type Currency = Balances;
    type Event = Event;
    type MarketId = MarketId;
    type PalletId = LiquidityMiningPalletId;
    type WeightInfo = zrml_liquidity_mining::weights::WeightInfo<Runtime>;
}

impl zrml_market_commons::Config for Runtime {
    type Currency = Balances;
    type MarketId = MarketId;
}

impl zrml_orderbook_v1::Config for Runtime {
    type Currency = Balances;
    type Event = Event;
    type MarketId = MarketId;
    type Shares = Tokens;
    type WeightInfo = zrml_orderbook_v1::weights::WeightInfo<Runtime>;
}

impl zrml_prediction_markets::Config for Runtime {
    type AdvisoryBond = AdvisoryBond;
    type ApprovalOrigin = EnsureRoot<AccountId>;
    type DisputeBond = DisputeBond;
    type DisputeFactor = DisputeFactor;
    type DisputePeriod = DisputePeriod;
    type Event = Event;
    type MarketCommons = MarketCommons;
    type LiquidityMining = LiquidityMining;
    type MaxCategories = MaxCategories;
    type MaxDisputes = MaxDisputes;
    type MinCategories = MinCategories;
    type OracleBond = OracleBond;
    type PalletId = PmPalletId;
    type ReportingPeriod = ReportingPeriod;
    type Shares = Tokens;
    type SimpleDisputes = SimpleDisputes;
    type Slash = ();
    type Swaps = Swaps;
    type Timestamp = Timestamp;
    type ValidityBond = ValidityBond;
    type WeightInfo = zrml_prediction_markets::weights::WeightInfo<Runtime>;
}

impl zrml_simple_disputes::Config for Runtime {
    type Event = Event;
    type LiquidityMining = LiquidityMining;
    type MarketCommons = MarketCommons;
    type OracleBond = OracleBond;
    type PalletId = SimpleDisputesPalletId;
    type Shares = Tokens;
    type Swaps = Swaps;
    type ValidityBond = ValidityBond;
}

impl zrml_swaps::Config for Runtime {
    type Event = Event;
    type ExitFee = ExitFee;
    type LiquidityMining = LiquidityMining;
    type MarketId = MarketId;
    type MaxAssets = MaxAssets;
    type MaxInRatio = MaxInRatio;
    type MaxOutRatio = MaxOutRatio;
    type MaxTotalWeight = MaxTotalWeight;
    type MaxWeight = MaxWeight;
    type MinLiquidity = MinLiquidity;
    type MinWeight = MinWeight;
    type PalletId = SwapsPalletId;
    type Shares = Currency;
    type WeightInfo = zrml_swaps::weights::WeightInfo<Runtime>;
}

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
                &(parent_header.number + 1),
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
            add_benchmark!(params, batches, pallet_timestamp, Timestamp);
            add_benchmark!(params, batches, pallet_utility, Utility);
            add_benchmark!(params, batches, zrml_swaps, Swaps);
            add_benchmark!(params, batches, zrml_prediction_markets, PredictionMarkets);
            add_benchmark!(params, batches, zrml_liquidity_mining, LiquidityMining);
            add_benchmark!(params, batches, zrml_orderbook_v1, Orderbook);

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
