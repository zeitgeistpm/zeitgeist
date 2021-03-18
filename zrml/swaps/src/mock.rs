use crate as zrml_swaps;
use crate::{Module, Trait, BASE};
use frame_support::{impl_outer_event, impl_outer_origin, parameter_types, weights::Weight};
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    FixedPointNumber, FixedU128, ModuleId, Perbill,
};

pub type AccountId = u128;
pub type Balance = u128;
pub type BlockNumber = u64;

pub const ALICE: AccountId = 0;
pub const BOB: AccountId = 1;
pub const CHARLIE: AccountId = 2;
pub const DAVE: AccountId = 3;
pub const EVE: AccountId = 4;

pub const FIXEDU128_0_5: FixedU128 = FixedU128::from_inner(BASE / 2);
pub const FIXEDU128_0: FixedU128 = from_integral(0);
pub const FIXEDU128_1: FixedU128 = from_integral(1);
pub const FIXEDU128_100: FixedU128 = from_integral(100);
pub const FIXEDU128_2: FixedU128 = from_integral(2);
pub const FIXEDU128_25: FixedU128 = from_integral(25);
pub const FIXEDU128_3: FixedU128 = from_integral(3);
pub const FIXEDU128_5: FixedU128 = from_integral(5);
pub const FIXEDU128_50: FixedU128 = from_integral(50);

impl_outer_event! {
    pub enum TestEvent for Test {
        frame_system<T>,
        pallet_balances<T>,
        zrml_shares<T>,
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

impl zrml_shares::Trait for Test {
    type Currency = Currency;
    type Event = TestEvent;
    type ModuleId = SharesModuleId;
}

parameter_types! {
    pub const SwapsModuleId: ModuleId = ModuleId(*b"test/swa");
    pub const ExitFee: FixedU128 = FIXEDU128_0;
    pub const MaxInRatio: FixedU128 = FIXEDU128_0_5;
    pub const MaxOutRatio: FixedU128 = from_parts(0, (1 / 3) + 1);
    pub const MinWeight: FixedU128 = FIXEDU128_1;
    pub const MaxWeight: FixedU128 = FIXEDU128_50;
    pub const MaxTotalWeight: FixedU128 = FIXEDU128_50;
    pub const MaxAssets: usize = 8;
    pub const MinLiquidity: FixedU128 = FIXEDU128_100;
}

impl Trait for Test {
    type Event = TestEvent;
    type Currency = Currency;
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

pub type Currency = pallet_balances::Module<Test>;
pub type Shares = zrml_shares::Module<Test>;
pub type Swaps = Module<Test>;

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

// Creates a `FixedU128` instance from the integer part without overflow.
#[inline]
pub const fn from_integral(integral: u64) -> FixedU128 {
    let integral_u128 = integral as u128;
    FixedU128::from_inner(integral_u128.wrapping_mul(<FixedU128 as FixedPointNumber>::DIV))
}

// Creates a `FixedU128` instance from the integer and decimal parts without overflow.
#[inline]
pub const fn from_parts(integral: u64, decimal: u32) -> FixedU128 {
    let integral_u128 = integral as u128;
    let decimal_u128 = decimal as _;
    let n = integral_u128.wrapping_mul(<FixedU128 as FixedPointNumber>::DIV);
    FixedU128::from_inner(n.wrapping_add(decimal_u128))
}
