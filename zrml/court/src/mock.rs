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

use crate::{self as zrml_court};
use frame_support::{
    construct_runtime,
    pallet_prelude::{DispatchError, Weight},
    parameter_types,
    traits::Everything,
    BoundedVec, PalletId,
};
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};
use zeitgeist_primitives::{
    constants::mock::{
        BlockHashCount, CourtCaseDuration, CourtPalletId, MaxReserves, MinimumPeriod, PmPalletId,
        StakeWeight, BASE,
    },
    traits::DisputeResolutionApi,
    types::{
        AccountIdTest, Balance, BlockNumber, BlockTest, Hash, Index, Market, MarketDispute,
        MarketId, Moment, UncheckedExtrinsicTest,
    },
};

pub const ALICE: AccountIdTest = 0;
pub const BOB: AccountIdTest = 1;
pub const CHARLIE: AccountIdTest = 2;
pub const INITIAL_BALANCE: u128 = 1000 * BASE;

parameter_types! {
    pub const TreasuryPalletId: PalletId = PalletId(*b"3.141592");
}

construct_runtime!(
    pub enum Runtime
    where
        Block = BlockTest<Runtime>,
        NodeBlock = BlockTest<Runtime>,
        UncheckedExtrinsic = UncheckedExtrinsicTest<Runtime>,
    {
        Balances: pallet_balances::{Call, Config<T>, Event<T>, Pallet, Storage},
        Court: zrml_court::{Event<T>, Pallet, Storage},
        MarketCommons: zrml_market_commons::{Pallet, Storage},
        RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Pallet, Storage},
        System: frame_system::{Call, Config, Event<T>, Pallet, Storage},
        Timestamp: pallet_timestamp::{Pallet},
    }
);

// NoopResolution implements DisputeResolutionApi with no-ops.
pub struct NoopResolution;

impl DisputeResolutionApi for NoopResolution {
    type AccountId = AccountIdTest;
    type Balance = Balance;
    type BlockNumber = BlockNumber;
    type MarketId = MarketId;
    type MaxDisputes = u32;
    type Moment = Moment;

    fn resolve(
        _market_id: &Self::MarketId,
        _market: &Market<Self::AccountId, Self::Balance, Self::BlockNumber, Self::Moment>,
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

    fn get_disputes(
        _market_id: &Self::MarketId,
    ) -> BoundedVec<MarketDispute<Self::AccountId, Self::BlockNumber>, Self::MaxDisputes> {
        Default::default()
    }
}

impl crate::Config for Runtime {
    type CourtCaseDuration = CourtCaseDuration;
    type DisputeResolution = NoopResolution;
    type Event = ();
    type MarketCommons = MarketCommons;
    type PalletId = CourtPalletId;
    type Random = RandomnessCollectiveFlip;
    type StakeWeight = StakeWeight;
    type TreasuryPalletId = TreasuryPalletId;
    type WeightInfo = crate::weights::WeightInfo<Runtime>;
}

impl frame_system::Config for Runtime {
    type AccountData = pallet_balances::AccountData<Balance>;
    type AccountId = AccountIdTest;
    type BaseCallFilter = Everything;
    type BlockHashCount = BlockHashCount;
    type BlockLength = ();
    type BlockNumber = BlockNumber;
    type BlockWeights = ();
    type Call = Call;
    type DbWeight = ();
    type Event = ();
    type Hash = Hash;
    type Hashing = BlakeTwo256;
    type Header = Header;
    type Index = Index;
    type Lookup = IdentityLookup<Self::AccountId>;
    type MaxConsumers = frame_support::traits::ConstU32<16>;
    type OnKilledAccount = ();
    type OnNewAccount = ();
    type Origin = Origin;
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
    type Event = ();
    type ExistentialDeposit = ();
    type MaxLocks = ();
    type MaxReserves = MaxReserves;
    type ReserveIdentifier = [u8; 8];
    type WeightInfo = ();
}

impl pallet_randomness_collective_flip::Config for Runtime {}

impl zrml_market_commons::Config for Runtime {
    type Currency = Balances;
    type MarketId = MarketId;
    type PredictionMarketsPalletId = PmPalletId;
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
                (ALICE, 1_000 * BASE),
                (BOB, 1_000 * BASE),
                (CHARLIE, 1_000 * BASE),
                (Court::treasury_account_id(), 1_000 * BASE),
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
