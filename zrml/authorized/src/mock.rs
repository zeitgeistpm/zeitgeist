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

extern crate alloc;

use crate::{self as zrml_authorized};
use alloc::{vec, vec::Vec};
use frame_support::{
    construct_runtime, ord_parameter_types,
    pallet_prelude::{DispatchError, Get, Weight},
    traits::Everything,
    BoundedVec,
};
use frame_system::EnsureSignedBy;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};
use zeitgeist_primitives::{
    constants::mock::{
        AuthorizedPalletId, BlockHashCount, CorrectionPeriod, MaxReserves, MinimumPeriod,
        PmPalletId, ReportPeriod, BASE,
    },
    traits::DisputeResolutionApi,
    types::{
        AccountIdTest, Balance, BlockNumber, BlockTest, Hash, Index, Market, MarketDispute,
        MarketId, Moment, OutcomeReport, UncheckedExtrinsicTest,
    },
};

pub const ALICE: AccountIdTest = 0;
pub const BOB: AccountIdTest = 1;
pub const CHARLIE: AccountIdTest = 2;

construct_runtime!(
    pub enum Runtime
    where
        Block = BlockTest<Runtime>,
        NodeBlock = BlockTest<Runtime>,
        UncheckedExtrinsic = UncheckedExtrinsicTest<Runtime>,
    {
        Authorized: zrml_authorized::{Event<T>, Pallet, Storage},
        Balances: pallet_balances::{Call, Config<T>, Event<T>, Pallet, Storage},
        MarketCommons: zrml_market_commons::{Pallet, Storage},
        System: frame_system::{Call, Config, Event<T>, Pallet, Storage},
        Timestamp: pallet_timestamp::{Pallet},
    }
);

ord_parameter_types! {
    pub const AuthorizedDisputeResolutionUser: AccountIdTest = ALICE;
}

pub struct MaxDisputes;
impl Get<u32> for MaxDisputes {
    fn get() -> u32 {
        64u32
    }
}

type MaxDisputesTest = MaxDisputes;

// MockResolution implements DisputeResolutionApi with no-ops.
pub struct MockResolution;

impl DisputeResolutionApi for MockResolution {
    type AccountId = AccountIdTest;
    type BlockNumber = BlockNumber;
    type MarketId = MarketId;
    type MaxDisputes = MaxDisputesTest;
    type Moment = Moment;

    fn resolve(
        _market_id: &Self::MarketId,
        _market: &Market<Self::AccountId, Self::BlockNumber, Self::Moment>,
    ) -> Result<Weight, DispatchError> {
        Ok(0)
    }

    fn add_auto_resolve(
        market_id: &Self::MarketId,
        resolve_at: Self::BlockNumber,
    ) -> Result<u32, DispatchError> {
        let ids_len = <crate::MarketIdsPerDisputeBlock<Runtime>>::try_mutate(
            resolve_at,
            |ids| -> Result<u32, DispatchError> {
                ids.try_push(*market_id).map_err(|_| DispatchError::Other("Storage Overflow"))?;
                Ok(ids.len() as u32)
            },
        )?;
        Ok(ids_len)
    }

    fn remove_auto_resolve(market_id: &Self::MarketId, resolve_at: Self::BlockNumber) -> u32 {
        <crate::MarketIdsPerDisputeBlock<Runtime>>::mutate(resolve_at, |ids| -> u32 {
            ids.retain(|id| id != market_id);
            ids.len() as u32
        })
    }

    fn get_disputes(
        _market_id: &Self::MarketId,
    ) -> BoundedVec<MarketDispute<Self::AccountId, Self::BlockNumber>, Self::MaxDisputes> {
        BoundedVec::try_from(vec![MarketDispute {
            at: 42u64,
            by: BOB,
            outcome: OutcomeReport::Scalar(42),
        }])
        .unwrap()
    }
}

impl crate::Config for Runtime {
    type ReportPeriod = ReportPeriod;
    type Event = ();
    type CorrectionPeriod = CorrectionPeriod;
    type DisputeResolution = MockResolution;
    type MarketCommons = MarketCommons;
    type PalletId = AuthorizedPalletId;
    type AuthorizedDisputeResolutionOrigin =
        EnsureSignedBy<AuthorizedDisputeResolutionUser, AccountIdTest>;
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
        Self { balances: vec![(ALICE, 1_000 * BASE), (BOB, 1_000 * BASE), (CHARLIE, 1_000 * BASE)] }
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
