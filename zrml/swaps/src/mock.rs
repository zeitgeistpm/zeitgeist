use crate::{Module, Trait};
use sp_core::H256;
use frame_system::EnsureSignedBy;
use frame_support::{impl_outer_origin, parameter_types, ord_parameter_types, weights::Weight};
use frame_support::traits::{OnInitialize, OnFinalize};
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    testing::Header, Perbill, ModuleId,
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

impl zrml_shares::Trait for Test {
	type Event = ();
	type Balance = Balance;
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
}

impl Trait for Test {
    type Event = ();
    type Currency = Balances;
    type Shares = Shares;
    type ModuleId = SwapsModuleId;
    type ExitFee = ExitFee;
    type MaxInRatio = MaxInRatio;
	type MaxOutRatio = MaxOutRatio;
	type MinWeight = MinWeight;
	type MaxWeight = MaxWeight;
	type MaxAssets = MaxAssets;
}

pub type Balances = pallet_balances::Module<Test>;
pub type Shares = zrml_shares::Module<Test>;
pub type Swaps = Module<Test>;
pub type System = frame_system::Module<Test>;

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
