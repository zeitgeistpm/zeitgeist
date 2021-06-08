#![cfg(test)]
use crate as lsd_lmsr;
use frame_support::{construct_runtime, parameter_types, PalletId};
use sp_runtime::{
    testing::Header,
    traits::{AccountIdConversion, BlakeTwo256, IdentityLookup},
};
use zeitgeist_primitives::{
    constants::{BASE, BLOCK_HASH_COUNT},
    types::{
        AccountIdTest, Asset, Balance, BlockNumber, BlockTest, Hash, Index, MarketId,
        UncheckedExtrinsicTest,
    },
};

pub const ALICE: AccountIdTest = 0;
pub const BOB: AccountIdTest = 1;
pub const CHARLIE: AccountIdTest = 2;
pub const DAVE: AccountIdTest = 3;
pub const EVE: AccountIdTest = 4;
pub const FRED: AccountIdTest = 5;

pub type Block = BlockTest<Runtime>;
pub type UncheckedExtrinsic = UncheckedExtrinsicTest<Runtime>;

parameter_types! {
    pub const BlockHashCount: u64 = BLOCK_HASH_COUNT;
    pub const DisputeBond: Balance = 100;
    pub const DisputeFactor: Balance = 25;
    pub const DisputePeriod: BlockNumber = 10;
    pub const ExistentialDeposit: u64 = 1;
    pub const GetNativeCurrencyId: Asset<MarketId> = Asset::Ztg;
    pub const MinimumPeriod: u64 = 0;
    pub const OracleBond: Balance = 100;
    pub const CourtPalletId: PalletId = PalletId(*b"test/crt");
    pub const SwapsPalletId: PalletId = PalletId(*b"test/swa");
    pub const ValidityBond: Balance = 200;
    pub DustAccount: AccountIdTest = PalletId(*b"orml/dst").into_account();
}

construct_runtime!(
    pub enum Runtime
    where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        Balances: pallet_balances::{Call, Config<T>, Event<T>, Pallet, Storage},
        System: frame_system::{Config, Event<T>, Pallet, Storage},
        Timestamp: pallet_timestamp::{Pallet},
        LsdLmsr: lsd_lmsr::{Pallet, Storage},
    }
);

impl crate::Config for Runtime {
    type Timestamp = Timestamp;
    type Asset = Asset<MarketId>;
    type Balance = Balance;
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
                (FRED, 1_000 * BASE),
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
