// Copyright 2024-2025 Forecasting Technologies LTD.
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

use crate as zrml_futarchy;
use crate::{
    mock::types::{MockOracle, MockScheduler},
    weights::WeightInfo,
};
use frame_support::{construct_runtime, parameter_types, traits::Everything};
use frame_system::mocking::MockBlock;
use sp_runtime::traits::{BlakeTwo256, ConstU32, IdentityLookup};
use zeitgeist_primitives::{
    constants::mock::{BlockHashCount, ExistentialDeposit, MaxLocks, MaxReserves},
    types::{AccountIdTest, Balance, BlockNumber, Hash},
};

#[cfg(feature = "runtime-benchmarks")]
use crate::mock::types::MockBenchmarkHelper;

parameter_types! {
    // zrml-futarchy
    pub const MaxProposals: u32 = 16;
    pub const MinDuration: BlockNumber = 10;
}

construct_runtime! {
    pub enum Runtime {
        System: frame_system,
        Balances: pallet_balances,
        Futarchy: zrml_futarchy,
    }
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

impl zrml_futarchy::Config for Runtime {
    #[cfg(feature = "runtime-benchmarks")]
    type BenchmarkHelper = MockBenchmarkHelper;
    type MaxProposals = MaxProposals;
    type MinDuration = MinDuration;
    type Oracle = MockOracle;
    type RuntimeEvent = RuntimeEvent;
    type Scheduler = MockScheduler;
    type WeightInfo = WeightInfo<Runtime>;
}
