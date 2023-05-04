// Copyright 2022 Forecasting Technologies LTD.
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

use crate::{self as zrml_market_commons};
use frame_support::{construct_runtime, traits::Everything};
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};
use zeitgeist_primitives::{
    constants::mock::{BlockHashCount, MaxReserves, MinimumPeriod, PmPalletId},
    types::{
        AccountIdTest, Balance, BlockNumber, BlockTest, Hash, Index, MarketId, Moment,
        UncheckedExtrinsicTest,
    },
};

construct_runtime!(
    pub enum Runtime
    where
        Block = BlockTest<Runtime>,
        NodeBlock = BlockTest<Runtime>,
        UncheckedExtrinsic = UncheckedExtrinsicTest<Runtime>,
    {
        Balances: pallet_balances::{Call, Config<T>, Event<T>, Pallet, Storage},
        MarketCommons: zrml_market_commons::{Pallet, Storage},
        System: frame_system::{Call, Config,Event<T>, Pallet, Storage},
        Timestamp: pallet_timestamp::{Pallet},
    }
);

impl crate::Config for Runtime {
    type Currency = Balances;
    type MarketId = MarketId;
    type PredictionMarketsPalletId = PmPalletId;
    type Timestamp = Timestamp;
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
    type RuntimeEvent = ();
    type Hash = Hash;
    type Hashing = BlakeTwo256;
    type Header = Header;
    type Index = Index;
    type Lookup = IdentityLookup<Self::AccountId>;
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
    type RuntimeEvent = ();
    type ExistentialDeposit = ();
    type MaxLocks = ();
    type MaxReserves = MaxReserves;
    type ReserveIdentifier = [u8; 8];
    type WeightInfo = ();
}

impl pallet_timestamp::Config for Runtime {
    type MinimumPeriod = MinimumPeriod;
    type Moment = Moment;
    type OnTimestampSet = ();
    type WeightInfo = ();
}

#[derive(Default)]
pub struct ExtBuilder {}

impl ExtBuilder {
    pub fn build(self) -> sp_io::TestExternalities {
        let mut t = frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap();

        pallet_balances::GenesisConfig::<Runtime> { balances: vec![] }
            .assimilate_storage(&mut t)
            .unwrap();

        t.into()
    }
}
