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

#![cfg(test)]

use crate::{self as zrml_simple_disputes};
use frame_support::{
    construct_runtime, ord_parameter_types,
    pallet_prelude::{DispatchError, Weight},
    traits::Everything,
};
use frame_system::mocking::MockBlock;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};
use zeitgeist_primitives::{
    constants::mock::{
        BlockHashCount, ExistentialDepositsAssets, GetNativeCurrencyId, MaxDisputes, MaxLocks,
        MaxReserves, MinimumPeriod, OutcomeBond, OutcomeFactor, SimpleDisputesPalletId, BASE,
    },
    traits::{DisputeResolutionApi, MarketOfDisputeResolutionApi},
    types::{
        AccountIdTest, Amount, Assets, Balance, BasicCurrencyAdapter, BlockNumber, Hash, MarketId,
        Moment,
    },
};

pub const ALICE: AccountIdTest = 0;
pub const BOB: AccountIdTest = 1;
pub const CHARLIE: AccountIdTest = 2;
pub const DAVE: AccountIdTest = 3;
pub const EVE: AccountIdTest = 4;
pub const FRED: AccountIdTest = 5;
pub const SUDO: AccountIdTest = 69;

pub const INITIAL_BALANCE: u128 = 1_000 * BASE;

ord_parameter_types! {
    pub const Sudo: AccountIdTest = SUDO;
}

construct_runtime!(
    pub enum Runtime {
        AssetManager: orml_currencies,
        Balances: pallet_balances,
        MarketCommons: zrml_market_commons,
        SimpleDisputes: zrml_simple_disputes,
        System: frame_system,
        Timestamp: pallet_timestamp,
        Tokens: orml_tokens,
    }
);

// NoopResolution implements DisputeResolutionApi with no-ops.
pub struct NoopResolution;

impl DisputeResolutionApi for NoopResolution {
    type AccountId = AccountIdTest;
    type Balance = Balance;
    type BlockNumber = BlockNumber;
    type MarketId = MarketId;
    type Moment = Moment;

    fn resolve(
        _market_id: &Self::MarketId,
        _market: &MarketOfDisputeResolutionApi<Self>,
    ) -> Result<Weight, DispatchError> {
        Ok(Weight::zero())
    }

    fn add_auto_resolve(
        _market_id: &Self::MarketId,
        _resolve_at: Self::BlockNumber,
    ) -> Result<u32, DispatchError> {
        Ok(0u32)
    }

    fn auto_resolve_exists(_market_id: &Self::MarketId, _resolve_at: Self::BlockNumber) -> bool {
        false
    }

    fn remove_auto_resolve(_market_id: &Self::MarketId, _resolve_at: Self::BlockNumber) -> u32 {
        0u32
    }
}

impl crate::Config for Runtime {
    type Currency = Balances;
    type RuntimeEvent = RuntimeEvent;
    type DisputeResolution = NoopResolution;
    type MarketCommons = MarketCommons;
    type MaxDisputes = MaxDisputes;
    type OutcomeBond = OutcomeBond;
    type OutcomeFactor = OutcomeFactor;
    type PalletId = SimpleDisputesPalletId;
    type WeightInfo = zrml_simple_disputes::weights::WeightInfo<Runtime>;
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

impl pallet_balances::Config for Runtime {
    type AccountStore = System;
    type Balance = Balance;
    type DustRemoval = ();
    type FreezeIdentifier = ();
    type RuntimeHoldReason = ();
    type RuntimeEvent = RuntimeEvent;
    type ExistentialDeposit = ();
    type MaxHolds = ();
    type MaxFreezes = ();
    type MaxLocks = MaxLocks;
    type MaxReserves = MaxReserves;
    type ReserveIdentifier = [u8; 8];
    type WeightInfo = ();
}

impl orml_currencies::Config for Runtime {
    type GetNativeCurrencyId = GetNativeCurrencyId;
    type MultiCurrency = Tokens;
    type NativeCurrency = BasicCurrencyAdapter<Runtime, Balances>;
    type WeightInfo = ();
}

impl orml_tokens::Config for Runtime {
    type Amount = Amount;
    type Balance = Balance;
    type CurrencyId = Assets;
    type DustRemovalWhitelist = Everything;
    type RuntimeEvent = RuntimeEvent;
    type ExistentialDeposits = ExistentialDepositsAssets;
    type MaxLocks = ();
    type MaxReserves = MaxReserves;
    type CurrencyHooks = ();
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
            balances: vec![
                (ALICE, INITIAL_BALANCE),
                (BOB, INITIAL_BALANCE),
                (CHARLIE, INITIAL_BALANCE),
                (DAVE, INITIAL_BALANCE),
                (EVE, INITIAL_BALANCE),
                (FRED, INITIAL_BALANCE),
                (SUDO, INITIAL_BALANCE),
            ],
        }
    }
}

impl ExtBuilder {
    pub fn build(self) -> sp_io::TestExternalities {
        let mut t = frame_system::GenesisConfig::<Runtime>::default().build_storage().unwrap();

        // see the logs in tests when using `RUST_LOG=debug cargo test -- --nocapture`
        let _ = env_logger::builder().is_test(true).try_init();

        pallet_balances::GenesisConfig::<Runtime> { balances: self.balances }
            .assimilate_storage(&mut t)
            .unwrap();

        t.into()
    }
}
