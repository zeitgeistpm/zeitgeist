use crate::{Module, Trait};
use frame_support::traits::{OnFinalize, OnInitialize};
use frame_support::{impl_outer_origin, ord_parameter_types, parameter_types, weights::Weight};
use frame_system::EnsureSignedBy;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    ModuleId, Perbill,
};

pub type AccountId = u128;
pub type Balance = u128;
pub type BlockNumber = u64;
pub type MarketId = u128;

pub const ALICE: AccountId = 0;
pub const BOB: AccountId = 1;
pub const CHARLIE: AccountId = 2;
pub const DAVE: AccountId = 3;
pub const EVE: AccountId = 4;
pub const SUDO: AccountId = 69;

// BASE is used as the number of decimals in order to set constants elsewhere.
pub const BASE: Balance = 10_000_000_000;

impl_outer_origin! {
    pub enum Origin for Test {}
}

#[derive(Clone, Eq, PartialEq)]
pub struct Test;

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: Weight = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
}

impl frame_system::Trait for Test {
    type BaseCallFilter = ();
    type Origin = Origin;
    type Call = ();
    type Index = u64;
    type BlockNumber = BlockNumber;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = ();
    type BlockHashCount = BlockHashCount;
    type MaximumBlockWeight = MaximumBlockWeight;
    type DbWeight = ();
    type BlockExecutionWeight = ();
    type ExtrinsicBaseWeight = ();
    type MaximumExtrinsicWeight = MaximumBlockWeight;
    type MaximumBlockLength = MaximumBlockLength;
    type AvailableBlockRatio = AvailableBlockRatio;
    type Version = ();
    type PalletInfo = ();
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
}

parameter_types! {
    pub const ExistentialDeposit: u64 = 1;
    pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Trait for Test {
    type Event = ();
    type Balance = Balance;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = frame_system::Module<Test>;
    type MaxLocks = MaxLocks;
    type WeightInfo = ();
}

parameter_types! {
    pub const MinimumPeriod: u64 = 0;
}

impl pallet_timestamp::Trait for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
    type WeightInfo = ();
}

parameter_types! {
    pub const SharesModuleId: ModuleId = ModuleId(*b"test/sha");
}

impl zrml_shares::Trait for Test {
    type Event = ();
    type Balance = Balance;
    type Currency = Balances;
    type ModuleId = SharesModuleId;
}

parameter_types! {
    pub const SwapsModuleId: ModuleId = ModuleId(*b"test/swa");
    pub const ExitFee: Balance = 0;
    pub const MaxInRatio: Balance = BASE / 2;
    pub const MaxOutRatio: Balance = (BASE / 3) + 1;
    pub const MinWeight: Balance = BASE;
    pub const MaxWeight: Balance = 50 * BASE;
    pub const MaxTotalWeight: Balance = 50 * BASE;
    pub const MaxAssets: Balance = 8;
    pub const MinLiquidity: Balance = 100 * BASE;
}

impl zrml_swaps::Trait for Test {
    type Event = ();
    type Currency = Balances;
    type Shares = Shares;
    type ModuleId = SwapsModuleId;
    type ExitFee = ExitFee;
    type MaxInRatio = MaxInRatio;
    type MaxOutRatio = MaxOutRatio;
    type MinWeight = MinWeight;
    type MaxWeight = MaxWeight;
    type MaxTotalWeight = MaxTotalWeight;
    type MaxAssets = MaxAssets;
    type MinLiquidity = MinLiquidity;
}

parameter_types! {
    pub const PmModuleId: ModuleId = ModuleId(*b"test/prm");
    pub const ReportingPeriod: BlockNumber = 10;
    pub const DisputePeriod: BlockNumber = 10;
    pub const DisputeBond: Balance = 100;
    pub const DisputeFactor: Balance = 25;
    pub const MaxDisputes: u16 = 5;
    pub const OracleBond: Balance = 100;
    pub const ValidityBond: Balance = 200;
    pub const AdvisoryBond: Balance = 50;
}

ord_parameter_types! {
    pub const Sudo: AccountId = 69;
}

impl Trait for Test {
    type Event = ();
    type Currency = Balances;
    type Shares = Shares;
    type MarketId = MarketId;
    type ModuleId = PmModuleId;
    type ReportingPeriod = ReportingPeriod;
    type DisputePeriod = DisputePeriod;
    type DisputeBond = DisputeBond;
    type DisputeFactor = DisputeFactor;
    type MaxDisputes = MaxDisputes;
    type ValidityBond = ValidityBond;
    type AdvisoryBond = AdvisoryBond;
    type OracleBond = OracleBond;
    type ApprovalOrigin = EnsureSignedBy<Sudo, AccountId>;
    type Slash = ();
    type Swap = Swaps;
}

pub type Balances = pallet_balances::Module<Test>;
pub type PredictionMarkets = Module<Test>;
pub type Timestamp = pallet_timestamp::Module<Test>;
pub type Shares = zrml_shares::Module<Test>;
pub type Swaps = zrml_swaps::Module<Test>;
pub type System = frame_system::Module<Test>;

pub struct ExtBuilder {
    balances: Vec<(AccountId, Balance)>,
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
            .build_storage::<Test>()
            .unwrap();

        pallet_balances::GenesisConfig::<Test> {
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
