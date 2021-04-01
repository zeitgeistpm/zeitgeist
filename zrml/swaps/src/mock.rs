use crate as zrml_swaps;
use crate::{Module, Trait, BASE};
use frame_support::{impl_outer_event, impl_outer_origin, parameter_types, weights::Weight};
use orml_currencies::BasicCurrencyAdapter;
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    ModuleId, Perbill,
};
use zeitgeist_primitives::Asset;

pub type AccountId = u128;
pub type Balance = u128;
pub type BlockNumber = u64;
pub type MarketId = u128;

pub const ALICE: AccountId = 0;
pub const BOB: AccountId = 1;
pub const CHARLIE: AccountId = 2;
pub const DAVE: AccountId = 3;
pub const EVE: AccountId = 4;

impl_outer_event! {
    pub enum TestEvent for Test {
        frame_system<T>,
        orml_currencies<T>,
        orml_tokens<T>,
        pallet_balances<T>,
        zrml_swaps<T>,
    }
}

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
    type Event = TestEvent;
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
    type Event = TestEvent;
    type Balance = Balance;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = frame_system::Module<Test>;
    type MaxLocks = MaxLocks;
    type WeightInfo = ();
}

parameter_types! {
    pub const SharesModuleId: ModuleId = ModuleId(*b"test/sha");
}

impl orml_tokens::Trait for Test {
    type Amount = i128;
    type Balance = Balance;
    type CurrencyId = Asset<H256, MarketId>;
    type Event = TestEvent;
    type OnReceived = ();
    type WeightInfo = ();
}

parameter_types! {
    pub const GetNativeCurrencyId: Asset<H256, MarketId> = Asset::Ztg;
}

impl orml_currencies::Trait for Test {
    type Event = TestEvent;
    type MultiCurrency = Tokens;
    type NativeCurrency = AdaptedBasicCurrency;
    type GetNativeCurrencyId = GetNativeCurrencyId;
    type WeightInfo = ();
}

pub type AdaptedBasicCurrency = BasicCurrencyAdapter<Test, Currency, i128, u128>;

parameter_types! {
    pub const SwapsModuleId: ModuleId = ModuleId(*b"test/swa");
    pub const ExitFee: Balance = 0;
    pub const MaxInRatio: Balance = BASE / 2;
    pub const MaxOutRatio: Balance = (BASE / 3) + 1;
    pub const MinWeight: Balance = BASE;
    pub const MaxWeight: Balance = 50 * BASE;
    pub const MaxTotalWeight: Balance = 50 * BASE;
    pub const MaxAssets: usize = 8;
    pub const MinLiquidity: Balance = 100 * BASE;
}

impl Trait for Test {
    type Event = TestEvent;
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
    type Shares = Shares;
}

pub type Currency = pallet_balances::Module<Test>;
pub type Shares = orml_currencies::Module<Test>;
pub type Swaps = Module<Test>;
pub type Tokens = orml_tokens::Module<Test>;

pub struct ExtBuilder {
    balances: Vec<(AccountId, Balance)>,
}

impl Default for ExtBuilder {
    fn default() -> Self {
        Self {
            balances: vec![
                (ALICE, 1_000),
                (BOB, 1_000),
                (CHARLIE, 1_000),
                (DAVE, 1_000),
                (EVE, 1_000),
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
