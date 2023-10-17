// Copyright 2022-2023 Forecasting Technologies LTD.
// Copyright 2021-2022 Zeitgeist PM LLC.
//
// This file is part of Zeitgeist.
//
// Zeitgeist is free software: you can redistribute it and/or modify it
// under the terms of the GNU General Public License as published by the
// Free Software Foundation, either version 3 of the License, or (at
// your option) any later version.
//
// Zeitgeist is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Zeitgeist. If not, see <https://www.gnu.org/licenses/>.
//
// This file incorporates work covered by the license above but
// published without copyright notice by Balancer Labs
// (<https://balancer.finance>, contact@balancer.finance) in the
// balancer-core repository
// <https://github.com/balancer-labs/balancer-core>.

#![cfg(feature = "mock")]
#![allow(
    // Mocks are only used for fuzzing and unit tests
    clippy::arithmetic_side_effects
)]

use crate as zrml_swaps;
use frame_support::{
    construct_runtime, parameter_types,
    traits::{Contains, Everything},
};
use orml_traits::parameter_type_with_key;
use sp_arithmetic::Perbill;
use sp_runtime::{
    testing::Header,
    traits::{AccountIdConversion, BlakeTwo256, IdentityLookup},
    DispatchError,
};
use substrate_fixed::{types::extra::U33, FixedI128, FixedU128};
use zeitgeist_primitives::{
    constants::mock::{
        BalanceFractionalDecimals, BlockHashCount, ExistentialDeposit, GetNativeCurrencyId,
        LiquidityMiningPalletId, MaxAssets, MaxInRatio, MaxLocks, MaxOutRatio, MaxReserves,
        MaxSwapFee, MaxTotalWeight, MaxWeight, MinAssets, MinSubsidy, MinWeight, MinimumPeriod,
        PmPalletId, SwapsPalletId, BASE,
    },
    types::{
        AccountIdTest, Amount, Asset, Balance, BasicCurrencyAdapter, BlockNumber, BlockTest,
        CurrencyId, Deadlines, Hash, Index, Market, MarketBonds, MarketCreation,
        MarketDisputeMechanism, MarketId, MarketPeriod, MarketStatus, MarketType, Moment, PoolId,
        ScoringRule, SerdeWrapper, UncheckedExtrinsicTest,
    },
};
use zrml_market_commons::MarketCommonsPalletApi;
use zrml_rikiddo::types::{EmaMarketVolume, FeeSigmoid, RikiddoSigmoidMV};

pub const ALICE: AccountIdTest = 0;
pub const BOB: AccountIdTest = 1;
pub const CHARLIE: AccountIdTest = 2;
pub const DAVE: AccountIdTest = 3;
pub const EVE: AccountIdTest = 4;

pub const ASSET_A: Asset<MarketId> = Asset::CategoricalOutcome(0, 65);
pub const ASSET_B: Asset<MarketId> = Asset::CategoricalOutcome(0, 66);
pub const ASSET_C: Asset<MarketId> = Asset::CategoricalOutcome(0, 67);
pub const ASSET_D: Asset<MarketId> = Asset::CategoricalOutcome(0, 68);
pub const ASSET_E: Asset<MarketId> = Asset::CategoricalOutcome(0, 69);

pub const ASSETS: [Asset<MarketId>; 4] = [ASSET_A, ASSET_B, ASSET_C, ASSET_D];
pub const BASE_ASSET: Asset<MarketId> = if let Some(asset) = ASSETS.last() {
    *asset
} else {
    panic!("Invalid asset vector");
};

pub const DEFAULT_MARKET_ID: MarketId = 0;
pub const DEFAULT_MARKET_ORACLE: AccountIdTest = DAVE;
pub const DEFAULT_MARKET_CREATOR: AccountIdTest = DAVE;

pub type UncheckedExtrinsic = UncheckedExtrinsicTest<Runtime>;

// Mocked exit fee for easier calculations
parameter_types! {
    pub storage ExitFeeMock: Balance = BASE / 10;
    pub const MinSubsidyPerAccount: Balance = BASE;
}

construct_runtime!(
    pub enum Runtime
    where
        Block = BlockTest<Runtime>,
        NodeBlock = BlockTest<Runtime>,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        Balances: pallet_balances::{Call, Config<T>, Event<T>, Pallet, Storage},
        Currencies: orml_currencies::{Pallet},
        LiquidityMining: zrml_liquidity_mining::{Config<T>, Event<T>, Pallet},
        MarketCommons: zrml_market_commons::{Pallet, Storage},
        RikiddoSigmoidFeeMarketEma: zrml_rikiddo::{Pallet, Storage},
        Swaps: zrml_swaps::{Call, Event<T>, Pallet},
        System: frame_system::{Call, Config, Event<T>, Pallet, Storage},
        Timestamp: pallet_timestamp::{Pallet},
        Tokens: orml_tokens::{Config<T>, Event<T>, Pallet, Storage},
    }
);

pub type AssetManager = Currencies;

impl crate::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type ExitFee = ExitFeeMock;
    type FixedTypeU = <Runtime as zrml_rikiddo::Config>::FixedTypeU;
    type FixedTypeS = <Runtime as zrml_rikiddo::Config>::FixedTypeS;
    type LiquidityMining = LiquidityMining;
    type MarketCommons = MarketCommons;
    type MaxAssets = MaxAssets;
    type MaxInRatio = MaxInRatio;
    type MaxOutRatio = MaxOutRatio;
    type MaxSwapFee = MaxSwapFee;
    type MaxTotalWeight = MaxTotalWeight;
    type MaxWeight = MaxWeight;
    type MinAssets = MinAssets;
    type MinSubsidy = MinSubsidy;
    type MinSubsidyPerAccount = MinSubsidyPerAccount;
    type MinWeight = MinWeight;
    type PalletId = SwapsPalletId;
    type RikiddoSigmoidFeeMarketEma = RikiddoSigmoidFeeMarketEma;
    type AssetManager = AssetManager;
    type WeightInfo = zrml_swaps::weights::WeightInfo<Runtime>;
}

impl frame_system::Config for Runtime {
    type AccountData = pallet_balances::AccountData<Balance>;
    type AccountId = AccountIdTest;
    type BaseCallFilter = Everything;
    type BlockHashCount = BlockHashCount;
    type BlockLength = ();
    type BlockNumber = BlockNumber;
    type BlockWeights = ();
    type RuntimeCall = RuntimeCall;
    type DbWeight = ();
    type RuntimeEvent = RuntimeEvent;
    type Hash = Hash;
    type Hashing = BlakeTwo256;
    type Header = Header;
    type Index = Index;
    type Lookup = IdentityLookup<Self::AccountId>;
    type MaxConsumers = frame_support::traits::ConstU32<16>;
    type OnKilledAccount = ();
    type OnNewAccount = ();
    type OnSetCode = ();
    type RuntimeOrigin = RuntimeOrigin;
    type PalletInfo = PalletInfo;
    type SS58Prefix = ();
    type SystemWeightInfo = ();
    type Version = ();
}

impl orml_currencies::Config for Runtime {
    type GetNativeCurrencyId = GetNativeCurrencyId;
    type MultiCurrency = Tokens;
    type NativeCurrency = BasicCurrencyAdapter<Runtime, Balances>;
    type WeightInfo = ();
}

parameter_type_with_key! {
    pub ExistentialDeposits: |currency_id: CurrencyId| -> Balance {
        match currency_id {
            &BASE_ASSET => ExistentialDeposit::get(),
            Asset::Ztg => ExistentialDeposit::get(),
            _ => 0,
        }
    };
}

pub struct DustRemovalWhitelist;

impl Contains<AccountIdTest> for DustRemovalWhitelist
where
    frame_support::PalletId: AccountIdConversion<AccountIdTest>,
{
    fn contains(ai: &AccountIdTest) -> bool {
        let pallets = vec![LiquidityMiningPalletId::get(), PmPalletId::get(), SwapsPalletId::get()];

        if let Some(pallet_id) = frame_support::PalletId::try_from_sub_account::<u128>(ai) {
            return pallets.contains(&pallet_id.0);
        }

        for pallet_id in pallets {
            let pallet_acc: AccountIdTest = pallet_id.into_account_truncating();

            if pallet_acc == *ai {
                return true;
            }
        }

        false
    }
}

impl orml_tokens::Config for Runtime {
    type Amount = Amount;
    type Balance = Balance;
    type CurrencyId = CurrencyId;
    type DustRemovalWhitelist = DustRemovalWhitelist;
    type RuntimeEvent = RuntimeEvent;
    type ExistentialDeposits = ExistentialDeposits;
    type MaxLocks = MaxLocks;
    type MaxReserves = MaxReserves;
    type CurrencyHooks = ();
    type ReserveIdentifier = [u8; 8];
    type WeightInfo = ();
}

impl pallet_balances::Config for Runtime {
    type AccountStore = System;
    type Balance = Balance;
    type DustRemoval = ();
    type RuntimeEvent = RuntimeEvent;
    type ExistentialDeposit = ExistentialDeposit;
    type MaxLocks = MaxLocks;
    type MaxReserves = MaxReserves;
    type ReserveIdentifier = [u8; 8];
    type WeightInfo = ();
}

impl zrml_liquidity_mining::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type MarketCommons = MarketCommons;
    type MarketId = MarketId;
    type PalletId = LiquidityMiningPalletId;
    type WeightInfo = zrml_liquidity_mining::weights::WeightInfo<Runtime>;
}

impl zrml_market_commons::Config for Runtime {
    type Balance = Balance;
    type MarketId = MarketId;
    type PredictionMarketsPalletId = PmPalletId;
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
        let mut storage =
            frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap();

        pallet_balances::GenesisConfig::<Runtime> { balances: self.balances }
            .assimilate_storage(&mut storage)
            .unwrap();

        let mut ext = sp_io::TestExternalities::from(storage);

        ext.execute_with(|| {
            MarketCommons::push_market(mock_market(4)).unwrap();
        });

        ext
    }
}

sp_api::mock_impl_runtime_apis! {
    impl zrml_swaps_runtime_api::SwapsApi<BlockTest<Runtime>, PoolId, AccountIdTest, Balance, MarketId>
      for Runtime
    {
        fn get_spot_price(
            pool_id: &PoolId,
            asset_in: &Asset<MarketId>,
            asset_out: &Asset<MarketId>,
            with_fees: bool,
        ) -> SerdeWrapper<Balance> {
            SerdeWrapper(Swaps::get_spot_price(pool_id, asset_in, asset_out, with_fees).ok().unwrap_or(0))
        }

        fn pool_account_id(pool_id: &PoolId) -> AccountIdTest {
            Swaps::pool_account_id(pool_id)
        }

        fn pool_shares_id(pool_id: PoolId) -> Asset<SerdeWrapper<MarketId>> {
            Asset::PoolShare(SerdeWrapper(pool_id))
        }

        fn get_all_spot_prices(
            pool_id: &PoolId,
            with_fees: bool
        ) -> Result<Vec<(Asset<MarketId>, Balance)>, DispatchError> {
            Swaps::get_all_spot_prices(pool_id, with_fees)
        }
    }
}

pub(super) fn mock_market(
    categories: u16,
) -> Market<AccountIdTest, Balance, BlockNumber, Moment, Asset<MarketId>> {
    Market {
        base_asset: BASE_ASSET,
        creation: MarketCreation::Permissionless,
        creator_fee: Perbill::from_parts(0),
        creator: DEFAULT_MARKET_CREATOR,
        market_type: MarketType::Categorical(categories),
        dispute_mechanism: Some(MarketDisputeMechanism::Authorized),
        metadata: vec![0; 50],
        oracle: DEFAULT_MARKET_ORACLE,
        period: MarketPeriod::Block(0..1),
        deadlines: Deadlines::default(),
        report: None,
        resolved_outcome: None,
        scoring_rule: ScoringRule::CPMM,
        status: MarketStatus::Active,
        bonds: MarketBonds::default(),
        early_close: None,
    }
}
