// Copyright 2022-2023 Forecasting Technologies LTD.
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

use crate::{self as zrml_court, mock_storage::pallet as mock_storage};
use frame_support::{
    construct_runtime, ord_parameter_types,
    pallet_prelude::{DispatchError, Weight},
    parameter_types,
    traits::{Everything, Hooks, NeverEnsureOrigin},
    PalletId,
};
use frame_system::{EnsureRoot, EnsureSignedBy};
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};
use zeitgeist_primitives::{
    constants::mock::{
        AppealBond, BlockHashCount, BlocksPerYear, CourtAggregationPeriod, CourtAppealPeriod,
        CourtLockId, CourtPalletId, CourtVotePeriod, InflationPeriod, MaxAppeals, MaxApprovals,
        MaxDelegations, MaxJurors, MaxReserves, MaxSelectedDraws, MinJurorStake, MinimumPeriod,
        PmPalletId, RequestInterval, BASE,
    },
    traits::DisputeResolutionApi,
    types::{
        AccountIdTest, Asset, Balance, BlockNumber, BlockTest, Hash, Index, Market, MarketId,
        Moment, UncheckedExtrinsicTest,
    },
};

pub const ALICE: AccountIdTest = 0;
pub const BOB: AccountIdTest = 1;
pub const CHARLIE: AccountIdTest = 2;
pub const DAVE: AccountIdTest = 3;
pub const EVE: AccountIdTest = 4;
pub const FERDIE: AccountIdTest = 5;
pub const GINA: AccountIdTest = 6;
pub const HARRY: AccountIdTest = 7;
pub const IAN: AccountIdTest = 8;
pub const POOR_PAUL: AccountIdTest = 9;
pub const INITIAL_BALANCE: u128 = 1000 * BASE;
pub const SUDO: AccountIdTest = 69;

ord_parameter_types! {
    pub const Sudo: AccountIdTest = SUDO;
}

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
        System: frame_system::{Call, Config, Event<T>, Pallet, Storage},
        Timestamp: pallet_timestamp::{Pallet},
        Treasury: pallet_treasury::{Call, Event<T>, Pallet, Storage},
        // Just a mock storage for testing.
        MockStorage: mock_storage::{Storage},
    }
);

// MockResolution implements DisputeResolutionApi with no-ops.
pub struct MockResolution;

impl mock_storage::Config for Runtime {
    type MarketCommons = MarketCommons;
}

impl DisputeResolutionApi for MockResolution {
    type AccountId = AccountIdTest;
    type Balance = Balance;
    type BlockNumber = BlockNumber;
    type MarketId = MarketId;
    type Moment = Moment;

    fn resolve(
        _market_id: &Self::MarketId,
        _market: &Market<
            Self::AccountId,
            Self::Balance,
            Self::BlockNumber,
            Self::Moment,
            Asset<Self::MarketId>,
        >,
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
    type AppealBond = AppealBond;
    type BlocksPerYear = BlocksPerYear;
    type CourtLockId = CourtLockId;
    type Currency = Balances;
    type CourtVotePeriod = CourtVotePeriod;
    type CourtAggregationPeriod = CourtAggregationPeriod;
    type CourtAppealPeriod = CourtAppealPeriod;
    type DisputeResolution = MockResolution;
    type Event = Event;
    type InflationPeriod = InflationPeriod;
    type MarketCommons = MarketCommons;
    type MaxAppeals = MaxAppeals;
    type MaxDelegations = MaxDelegations;
    type MaxSelectedDraws = MaxSelectedDraws;
    type MaxJurors = MaxJurors;
    type MinJurorStake = MinJurorStake;
    type MonetaryGovernanceOrigin = EnsureRoot<AccountIdTest>;
    type CourtPalletId = CourtPalletId;
    type Random = MockStorage;
    type RequestInterval = RequestInterval;
    type Slash = Treasury;
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
    type Event = Event;
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
    type Event = Event;
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

impl pallet_treasury::Config for Runtime {
    type ApproveOrigin = EnsureSignedBy<Sudo, AccountIdTest>;
    type Burn = ();
    type BurnDestination = ();
    type Currency = Balances;
    type Event = Event;
    type MaxApprovals = MaxApprovals;
    type OnSlash = ();
    type PalletId = TreasuryPalletId;
    type ProposalBond = ();
    type ProposalBondMinimum = ();
    type ProposalBondMaximum = ();
    type RejectOrigin = EnsureSignedBy<Sudo, AccountIdTest>;
    type SpendFunds = ();
    type SpendOrigin = NeverEnsureOrigin<Balance>;
    type SpendPeriod = ();
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
                (EVE, 1_000 * BASE),
                (DAVE, 1_000 * BASE),
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

        let mut t: sp_io::TestExternalities = t.into();
        // required to assert for events
        t.execute_with(|| System::set_block_number(1));
        t
    }
}

pub fn run_to_block(n: BlockNumber) {
    while System::block_number() < n {
        Balances::on_finalize(System::block_number());
        Court::on_finalize(System::block_number());
        System::on_finalize(System::block_number());
        System::set_block_number(System::block_number() + 1);

        // new block
        let parent_block_hash = System::parent_hash();
        let current_digest = System::digest();
        System::initialize(&System::block_number(), &parent_block_hash, &current_digest);
        System::on_initialize(System::block_number());
        Court::on_initialize(System::block_number());
        Balances::on_initialize(System::block_number());
    }
}

pub fn run_blocks(n: BlockNumber) {
    run_to_block(System::block_number() + n);
}
