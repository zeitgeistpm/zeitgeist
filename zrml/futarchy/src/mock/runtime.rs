// Copyright 2024 Forecasting Technologies LTD.
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
use crate::mock::types::{MockOracle, MockScheduler};
use frame_support::{construct_runtime, parameter_types, traits::Everything};
use frame_system::{mocking::MockBlock, EnsureRoot};
use sp_runtime::traits::{BlakeTwo256, ConstU32, IdentityLookup};
use zeitgeist_primitives::{
    constants::mock::{BlockHashCount, ExistentialDeposit, MaxLocks, MaxReserves},
    types::{AccountIdTest, Balance, BlockNumber, Hash},
};

parameter_types! {
    // zrml-futarchy
    pub const MinDuration: BlockNumber = 10;

    // pallet-preimage
    pub const PreimageBaseDeposit: Balance = 0;
    pub const PreimageByteDeposit: Balance = 0;
}

construct_runtime! {
    pub enum Runtime {
        System: frame_system,
        Balances: pallet_balances,
        Preimage: pallet_preimage,
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

impl pallet_preimage::Config for Runtime {
    type WeightInfo = ();
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type ManagerOrigin = EnsureRoot<<Runtime as frame_system::Config>::AccountId>;
    type BaseDeposit = PreimageBaseDeposit;
    type ByteDeposit = PreimageByteDeposit;
}

impl zrml_futarchy::Config for Runtime {
    type MinDuration = MinDuration;
    type Oracle = MockOracle;
    type RuntimeEvent = RuntimeEvent;
    type Scheduler = MockScheduler;
    type SubmitOrigin = EnsureRoot<<Runtime as frame_system::Config>::AccountId>;
}
