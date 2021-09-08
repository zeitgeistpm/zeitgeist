#![cfg(feature = "mock")]

use crate as zrml_swaps;
use frame_support::construct_runtime;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};
use substrate_fixed::{types::extra::U33, FixedI128, FixedU128};
use zeitgeist_primitives::{
    constants::{
        BalanceFractionalDecimals, BlockHashCount, DustAccountTest, ExistentialDeposit, ExistentialDeposits, ExitFee, GetNativeCurrencyId,
        LiquidityMiningPalletId, MaxAssets, MaxInRatio, MaxLocks, MaxOutRatio, MaxReserves,
        MaxTotalWeight, MaxWeight, MinLiquidity, MinWeight, MinimumPeriod, SwapsPalletId,
    },
    types::{
        AccountIdTest, Amount, Asset, Balance, BasicCurrencyAdapter, BlockNumber, BlockTest,
        CurrencyId, Hash, Index, MarketId, Moment, PoolId, SerdeWrapper, UncheckedExtrinsicTest,
    },
};
use zrml_rikiddo::types::{EmaMarketVolume, FeeSigmoid, RikiddoSigmoidMV};

pub const ALICE: AccountIdTest = 0;
pub const BOB: AccountIdTest = 1;
pub const CHARLIE: AccountIdTest = 2;
pub const DAVE: AccountIdTest = 3;
pub const EVE: AccountIdTest = 4;

pub type UncheckedExtrinsic = UncheckedExtrinsicTest<Runtime>;

construct_runtime!(
    pub enum Runtime
    where
        Block = BlockTest<Runtime>,
        NodeBlock = BlockTest<Runtime>,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        Balances: pallet_balances::{Call, Config<T>, Event<T>, Pallet, Storage},
        Currencies: orml_currencies::{Event<T>, Pallet},
        LiquidityMining: zrml_liquidity_mining::{Config<T>, Event<T>, Pallet},
        MarketCommons: zrml_market_commons::{Pallet, Storage},
        Rikiddo: zrml_rikiddo::{Pallet, Storage},
        Swaps: zrml_swaps::{Call, Event<T>, Pallet},
        System: frame_system::{Call, Config, Event<T>, Pallet, Storage},
        Timestamp: pallet_timestamp::{Pallet},
        Tokens: orml_tokens::{Config<T>, Event<T>, Pallet, Storage},
    }
);

impl crate::Config for Runtime {
    type Event = Event;
    type ExitFee = ExitFee;
    type FixedTypeU = FixedU128<U33>;
    type FixedTypeS = FixedI128<U33>;
    type LiquidityMining = LiquidityMining;
    type MarketId = MarketId;
    type MaxAssets = MaxAssets;
    type MaxInRatio = MaxInRatio;
    type MaxOutRatio = MaxOutRatio;
    type MaxTotalWeight = MaxTotalWeight;
    type MaxWeight = MaxWeight;
    type MinLiquidity = MinLiquidity;
    type MinWeight = MinWeight;
    type PalletId = SwapsPalletId;
    type Rikiddo = Rikiddo;
    type Shares = Currencies;
    type WeightInfo = zrml_swaps::weights::WeightInfo<Runtime>;
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
    type NativeCurrency = BasicCurrencyAdapter<Runtime, Balances>;
    type WeightInfo = ();
}

impl orml_tokens::Config for Runtime {
    type Amount = Amount;
    type Balance = Balance;
    type CurrencyId = CurrencyId;
    type DustRemovalWhitelist = ();
    type Event = Event;
    type ExistentialDeposits = ExistentialDeposits;
    type MaxLocks = MaxLocks;
    type OnDust = orml_tokens::TransferDust<Runtime, DustAccountTest>;
    type WeightInfo = ();
}

impl pallet_balances::Config for Runtime {
    type AccountStore = System;
    type Balance = Balance;
    type DustRemoval = ();
    type Event = Event;
    type ExistentialDeposit = ExistentialDeposit;
    type MaxLocks = MaxLocks;
    type MaxReserves = MaxReserves;
    type ReserveIdentifier = [u8; 8];
    type WeightInfo = ();
}

impl zrml_liquidity_mining::Config for Runtime {
    type Event = Event;
    type MarketCommons = MarketCommons;
    type MarketId = MarketId;
    type PalletId = LiquidityMiningPalletId;
    type WeightInfo = zrml_liquidity_mining::weights::WeightInfo<Runtime>;
}

impl zrml_market_commons::Config for Runtime {
    type Currency = Balances;
    type MarketId = MarketId;
    type Timestamp = Timestamp;
}

impl zrml_rikiddo::Config for Runtime {
    type Timestamp = Timestamp;
    type Balance = Balance;
    type FixedTypeU = FixedU128<U33>;
    type FixedTypeS = FixedI128<U33>;
    type BalanceFractionalDecimals = BalanceFractionalDecimals;
    type PoolId = PoolId;
    type Rikiddo = RikiddoSigmoidMV<
        Self::FixedTypeU,
        Self::FixedTypeS,
        FeeSigmoid<Self::FixedTypeS>,
        EmaMarketVolume<Self::FixedTypeU>,
    >;
}

impl pallet_timestamp::Config for Runtime {
    type MinimumPeriod = MinimumPeriod;
    type Moment = Moment;
    type OnTimestampSet = ();
    type WeightInfo = ();
}

pub struct ExtBuilder {
    balances: Vec<(AccountIdTest, Balance)>,
}

impl Default for ExtBuilder {
    fn default() -> Self {
        Self {
            balances: vec![(ALICE, BASE), (BOB, BASE), (CHARLIE, BASE), (DAVE, BASE), (EVE, BASE)],
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

sp_api::mock_impl_runtime_apis! {
    impl zrml_swaps_runtime_api::SwapsApi<BlockTest<Runtime>, PoolId, AccountIdTest, Balance, MarketId>
      for Runtime
    {
        fn get_spot_price(
            pool_id: PoolId,
            asset_in: Asset<MarketId>,
            asset_out: Asset<MarketId>,
        ) -> SerdeWrapper<Balance> {
            SerdeWrapper(Swaps::get_spot_price(pool_id, asset_in, asset_out).ok().unwrap_or(0))
        }

        fn pool_account_id(pool_id: PoolId) -> AccountIdTest {
            Swaps::pool_account_id(pool_id)
        }

        fn pool_shares_id(pool_id: PoolId) -> Asset<SerdeWrapper<MarketId>> {
            Asset::PoolShare(SerdeWrapper(pool_id))
        }
    }
}
