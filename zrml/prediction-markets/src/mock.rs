use crate as prediction_markets;
use frame_support::{
    construct_runtime, ord_parameter_types, parameter_types,
    traits::{OnFinalize, OnInitialize},
    weights::Weight,
    PalletId,
};
use frame_system::EnsureSignedBy;
use orml_traits::parameter_type_with_key;
use sp_runtime::{
    testing::Header,
    traits::{AccountIdConversion, BlakeTwo256, IdentityLookup},
    Perbill,
};
use zeitgeist_primitives::{
    constants::{
        ExitFee, MaxAssets, MaxCategories, MaxDisputes, MaxInRatio, MaxOutRatio, MaxTotalWeight,
        MaxWeight, MinLiquidity, MinWeight, BASE, BLOCK_HASH_COUNT,
    },
    types::{
        AccountIdTest, Amount, Asset, Balance, BlockNumber, BlockTest, CurrencyId, Hash, Index,
        MarketId, SerdeWrapper, UncheckedExtrinsicTest,
    },
};

pub const ALICE: AccountIdTest = 0;
pub const BOB: AccountIdTest = 1;
pub const CHARLIE: AccountIdTest = 2;
pub const DAVE: AccountIdTest = 3;
pub const EVE: AccountIdTest = 4;
pub const SUDO: AccountIdTest = 69;

pub type Block = BlockTest<Runtime>;
pub type UncheckedExtrinsic = UncheckedExtrinsicTest<Runtime>;
pub type AdaptedBasicCurrency =
    orml_currencies::BasicCurrencyAdapter<Runtime, Balances, Amount, Balance>;

ord_parameter_types! {
    pub const Sudo: AccountIdTest = 69;
}

parameter_types! {
    pub const AdvisoryBond: Balance = 50;
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    pub const BlockHashCount: u64 = BLOCK_HASH_COUNT;
    pub const DisputeBond: Balance = 100;
    pub const DisputeFactor: Balance = 25;
    pub const DisputePeriod: BlockNumber = 10;
    pub const ExistentialDeposit: u64 = 1;
    pub const GetNativeCurrencyId: Asset<MarketId> = Asset::Ztg;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const MaximumBlockWeight: Weight = 1024;
    pub const MaxLocks: u32 = 50;
    pub const MinimumPeriod: u64 = 0;
    pub const OracleBond: Balance = 100;
    pub const PmPalletId: PalletId = PalletId(*b"test/prm");
    pub const ReportingPeriod: BlockNumber = 10;
    pub const SharesPalletId: PalletId = PalletId(*b"test/sha");
    pub const SwapsPalletId: PalletId = PalletId(*b"test/swa");
    pub const ValidityBond: Balance = 200;
    pub DustAccount: AccountIdTest = PalletId(*b"orml/dst").into_account();
}

parameter_type_with_key! {
  pub ExistentialDeposits: |_currency_id: CurrencyId| -> Balance {
    Default::default()
  };
}

construct_runtime!(
    pub enum Runtime
    where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        Balances: pallet_balances::{Call, Config<T>, Event<T>, Pallet, Storage},
        Currency: orml_currencies::{Call, Event<T>, Pallet, Storage},
        PredictionMarkets: prediction_markets::{Event<T>, Pallet, Storage},
        Tokens: orml_tokens::{Config<T>, Event<T>, Pallet, Storage},
        Swaps: zrml_swaps::{Call, Event<T>, Pallet},
        System: frame_system::{Config, Event<T>, Pallet, Storage},
        Timestamp: pallet_timestamp::{Pallet},
    }
);

impl crate::Config for Runtime {
    type AdvisoryBond = AdvisoryBond;
    type ApprovalOrigin = EnsureSignedBy<Sudo, AccountIdTest>;
    type Currency = Balances;
    type DisputeBond = DisputeBond;
    type DisputeFactor = DisputeFactor;
    type DisputePeriod = DisputePeriod;
    type Event = Event;
    type MarketId = MarketId;
    type MaxCategories = MaxCategories;
    type MaxDisputes = MaxDisputes;
    type PalletId = PmPalletId;
    type OracleBond = OracleBond;
    type ReportingPeriod = ReportingPeriod;
    type Shares = Tokens;
    type Slash = ();
    type Swap = Swaps;
    type Timestamp = Timestamp;
    type ValidityBond = ValidityBond;
    type WeightInfo = prediction_markets::weights::WeightInfo<Runtime>;
}

impl frame_system::Config for Runtime {
    type AccountData = pallet_balances::AccountData<Balance>;
    type AccountId = AccountIdTest;
    type BaseCallFilter = ();
    type BlockHashCount = BlockHashCount;
    type BlockLength = ();
    type BlockNumber = BlockNumber;
    type BlockWeights = ();
    type Call = Call;
    type DbWeight = ();
    type Event = Event;
    type Hash = Hash;
    type Hashing = BlakeTwo256;
    type Header = Header;
    type Index = Index;
    type Lookup = IdentityLookup<Self::AccountId>;
    type OnKilledAccount = ();
    type OnNewAccount = ();
    type OnSetCode = ();
    type Origin = Origin;
    type PalletInfo = PalletInfo;
    type SS58Prefix = ();
    type SystemWeightInfo = ();
    type Version = ();
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
    type OnDust = orml_tokens::TransferDust<Runtime, DustAccount>;
    type WeightInfo = ();
}

impl pallet_balances::Config for Runtime {
    type AccountStore = System;
    type Balance = Balance;
    type DustRemoval = ();
    type Event = Event;
    type ExistentialDeposit = ExistentialDeposit;
    type MaxLocks = ();
    type WeightInfo = ();
}

impl pallet_timestamp::Config for Runtime {
    type MinimumPeriod = MinimumPeriod;
    type Moment = u64;
    type OnTimestampSet = ();
    type WeightInfo = ();
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
    type PalletId = SwapsPalletId;
    type Shares = Currency;
    type WeightInfo = zrml_swaps::weights::WeightInfo<Runtime>;
}

pub struct ExtBuilder {
    balances: Vec<(AccountIdTest, Balance)>,
}

impl Default for ExtBuilder {
    fn default() -> Self {
        Self {
            balances: vec![
                (ALICE, 1_000 * BASE),
                (BOB, 1_000 * BASE),
                (CHARLIE, 1_000 * BASE),
                (DAVE, 1_000 * BASE),
                (EVE, 1_000 * BASE),
            ],
        }
    }
}

impl ExtBuilder {
    pub fn build(self) -> sp_io::TestExternalities {
        let mut t = frame_system::GenesisConfig::default()
            .build_storage::<Runtime>()
            .unwrap();

        pallet_balances::GenesisConfig::<Runtime> {
            balances: self.balances,
        }
        .assimilate_storage(&mut t)
        .unwrap();

        t.into()
    }
}

pub fn run_to_block(n: BlockNumber) {
    while System::block_number() < n {
        Balances::on_finalize(System::block_number());
        PredictionMarkets::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);
        System::on_initialize(System::block_number());
        PredictionMarkets::on_initialize(System::block_number());
        Balances::on_initialize(System::block_number());
    }
}

sp_api::mock_impl_runtime_apis! {
    impl zrml_prediction_markets_runtime_api::PredictionMarketsApi<Block, MarketId, Hash> for Runtime {
        fn market_outcome_share_id(_: MarketId, _: u16) -> Asset<MarketId> {
            Asset::PoolShare(SerdeWrapper(1))
        }
    }
}
