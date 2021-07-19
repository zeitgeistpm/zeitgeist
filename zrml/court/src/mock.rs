#![cfg(test)]

use crate as zrml_court;
use frame_support::{construct_runtime, parameter_types, PalletId};
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};
use zeitgeist_primitives::{
    constants::{MaxReserves, BASE, BLOCK_HASH_COUNT},
    types::{
        AccountIdTest, Balance, BlockNumber, BlockTest, Hash, Index, MarketId,
        UncheckedExtrinsicTest,
    },
};

pub const ALICE: AccountIdTest = 0;
pub const BOB: AccountIdTest = 1;

type Block = BlockTest<Runtime>;
type UncheckedExtrinsic = UncheckedExtrinsicTest<Runtime>;

parameter_types! {
    pub const BlockHashCount: u64 = BLOCK_HASH_COUNT;
    pub const LmPalletId: PalletId = PalletId(*b"test/lmg");
    pub const StakeWeight: u128 = 2 * BASE;
}

construct_runtime!(
    pub enum Runtime
    where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        Balances: pallet_balances::{Call, Config<T>, Event<T>, Pallet, Storage},
        Court: zrml_court::{Event<T>, Pallet, Storage},
        MarketCommons: zrml_market_commons::{Pallet, Storage},
        System: frame_system::{Call, Config, Event<T>, Pallet, Storage},
    }
);

impl crate::Config for Runtime {
    type Event = ();
    type StakeWeight = StakeWeight;
    type MarketCommons = MarketCommons;
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
    type Event = ();
    type Hash = Hash;
    type Hashing = BlakeTwo256;
    type Header = Header;
    type Index = Index;
    type Lookup = IdentityLookup<Self::AccountId>;
    type OnKilledAccount = ();
    type OnNewAccount = ();
    type Origin = Origin;
    type PalletInfo = PalletInfo;
    type SS58Prefix = ();
    type SystemWeightInfo = ();
    type Version = ();
    type OnSetCode = ();
}

impl pallet_balances::Config for Runtime {
    type AccountStore = System;
    type Balance = Balance;
    type DustRemoval = ();
    type Event = ();
    type ExistentialDeposit = ();
    type MaxLocks = ();
    type MaxReserves = MaxReserves;
    type ReserveIdentifier = [u8; 4];
    type WeightInfo = ();
}

impl zrml_market_commons::Config for Runtime {
    type Currency = Balances;
    type MarketId = MarketId;
}

pub struct ExtBuilder {
    balances: Vec<(AccountIdTest, Balance)>,
}

impl Default for ExtBuilder {
    fn default() -> Self {
        Self { balances: vec![(ALICE, 1_000 * BASE), (BOB, 1_000 * BASE)] }
    }
}

impl ExtBuilder {
    pub fn build(self) -> sp_io::TestExternalities {
        let mut t = frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap();

        pallet_balances::GenesisConfig::<Runtime> { balances: self.balances }
            .assimilate_storage(&mut t)
            .unwrap();

        t.into()
    }
}
