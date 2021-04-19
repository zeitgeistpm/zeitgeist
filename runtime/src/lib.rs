#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

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

use frame_support::{
    construct_runtime, parameter_types,
    traits::Randomness,
    weights::{
        constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_PER_SECOND},
        DispatchClass, IdentityFee, Weight,
    },
};
use frame_system::{
    limits::{BlockLength, BlockWeights},
    EnsureRoot,
};
use orml_traits::parameter_type_with_key;
#[cfg(feature = "parachain")]
use parachain_params::*;
use sp_api::impl_runtime_apis;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
use sp_runtime::{
    create_runtime_str, generic, impl_opaque_keys,
    traits::{AccountIdConversion, AccountIdLookup, BlakeTwo256, Block as BlockT},
    transaction_validity::{TransactionSource, TransactionValidity},
    ApplyExtrinsicResult, ModuleId, Perbill,
};
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;
use zeitgeist_primitives::*;
use zrml_swaps_runtime_api::BalanceInfo;

pub const DAYS: BlockNumber = HOURS * 24;
pub const DOLLARS: Balance = BASE / 100; // 100_000_000
pub const HOURS: BlockNumber = MINUTES * 60;
pub const CENTS: Balance = DOLLARS / 100; // 1_000_000
pub const MILLICENTS: Balance = CENTS / 1000; // 1_000
pub const MILLISECS_PER_BLOCK: u64 = 12000;
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const PRIMARY_PROBABILITY: (u64, u64) = (1, 4);
pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("zeitgeist"),
    impl_name: create_runtime_str!("zeitgeist"),
    authoring_version: 1,
    spec_version: 13,
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 1,
};

const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(10);
const MAXIMUM_BLOCK_WEIGHT: Weight = 2 * WEIGHT_PER_SECOND;
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

pub type AdaptedBasicCurrency =
    orml_currencies::BasicCurrencyAdapter<Runtime, Balances, Amount, Balance>;
pub type Address = sp_runtime::MultiAddress<AccountId, ()>;
pub type Balance = u128;
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
  pub const AdvisoryBond: Balance = 25 * DOLLARS;
  pub const BlockHashCount: BlockNumber = 250;
  pub const DisputeBond: Balance = 5 * BASE;
  pub const DisputeFactor: Balance = 2 * BASE;
  pub const DisputePeriod: BlockNumber = DAYS;
  pub const ExistentialDeposit: u128 = 500;
  pub const ExitFee: Balance = 0;
  pub const GetNativeCurrencyId: Asset<MarketId> = Asset::Ztg;
  pub const MaxAssets: usize = 8;
  pub const MaxCategories: u16 = 8;
  pub const MaxDisputes: u16 = 6;
  pub const MaxInRatio: Balance = BASE / 2;
  pub const MaxLocks: u32 = 50;
  pub const MaxOutRatio: Balance = (BASE / 3) + 1;
  pub const MaxTotalWeight: Balance = 50 * BASE;
  pub const MaxWeight: Balance = 50 * BASE;
  pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
  pub const MinLiquidity: Balance = 100 * BASE;
  pub const MinWeight: Balance = BASE;
  pub const OracleBond: Balance = 50 * DOLLARS;
  pub const PmModuleId: ModuleId = ModuleId(*b"zge/pred");
  pub const ReportingPeriod: BlockNumber = DAYS;
  pub const SS58Prefix: u8 = 42;
  pub const SwapsModuleId: ModuleId = ModuleId(*b"zge/swap");
  pub const TransactionByteFee: Balance = 1;
  pub const ValidityBond: Balance = 50 * DOLLARS;
  pub const Version: RuntimeVersion = VERSION;
  pub DustAccount: AccountId = ModuleId(*b"orml/dst").into_account();
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
        construct_runtime!(
            pub enum Runtime where
                Block = Block,
                NodeBlock = opaque::Block,
                UncheckedExtrinsic = UncheckedExtrinsic,
            {
                Balances: pallet_balances::{Call, Config<T>, Event<T>, Pallet, Storage},
                Currency: orml_currencies::{Call, Event<T>, Pallet, Storage},
                Orderbook: zrml_orderbook_v1::{Call, Event<T>, Pallet, Storage},
                PredictionMarkets: zrml_prediction_markets::{Call, Event<T>, Pallet, Storage},
                RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Call, Pallet, Storage},
                Sudo: pallet_sudo::{Call, Config<T>, Event<T>, Pallet, Storage},
                Swaps: zrml_swaps::{Call, Event<T>, Pallet, Storage},
                System: frame_system::{Call, Config, Event<T>, Pallet, Storage},
                Timestamp: pallet_timestamp::{Call, Pallet, Storage, Inherent},
                Tokens: orml_tokens::{Config<T>, Event<T>, Pallet, Storage},
                TransactionPayment: pallet_transaction_payment::{Pallet, Storage},

                $($additional_pallets)*
            }
        );
    }
}
#[cfg(feature = "parachain")]
create_zeitgeist_runtime!(
    ParachainInfo: parachain_info::{Config, Pallet, Storage},
    ParachainSystem: cumulus_pallet_parachain_system::{Call, Pallet, Storage, Inherent, Event},
    XcmHandler: cumulus_pallet_xcm_handler::{Pallet, Call, Event<T>, Origin},
);
#[cfg(not(feature = "parachain"))]
create_zeitgeist_runtime!(
    Aura: pallet_aura::{Config<T>, Pallet},
    Grandpa: pallet_grandpa::{Call, Config, Event, Pallet, Storage},
);

#[cfg(feature = "parachain")]
impl cumulus_pallet_parachain_system::Config for Runtime {
    type DownwardMessageHandlers = XcmHandler;
    type Event = Event;
    type OnValidationData = ();
    type SelfParaId = parachain_info::Pallet<Runtime>;
    type XcmpMessageHandlers = XcmHandler;
}

#[cfg(feature = "parachain")]
impl cumulus_pallet_xcm_handler::Config for Runtime {
    type AccountIdConverter = parachain_params::LocationConverter;
    type Event = Event;
    type SendXcmOrigin = EnsureRoot<AccountId>;
    type UpwardMessageSender = ParachainSystem;
    type XcmExecutor = xcm_executor::XcmExecutor<xcm_config::XcmConfig>;
    type XcmpMessageSender = ParachainSystem;
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
    type OnSetCode = ParachainSystem;
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
    type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
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
    type WeightInfo = ();
}

impl pallet_transaction_payment::Config for Runtime {
    type FeeMultiplierUpdate = ();
    type OnChargeTransaction = pallet_transaction_payment::CurrencyAdapter<Balances, ()>;
    type TransactionByteFee = TransactionByteFee;
    type WeightToFee = IdentityFee<Balance>;
}

#[cfg(feature = "parachain")]
impl parachain_info::Config for Runtime {}

impl zrml_orderbook_v1::Config for Runtime {
    type Currency = Balances;
    type Event = Event;
    type MarketId = MarketId;
    type Shares = Tokens;
}

impl zrml_prediction_markets::Config for Runtime {
    type AdvisoryBond = AdvisoryBond;
    type ApprovalOrigin = EnsureRoot<AccountId>;
    type Currency = Balances;
    type DisputeBond = DisputeBond;
    type DisputeFactor = DisputeFactor;
    type DisputePeriod = DisputePeriod;
    type Event = Event;
    type MarketId = MarketId;
    type MaxCategories = MaxCategories;
    type MaxDisputes = MaxDisputes;
    type ModuleId = PmModuleId;
    type OracleBond = OracleBond;
    type ReportingPeriod = ReportingPeriod;
    type Shares = Tokens;
    type Slash = ();
    type Swap = Swaps;
    type ValidityBond = ValidityBond;
}

impl zrml_swaps::Config for Runtime {
    type Event = Event;
    type ExitFee = ExitFee;
    type MarketId = MarketId;
    type MaxAssets = MaxAssets;
    type MaxInRatio = MaxInRatio;
    type MaxOutRatio = MaxOutRatio;
    type MaxTotalWeight = MaxTotalWeight;
    type MaxWeight = MaxWeight;
    type MinLiquidity = MinLiquidity;
    type MinWeight = MinWeight;
    type ModuleId = SwapsModuleId;
    type Shares = Tokens;
    type WeightInfo = zrml_swaps::weights::WeightInfo<Runtime>;
}

impl_runtime_apis! {
    #[cfg(feature = "runtime-benchmarks")]
    impl frame_benchmarking::Benchmark<Block> for Runtime {
        fn dispatch_benchmark(
            config: frame_benchmarking::BenchmarkConfig,
        ) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
            use frame_benchmarking::{add_benchmark, BenchmarkBatch, Benchmarking, TrackedStorageKey};
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
            add_benchmark!(params, batches, zrml_swaps, Swaps);

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

        fn random_seed() -> <Block as BlockT>::Hash {
            RandomnessCollectiveFlip::random_seed().0
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
        ) -> TransactionValidity {
            Executive::validate_transaction(source, tx)
        }
    }

    impl zrml_swaps_runtime_api::SwapsApi<Block, PoolId, AccountId, Balance, MarketId>
      for Runtime
    {
        fn get_spot_price(
            pool_id: u128,
            asset_in: Asset<MarketId>,
            asset_out: Asset<MarketId>,
        ) -> BalanceInfo<Balance> {
            BalanceInfo {
                amount: Swaps::get_spot_price(pool_id, asset_in, asset_out).ok().unwrap_or(0),
            }
        }

        fn pool_account_id(pool_id: u128) -> AccountId {
            Swaps::pool_account_id(pool_id)
        }

        fn pool_shares_id(pool_id: u128) -> Asset<MarketId> {
            Swaps::pool_shares_id(pool_id)
        }
    }
}

#[cfg(feature = "parachain")]
cumulus_pallet_parachain_system::register_validate_block!(Runtime, Executive);

#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
    NativeVersion {
        runtime_version: VERSION,
        can_author_with: Default::default(),
    }
}
