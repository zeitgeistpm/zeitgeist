#![cfg(feature = "mock")]

use crate as zrml_swaps;
use frame_support::{construct_runtime, parameter_types, PalletId};
use orml_currencies::BasicCurrencyAdapter;
use orml_traits::parameter_type_with_key;
use sp_runtime::{
    testing::Header,
    traits::{AccountIdConversion, BlakeTwo256, IdentityLookup},
};
use zeitgeist_primitives::{
    constants::{
        ExitFee, LiquidityMiningPalletId, MaxAssets, MaxInRatio, MaxLocks, MaxOutRatio,
        MaxReserves, MaxTotalWeight, MaxWeight, MinLiquidity, MinWeight, SwapsPalletId,
        BLOCK_HASH_COUNT,
    },
    types::{
        AccountIdTest, Amount, Asset, Balance, BlockNumber, BlockTest, CurrencyId, Hash, Index,
        MarketId, PoolId, SerdeWrapper, UncheckedExtrinsicTest,
    },
};

// parameter_types imported from zeitgeist_primitives
parameter_types! {
    pub const BlockHashCount: u64 = BLOCK_HASH_COUNT;
    pub const ExistentialDeposit: u32 = 1;
    pub const GetNativeCurrencyId: CurrencyId = Asset::Ztg;
    pub DustAccount: AccountIdTest = PalletId(*b"orml/dst").into_account();
}

parameter_type_with_key! {
  pub ExistentialDeposits: |_currency_id: CurrencyId| -> Balance {
    Default::default()
  };
}

pub const ALICE: AccountIdTest = 0;
pub const BOB: AccountIdTest = 1;
pub const CHARLIE: AccountIdTest = 2;
pub const DAVE: AccountIdTest = 3;
pub const EVE: AccountIdTest = 4;

pub type AdaptedBasicCurrency = BasicCurrencyAdapter<Runtime, Balances, i128, u128>;
pub type Block = BlockTest<Runtime>;
pub type UncheckedExtrinsic = UncheckedExtrinsicTest<Runtime>;

construct_runtime!(
    pub enum Runtime
    where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        Balances: pallet_balances::{Call, Config<T>, Event<T>, Pallet, Storage},
        Currencies: orml_currencies::{Event<T>, Pallet},
        LiquidityMining: zrml_liquidity_mining::{Config<T>, Event<T>, Pallet},
        Swaps: zrml_swaps::{Call, Event<T>, Pallet},
        System: frame_system::{Call, Config, Event<T>, Pallet, Storage},
        Tokens: orml_tokens::{Config<T>, Event<T>, Pallet, Storage},
    }
);

impl crate::Config for Runtime {
    type Event = Event;
    type ExitFee = ExitFee;
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
    type NativeCurrency = AdaptedBasicCurrency;
    type WeightInfo = ();
}

impl orml_tokens::Config for Runtime {
    type Amount = Amount;
    type Balance = Balance;
    type CurrencyId = CurrencyId;
    type Event = Event;
    type ExistentialDeposits = ExistentialDeposits;
    type MaxLocks = MaxLocks;
    type OnDust = orml_tokens::TransferDust<Runtime, DustAccount>;
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
    type Currency = Balances;
    type Event = Event;
    type MarketId = MarketId;
    type PalletId = LiquidityMiningPalletId;
    type WeightInfo = zrml_liquidity_mining::weights::WeightInfo<Runtime>;
}

pub struct ExtBuilder {
    balances: Vec<(AccountIdTest, Balance)>,
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
        let mut t = frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap();

        pallet_balances::GenesisConfig::<Runtime> { balances: self.balances }
            .assimilate_storage(&mut t)
            .unwrap();

        t.into()
    }
}

sp_api::mock_impl_runtime_apis! {
    impl zrml_swaps_runtime_api::SwapsApi<Block, PoolId, AccountIdTest, Balance, MarketId>
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
