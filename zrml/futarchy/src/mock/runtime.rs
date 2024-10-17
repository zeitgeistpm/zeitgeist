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
use crate::mock::types::{MockOracleQuery, MockScheduler};
use frame_support::{
    construct_runtime,
    pallet_prelude::Weight,
    parameter_types,
    traits::{EqualPrivilegeOnly, Everything},
};
use frame_system::{mocking::MockBlock, EnsureRoot};
use sp_runtime::{
    traits::{BlakeTwo256, ConstU32, IdentityLookup},
    Perbill,
};
use zeitgeist_primitives::{
    constants::{
        mock::{
            BlockHashCount, ExistentialDeposit, ExistentialDeposits, GetNativeCurrencyId, MaxLocks,
            MaxReserves, MinimumPeriod,
        },
        BLOCKS_PER_MINUTE,
    },
    types::{
        AccountIdTest, Amount, Balance, BasicCurrencyAdapter, BlockNumber, CurrencyId, Hash, Moment,
    },
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
        Futarchy: zrml_futarchy,
        Balances: pallet_balances,
        Currencies: orml_currencies,
        Preimage: pallet_preimage,
        System: frame_system,
        Timestamp: pallet_timestamp,
        Tokens: orml_tokens,
    }
}

impl zrml_futarchy::Config for Runtime {
    type MultiCurrency = Currencies;
    type MinDuration = MinDuration;
    type OracleQuery = MockOracleQuery;
    type Preimages = Preimage;
    type RuntimeEvent = RuntimeEvent;
    type Scheduler = MockScheduler;
    type SubmitOrigin = EnsureRoot<<Runtime as frame_system::Config>::AccountId>;
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

impl pallet_preimage::Config for Runtime {
    type WeightInfo = ();
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type ManagerOrigin = EnsureRoot<<Runtime as frame_system::Config>::AccountId>;
    type BaseDeposit = PreimageBaseDeposit;
    type ByteDeposit = PreimageByteDeposit;
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
