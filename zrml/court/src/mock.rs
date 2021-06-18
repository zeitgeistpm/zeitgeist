#![cfg(test)]

use crate as court;
use frame_support::{construct_runtime, parameter_types, PalletId};
use orml_traits::parameter_type_with_key;
use sp_runtime::{
    testing::Header,
    traits::{AccountIdConversion, BlakeTwo256, IdentityLookup},
};
use zeitgeist_primitives::{
    constants::{
        CourtPalletId, ExitFee, MaxAssets, MaxDisputes, MaxInRatio, MaxOutRatio, MaxTotalWeight,
        MaxWeight, MinLiquidity, MinWeight, SwapsPalletId, BASE, BLOCK_HASH_COUNT,
    },
    types::{
        AccountIdTest, Amount, Asset, Balance, BlockNumber, BlockTest, CurrencyId, Hash, Index,
        MarketId, UncheckedExtrinsicTest,
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
pub type AdaptedBasicCurrency =
    orml_currencies::BasicCurrencyAdapter<Runtime, Balances, Amount, Balance>;

parameter_types! {
    pub const BlockHashCount: u64 = BLOCK_HASH_COUNT;
    pub const DisputeBond: Balance = 100;
    pub const DisputeFactor: Balance = 25;
    pub const DisputePeriod: BlockNumber = 10;
    pub const ExistentialDeposit: u64 = 1;
    pub const GetNativeCurrencyId: Asset<MarketId> = Asset::Ztg;
    pub const MinimumPeriod: u64 = 0;
    pub const OracleBond: Balance = 100;
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
        Court: court::{Event<T>, Pallet, Storage},
        Currency: orml_currencies::{Call, Event<T>, Pallet, Storage},
        MarketCommons: zrml_market_commons::{Pallet, Storage},
        Swaps: zrml_swaps::{Call, Event<T>, Pallet},
        System: frame_system::{Config, Event<T>, Pallet, Storage},
        Timestamp: pallet_timestamp::{Pallet},
        Tokens: orml_tokens::{Config<T>, Event<T>, Pallet, Storage},
    }
);

impl crate::Config for Runtime {
    type Currency = Balances;
    type DisputeBond = DisputeBond;
    type DisputeFactor = DisputeFactor;
    type DisputePeriod = DisputePeriod;
    type Event = Event;
    type MarketCommons = MarketCommons;
    type MarketId = MarketId;
    type MaxDisputes = MaxDisputes;
    type OracleBond = OracleBond;
    type PalletId = CourtPalletId;
    type Shares = Tokens;
    type Swaps = Swaps;
    type ValidityBond = ValidityBond;
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
    type MaxLocks = ();
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

impl zrml_market_commons::Config for Runtime {
    type MarketId = MarketId;
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
                (FRED, 1_000 * BASE),
            ],
        }
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
