// Copyright 2023-2025 Forecasting Technologies LTD.
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

extern crate alloc;

use crate as zrml_parimutuel;
use crate::{AssetOf, BalanceOf, MarketIdOf};
use alloc::{vec, vec::Vec};
use core::marker::PhantomData;
use frame_support::{construct_runtime, pallet_prelude::Get, parameter_types, traits::Everything};
use frame_system::mocking::MockBlock;
use orml_traits::MultiCurrency;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage, Perbill, SaturatedConversion,
};
use zeitgeist_primitives::{
    constants::mock::{
        BlockHashCount, ExistentialDeposits, GetNativeCurrencyId, MaxLocks, MaxReserves,
        MinBetSize, MinimumPeriod, ParimutuelPalletId, BASE, CENT,
    },
    traits::DistributeFees,
    types::{
        AccountIdTest, Amount, Balance, BasicCurrencyAdapter, CurrencyId, Hash, MarketId, Moment,
    },
};

pub const ALICE: AccountIdTest = 0;
pub const BOB: AccountIdTest = 1;
pub const CHARLIE: AccountIdTest = 2;

pub const MARKET_CREATOR: AccountIdTest = 42;

pub const INITIAL_BALANCE: u128 = 1_000 * BASE;

pub const EXTERNAL_FEES: u128 = CENT;

parameter_types! {
    pub const FeeAccount: AccountIdTest = MARKET_CREATOR;
}

pub fn fee_percentage() -> Perbill {
    Perbill::from_rational(EXTERNAL_FEES, BASE)
}

pub fn calculate_fee<T: crate::Config>(amount: BalanceOf<T>) -> BalanceOf<T> {
    fee_percentage().mul_floor(amount.saturated_into::<BalanceOf<T>>())
}

pub struct ExternalFees<T, F>(PhantomData<T>, PhantomData<F>);

impl<T: crate::Config, F> DistributeFees for ExternalFees<T, F>
where
    F: Get<T::AccountId>,
{
    type Asset = AssetOf<T>;
    type AccountId = T::AccountId;
    type Balance = BalanceOf<T>;
    type MarketId = MarketIdOf<T>;

    fn distribute(
        _market_id: Self::MarketId,
        asset: Self::Asset,
        account: &Self::AccountId,
        amount: Self::Balance,
    ) -> Self::Balance {
        let fees = calculate_fee::<T>(amount.saturated_into::<BalanceOf<T>>());
        let _ = T::AssetManager::transfer(asset, account, &F::get(), fees);
        fees
    }

    fn fee_percentage(_market_id: Self::MarketId) -> Perbill {
        fee_percentage()
    }
}

construct_runtime!(
    pub enum Runtime {
        AssetManager: orml_currencies,
        Balances: pallet_balances,
        MarketCommons: zrml_market_commons,
        Parimutuel: zrml_parimutuel,
        System: frame_system,
        Timestamp: pallet_timestamp,
        Tokens: orml_tokens,
    }
);

impl crate::Config for Runtime {
    type ExternalFees = ExternalFees<Runtime, FeeAccount>;
    type RuntimeEvent = RuntimeEvent;
    type MarketCommons = MarketCommons;
    type AssetManager = AssetManager;
    type MinBetSize = MinBetSize;
    type PalletId = ParimutuelPalletId;
    type WeightInfo = crate::weights::WeightInfo<Runtime>;
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
    type RuntimeTask = RuntimeTask;
    type DbWeight = ();
    type RuntimeEvent = RuntimeEvent;
    type Hash = Hash;
    type Hashing = BlakeTwo256;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Nonce = u64;
    type MaxConsumers = frame_support::traits::ConstU32<16>;
    type MultiBlockMigrator = ();
    type OnKilledAccount = ();
    type OnNewAccount = ();
    type RuntimeOrigin = RuntimeOrigin;
    type PalletInfo = PalletInfo;
    type PreInherents = ();
    type PostInherents = ();
    type PostTransactions = ();
    type SingleBlockMigrations = ();
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
    type MaxFreezes = ();
    type MaxLocks = MaxLocks;
    type MaxReserves = MaxReserves;
    type ReserveIdentifier = [u8; 8];
    type RuntimeFreezeReason = ();
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

impl orml_currencies::Config for Runtime {
    type GetNativeCurrencyId = GetNativeCurrencyId;
    type MultiCurrency = Tokens;
    type NativeCurrency = BasicCurrencyAdapter<Runtime, Balances>;
    type WeightInfo = ();
}

impl orml_tokens::Config for Runtime {
    type Amount = Amount;
    type Balance = Balance;
    type CurrencyId = CurrencyId;
    type DustRemovalWhitelist = Everything;
    type RuntimeEvent = RuntimeEvent;
    type ExistentialDeposits = ExistentialDeposits;
    type MaxLocks = ();
    type MaxReserves = MaxReserves;
    type CurrencyHooks = ();
    type ReserveIdentifier = [u8; 8];
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

        let mut t: sp_io::TestExternalities = t.into();

        // to ensure we can have events emitted in the tests. events not present at genesis block
        t.execute_with(|| System::set_block_number(1));

        t
    }
}
