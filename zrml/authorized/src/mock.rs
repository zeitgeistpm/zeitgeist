// Copyright 2022-2025 Forecasting Technologies LTD.
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

use crate::{self as zrml_authorized, mock_storage::pallet as mock_storage};
use alloc::{vec, vec::Vec};
use frame_support::{construct_runtime, ord_parameter_types, traits::Everything, weights::Weight};
use frame_system::{mocking::MockBlock, EnsureSignedBy};
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage, DispatchError,
};
use zeitgeist_primitives::{
    constants::mock::{
        AuthorizedPalletId, BlockHashCount, CorrectionPeriod, ExistentialDeposit, MaxLocks,
        MaxReserves, MinimumPeriod, BASE,
    },
    traits::{DisputeResolutionApi, MarketOfDisputeResolutionApi},
    types::{AccountIdTest, Balance, BlockNumber, Hash, MarketId, Moment},
};

pub const ALICE: AccountIdTest = 0;
pub const BOB: AccountIdTest = 1;
pub const CHARLIE: AccountIdTest = 2;

construct_runtime!(
    pub enum Runtime {
        Authorized: zrml_authorized,
        Balances: pallet_balances,
        MarketCommons: zrml_market_commons,
        System: frame_system,
        Timestamp: pallet_timestamp,
        // Just a mock storage for testing.
        MockStorage: mock_storage,
    }
);

ord_parameter_types! {
    pub const AuthorizedDisputeResolutionUser: AccountIdTest = ALICE;
}

// MockResolution implements DisputeResolutionApi with no-ops.
pub struct MockResolution;

impl DisputeResolutionApi for MockResolution {
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
        market_id: &Self::MarketId,
        resolve_at: Self::BlockNumber,
    ) -> Result<u32, DispatchError> {
        let ids_len = <mock_storage::MarketIdsPerDisputeBlock<Runtime>>::try_mutate(
            resolve_at,
            |ids| -> Result<u32, DispatchError> {
                ids.try_push(*market_id).map_err(|_| DispatchError::Other("Storage Overflow"))?;
                Ok(ids.len() as u32)
            },
        )?;
        Ok(ids_len)
    }

    fn auto_resolve_exists(market_id: &Self::MarketId, resolve_at: Self::BlockNumber) -> bool {
        <mock_storage::MarketIdsPerDisputeBlock<Runtime>>::get(resolve_at).contains(market_id)
    }

    fn remove_auto_resolve(market_id: &Self::MarketId, resolve_at: Self::BlockNumber) -> u32 {
        <mock_storage::MarketIdsPerDisputeBlock<Runtime>>::mutate(resolve_at, |ids| -> u32 {
            ids.retain(|id| id != market_id);
            ids.len() as u32
        })
    }
}

impl crate::Config for Runtime {
    type Currency = Balances;
    type RuntimeEvent = RuntimeEvent;
    type CorrectionPeriod = CorrectionPeriod;
    type DisputeResolution = MockResolution;
    type MarketCommons = MarketCommons;
    type PalletId = AuthorizedPalletId;
    type AuthorizedDisputeResolutionOrigin =
        EnsureSignedBy<AuthorizedDisputeResolutionUser, AccountIdTest>;
    type WeightInfo = crate::weights::WeightInfo<Runtime>;
}

impl mock_storage::Config for Runtime {
    type MarketCommons = MarketCommons;
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
    type RuntimeTask = RuntimeTask;
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
    type ExistentialDeposit = ExistentialDeposit;
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
        let mut t = frame_system::GenesisConfig::<Runtime>::default().build_storage().unwrap();

        // see the logs in tests when using `RUST_LOG=debug cargo test -- --nocapture`
        let _ = env_logger::builder().is_test(true).try_init();

        pallet_balances::GenesisConfig::<Runtime> { balances: self.balances }
            .assimilate_storage(&mut t)
            .unwrap();

        t.into()
    }
}
