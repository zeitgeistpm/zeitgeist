use crate::{Module, Trait};
use sp_core::H256;
use frame_support::{impl_outer_origin, parameter_types, weights::Weight};
use sp_runtime::{
    ModuleId,
    traits::{BlakeTwo256, IdentityLookup},
    testing::Header, Perbill,
};

pub type AccountId = u128;
pub type Amount = i128;
pub type Balance = u128;
pub type CurrencyId = u128;

pub const ALICE: AccountId = 0;
pub const BOB: AccountId = 1;

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
	type BlockNumber = u64;
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
	type ModuleToIndex = ();
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
}

impl orml_tokens::Trait for Test {
    type Event = ();
    type Balance = Balance;
    type Amount = Amount;
    type CurrencyId = CurrencyId;
    type OnReceived = ();
}

parameter_types! {
    pub const GetNativeCurrencyId: CurrencyId = 0;
    pub const CurveDeposit: Balance = 10;
    pub const BondingCurveModuleId: ModuleId = ModuleId(*b"mtg/bonc");
}

impl Trait for Test {
    type Event = ();
    type Currency = orml_tokens::Module<Test>;
    type GetNativeCurrencyId = GetNativeCurrencyId;
    type CurveDeposit = CurveDeposit;
    type ModuleId = BondingCurveModuleId;
}

pub type System = frame_system::Module<Test>;
pub type Tokens = orml_tokens::Module<Test>;
pub type BondingCurve = Module<Test>;

pub struct ExtBuilder {
	endowed_accounts: Vec<(AccountId, CurrencyId, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			endowed_accounts: vec![
				(ALICE, 0, 1_000),
				(BOB, 0, 1_000),

			],
		}
	}
}

impl ExtBuilder {
	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Test>()
			.unwrap();
		
		orml_tokens::GenesisConfig::<Test> {
			endowed_accounts: self.endowed_accounts,
		}
		.assimilate_storage(&mut t)
		.unwrap();

		t.into()
	}
}
