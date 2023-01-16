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
    traits::{Everything, NeverEnsureOrigin},
};
use frame_system::EnsureSignedBy;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
};
use zeitgeist_primitives::{
    constants::mock::{
        BlockHashCount, ExistentialDeposits, GetNativeCurrencyId, MaxApprovals, MaxDisputes,
        MaxReserves, MinimumPeriod, OutcomeBond, OutcomeFactor, PmPalletId, SimpleDisputesPalletId,
        TreasuryPalletId,
    },
    traits::DisputeResolutionApi,
    types::{
        AccountIdTest, Amount, Balance, BasicCurrencyAdapter, BlockNumber, BlockTest, CurrencyId,
        Hash, Index, Market, MarketId, Moment, UncheckedExtrinsicTest,
    },
};

#[cfg(feature = "with-global-disputes")]
use zeitgeist_primitives::constants::mock::{
    GlobalDisputeLockId, GlobalDisputesPalletId, MaxGlobalDisputeVotes, MaxOwners,
    MinOutcomeVoteAmount, RemoveKeysLimit, VotingOutcomeFee,
};

pub const SUDO: AccountIdTest = 69;

ord_parameter_types! {
    pub const Sudo: AccountIdTest = SUDO;
}

#[cfg(feature = "with-global-disputes")]
construct_runtime!(
    pub enum Runtime
    where
        Block = BlockTest<Runtime>,
        NodeBlock = BlockTest<Runtime>,
        UncheckedExtrinsic = UncheckedExtrinsicTest<Runtime>,
    {
        Balances: pallet_balances::{Call, Config<T>, Event<T>, Pallet, Storage},
        AssetManager: orml_currencies::{Call, Pallet, Storage},
        MarketCommons: zrml_market_commons::{Pallet, Storage},
        SimpleDisputes: zrml_simple_disputes::{Event<T>, Pallet, Storage},
        GlobalDisputes: zrml_global_disputes::{Event<T>, Pallet, Storage},
        System: frame_system::{Call, Config, Event<T>, Pallet, Storage},
        Timestamp: pallet_timestamp::{Pallet},
        Tokens: orml_tokens::{Config<T>, Event<T>, Pallet, Storage},
        Treasury: pallet_treasury::{Call, Event<T>, Pallet, Storage},
    }
);

#[cfg(not(feature = "with-global-disputes"))]
construct_runtime!(
    pub enum Runtime
    where
        Block = BlockTest<Runtime>,
        NodeBlock = BlockTest<Runtime>,
        UncheckedExtrinsic = UncheckedExtrinsicTest<Runtime>,
    {
        Balances: pallet_balances::{Call, Config<T>, Event<T>, Pallet, Storage},
        AssetManager: orml_currencies::{Call, Pallet, Storage},
        MarketCommons: zrml_market_commons::{Pallet, Storage},
        SimpleDisputes: zrml_simple_disputes::{Event<T>, Pallet, Storage},
        System: frame_system::{Call, Config, Event<T>, Pallet, Storage},
        Timestamp: pallet_timestamp::{Pallet},
        Tokens: orml_tokens::{Config<T>, Event<T>, Pallet, Storage},
        Treasury: pallet_treasury::{Call, Event<T>, Pallet, Storage},
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
        Ok(0)
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
    type AssetManager = AssetManager;
    type Event = ();
    type OutcomeBond = OutcomeBond;
    type OutcomeFactor = OutcomeFactor;
    type DisputeResolution = NoopResolution;
    #[cfg(feature = "with-global-disputes")]
    type GlobalDisputes = GlobalDisputes;
    type MarketCommons = MarketCommons;
    type MaxDisputes = MaxDisputes;
    type PalletId = SimpleDisputesPalletId;
    type PredictionMarketsPalletId = PmPalletId;
    type WeightInfo = zrml_simple_disputes::weights::WeightInfo<Runtime>;
}

#[cfg(feature = "with-global-disputes")]
impl zrml_global_disputes::Config for Runtime {
    type Event = ();
    type MarketCommons = MarketCommons;
    type Currency = Balances;
    type GlobalDisputeLockId = GlobalDisputeLockId;
    type GlobalDisputesPalletId = GlobalDisputesPalletId;
    type MaxGlobalDisputeVotes = MaxGlobalDisputeVotes;
    type MaxOwners = MaxOwners;
    type MinOutcomeVoteAmount = MinOutcomeVoteAmount;
    type RemoveKeysLimit = RemoveKeysLimit;
    type VotingOutcomeFee = VotingOutcomeFee;
    type WeightInfo = zrml_global_disputes::weights::WeightInfo<Runtime>;
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
    type Event = ();
    type ExistentialDeposits = ExistentialDeposits;
    type MaxLocks = ();
    type MaxReserves = MaxReserves;
    type OnDust = ();
    type OnKilledTokenAccount = ();
    type OnNewTokenAccount = ();
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
    type Event = ();
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

pub struct ExtBuilder;

impl ExtBuilder {
    pub fn build(self) -> sp_io::TestExternalities {
        frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap().into()
    }
}
