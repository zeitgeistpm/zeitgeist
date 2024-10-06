#![cfg(feature = "mock")]

use crate as zrml_combinatorial_tokens;
use crate::types::CryptographicIdManager;
use frame_support::{construct_runtime, traits::Everything, Blake2_256};
use frame_system::mocking::MockBlock;
use sp_runtime::traits::{BlakeTwo256, ConstU32, IdentityLookup};
use zeitgeist_primitives::{
    constants::mock::{
        BlockHashCount, ExistentialDeposit, ExistentialDeposits, GetNativeCurrencyId, MaxLocks,
        MaxReserves, MinimumPeriod, CombinatorialTokensPalletId,
    },
    types::{
        AccountIdTest, Amount, Balance, BasicCurrencyAdapter, CurrencyId,
        MarketId, Moment, Hash
    },
};

construct_runtime! {
    pub enum Runtime {
        CombinatorialTokens: zrml_combinatorial_tokens,
        AssetManager: orml_currencies,
        Balances: pallet_balances,
        MarketCommons: zrml_market_commons,
        System: frame_system,
        Timestamp: pallet_timestamp,
        Tokens: orml_tokens,
    }
}

impl zrml_combinatorial_tokens::Config for Runtime {
    type CombinatorialIdManager = CryptographicIdManager<MarketId, Blake2_256>;
    type MarketCommons = MarketCommons;
    type MultiCurrency = AssetManager;
    type PalletId = CombinatorialTokensPalletId;
    type RuntimeEvent = RuntimeEvent;
}

impl orml_currencies::Config for Runtime {
    type GetNativeCurrencyId = GetNativeCurrencyId;
    type MultiCurrency = Tokens;
    type NativeCurrency = BasicCurrencyAdapter<Runtime, Balances>;
    type WeightInfo = ();
}

impl pallet_balances::Config for Runtime {
    type AccountStore = System;
    type Balance = Balance;
    type DustRemoval = ();
    type FreezeIdentifier = ();
    type RuntimeHoldReason = ();
    type RuntimeEvent = RuntimeEvent;
    type ExistentialDeposit = ExistentialDeposit;
    type MaxHolds = ();
    type MaxFreezes = ();
    type MaxLocks = MaxLocks;
    type MaxReserves = MaxReserves;
    type ReserveIdentifier = [u8; 8];
    type WeightInfo = ();
}

impl zrml_market_commons::Config for Runtime {
    type Balance = Balance;
    type MarketId = MarketId;
    type Timestamp = Timestamp;
}

impl frame_system::Config for Runtime {
    type AccountData = pallet_balances::AccountData<Balance>;
    type AccountId = AccountIdTest;
    type BaseCallFilter = Everything;
    type Block = MockBlock<Runtime>;
    type BlockHashCount = BlockHashCount;
    type BlockLength = ();
    type BlockWeights = ();
    type RuntimeCall = RuntimeCall;
    type DbWeight = ();
    type RuntimeEvent = RuntimeEvent;
    type Hash = Hash;
    type Hashing = BlakeTwo256;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Nonce = u64;
    type MaxConsumers = ConstU32<16>;
    type OnKilledAccount = ();
    type OnNewAccount = ();
    type RuntimeOrigin = RuntimeOrigin;
    type PalletInfo = PalletInfo;
    type SS58Prefix = ();
    type SystemWeightInfo = ();
    type Version = ();
    type OnSetCode = ();
}

impl pallet_timestamp::Config for Runtime {
    type MinimumPeriod = MinimumPeriod;
    type Moment = Moment;
    type OnTimestampSet = ();
    type WeightInfo = ();
}

impl orml_tokens::Config for Runtime {
    type Amount = Amount;
    type Balance = Balance;
    type CurrencyId = CurrencyId;
    type DustRemovalWhitelist = Everything;
    type RuntimeEvent = RuntimeEvent;
    type ExistentialDeposits = ExistentialDeposits;
    type MaxLocks = MaxLocks;
    type MaxReserves = MaxReserves;
    type CurrencyHooks = ();
    type ReserveIdentifier = [u8; 8];
    type WeightInfo = ();
}
