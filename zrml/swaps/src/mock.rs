// Copyright 2022-2024 Forecasting Technologies LTD.
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
use frame_system::mocking::MockBlock;
use orml_traits::parameter_type_with_key;
use sp_runtime::{
    traits::{AccountIdConversion, BlakeTwo256, IdentityLookup},
    BuildStorage,
};
use zeitgeist_primitives::{
    constants::mock::{
        BlockHashCount, ExistentialDeposit, GetNativeCurrencyId, MaxAssets, MaxLocks, MaxReserves,
        MaxSwapFee, MaxTotalWeight, MaxWeight, MinAssets, MinWeight, MinimumPeriod, SwapsPalletId,
        BASE,
    },
    types::{
        AccountIdTest, Amount, Asset, Balance, BasicCurrencyAdapter, CurrencyId, Hash, MarketId,
        Moment, PoolId, SerdeWrapper, UncheckedExtrinsicTest,
    },
};

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

pub type UncheckedExtrinsic = UncheckedExtrinsicTest<Runtime>;

// Mocked exit fee for easier calculations
parameter_types! {
    pub storage ExitFeeMock: Balance = BASE / 10;
}

construct_runtime!(
    pub enum Runtime {
        Balances: pallet_balances,
        Currencies: orml_currencies,
        Swaps: zrml_swaps,
        System: frame_system,
        Timestamp: pallet_timestamp,
        Tokens: orml_tokens,
    }
);

impl crate::Config for Runtime {
    type Asset = Asset<MarketId>;
    type MultiCurrency = Currencies;
    type RuntimeEvent = RuntimeEvent;
    type ExitFee = ExitFeeMock;
    type MaxAssets = MaxAssets;
    type MaxSwapFee = MaxSwapFee;
    type MaxTotalWeight = MaxTotalWeight;
    type MaxWeight = MaxWeight;
    type MinAssets = MinAssets;
    type MinWeight = MinWeight;
    type PalletId = SwapsPalletId;
    type WeightInfo = zrml_swaps::weights::WeightInfo<Runtime>;
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
    type MaxConsumers = frame_support::traits::ConstU32<16>;
    type OnKilledAccount = ();
    type OnNewAccount = ();
    type RuntimeOrigin = RuntimeOrigin;
    type PalletInfo = PalletInfo;
    type SS58Prefix = ();
    type SystemWeightInfo = ();
    type Version = ();
    type OnSetCode = ();
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
            _ => 10_000_000,
        }
    };
}

pub struct DustRemovalWhitelist;

impl Contains<AccountIdTest> for DustRemovalWhitelist
where
    frame_support::PalletId: AccountIdConversion<AccountIdTest>,
{
    fn contains(ai: &AccountIdTest) -> bool {
        let pallets = vec![SwapsPalletId::get()];

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
            frame_system::GenesisConfig::<Runtime>::default().build_storage().unwrap();

        // see the logs in tests when using `RUST_LOG=debug cargo test -- --nocapture`
        let _ = env_logger::builder().is_test(true).try_init();

        pallet_balances::GenesisConfig::<Runtime> { balances: self.balances }
            .assimilate_storage(&mut storage)
            .unwrap();

        sp_io::TestExternalities::from(storage)
    }
}

type Block = MockBlock<Runtime>;

sp_api::mock_impl_runtime_apis! {
    impl zrml_swaps_runtime_api::SwapsApi<MockBlock<Runtime>, PoolId, AccountIdTest, Balance, MarketId>
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
            Asset::PoolShare(pool_id)
        }
    }
}
