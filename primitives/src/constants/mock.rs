// Copyright 2022-2024 Forecasting Technologies LTD.
// Copyright 2022 Zeitgeist PM LLC.
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

#![cfg(feature = "mock")]

pub use super::*;
use crate::{
    assets::Asset,
    types::{Assets, Balance, Currencies, Moment},
};
use frame_support::{pallet_prelude::Weight, parameter_types, traits::LockIdentifier, PalletId};
use orml_traits::parameter_type_with_key;
use sp_arithmetic::Perbill;

// Asset-Router
parameter_types! {
    pub const DestroyAccountWeight: Weight = Weight::from_all(1000);
    pub const DestroyApprovalWeight: Weight = Weight::from_all(1000);
    pub const DestroyFinishWeight: Weight = Weight::from_all(1000);
}

// Assets
parameter_types! {
    pub const AssetsAccountDeposit: Balance = 0;
    pub const AssetsApprovalDeposit: Balance = 0;
    pub const AssetsDeposit: Balance = 0;
    pub const AssetsStringLimit: u32 = 256;
    pub const AssetsMetadataDepositBase: Balance = 0;
    pub const AssetsMetadataDepositPerByte: Balance = 0;
}

// Authorized
parameter_types! {
    pub const AuthorizedPalletId: PalletId = PalletId(*b"zge/atzd");
    pub const CorrectionPeriod: BlockNumber = 4;
}

// Court
parameter_types! {
    pub const AppealBond: Balance = 5 * BASE;
    pub const AppealBondFactor: Balance = 2 * BASE;
    pub const BlocksPerYear: BlockNumber = 10000;
    pub const CourtPalletId: PalletId = PalletId(*b"zge/cout");
    pub const RequestInterval: BlockNumber = 15;
    pub const VotePeriod: BlockNumber = 3;
    pub const AggregationPeriod: BlockNumber = 4;
    pub const AppealPeriod: BlockNumber = 5;
    pub const LockId: LockIdentifier = *b"zge/cloc";
    pub const MaxAppeals: u32 = 4;
    pub const MaxDelegations: u32 = 5;
    pub const MaxSelectedDraws: u32 = 510;
    pub const MaxCourtParticipants: u32 = 1_000;
    pub const MaxYearlyInflation: Perbill = Perbill::from_percent(10u32);
    pub const MinJurorStake: Balance = 50 * CENT;
    pub const InflationPeriod: BlockNumber = 20;
}

// Global disputes parameters
parameter_types! {
    pub const AddOutcomePeriod: BlockNumber = 20;
    pub const GlobalDisputeLockId: LockIdentifier = *b"zge/vote";
    pub const GlobalDisputesPalletId: PalletId = PalletId(*b"zge/gldp");
    pub const MaxGlobalDisputeVotes: u32 = 50;
    pub const MaxOwners: u32 = 10;
    pub const MinOutcomeVoteAmount: Balance = 10 * CENT;
    pub const RemoveKeysLimit: u32 = 250;
    pub const GdVotingPeriod: BlockNumber = 140;
    pub const VotingOutcomeFee: Balance = 100 * CENT;
}

// Hybrid Router parameters
parameter_types! {
    pub const HybridRouterPalletId: PalletId = PalletId(*b"zge/hybr");
    pub const MaxOrders: u32 = 100;
}

// Liquidity Mining parameters
parameter_types! {
    pub const LiquidityMiningPalletId: PalletId = PalletId(*b"zge/lymg");
}

// NeoSwaps
parameter_types! {
    pub storage NeoExitFee: Balance = CENT;
    pub const NeoMaxSwapFee: Balance = 10 * CENT;
    pub const MaxLiquidityTreeDepth: u32 = 3u32;
    pub const NeoSwapsPalletId: PalletId = PalletId(*b"zge/neos");
}

// Prediction Market parameters
parameter_types! {
    pub const AdvisoryBond: Balance = 25 * CENT;
    pub const CloseEarlyProtectionTimeFramePeriod: Moment = 3 * MILLISECS_PER_BLOCK as u64;
    pub const CloseEarlyProtectionBlockPeriod: BlockNumber = 3;
    pub const CloseEarlyRequestBond: Balance = 5 * BASE;
    pub const CloseEarlyDisputeBond: Balance = 10 * BASE;
    pub const DisputeBond: Balance = 5 * BASE;
    pub const DisputeFactor: Balance = 2 * BASE;
    pub const MaxCategories: u16 = 10;
    pub const MaxCreatorFee: Perbill = Perbill::from_percent(1);
    pub const MaxDisputeDuration: BlockNumber = 50;
    pub const MaxDisputes: u16 = 6;
    pub const MaxEditReasonLen: u32 = 1024;
    pub const MaxGracePeriod: BlockNumber = 20;
    pub const MaxMarketLifetime: BlockNumber = 100_000_000_000;
    pub const MaxOracleDuration: BlockNumber = 30;
    pub const MaxRejectReasonLen: u32 = 1024;
    pub const MinCategories: u16 = 2;
    pub const MinDisputeDuration: BlockNumber = 2;
    pub const MinOracleDuration: BlockNumber = 2;
    pub const OracleBond: Balance = 50 * CENT;
    pub const OutsiderBond: Balance = 2 * OracleBond::get();
    pub const PmPalletId: PalletId = PalletId(*b"zge/pred");
    pub const CloseEarlyBlockPeriod: BlockNumber = 6;
    pub const CloseEarlyTimeFramePeriod: Moment = 6 * MILLISECS_PER_BLOCK as u64;
    pub const ValidityBond: Balance = 50 * CENT;
}

// Simple disputes parameters
parameter_types! {
    pub const SimpleDisputesPalletId: PalletId = PalletId(*b"zge/sedp");
    pub const OutcomeBond: Balance = 5 * BASE;
    pub const OutcomeFactor: Balance = 2 * BASE;
}

// Swaps parameters
parameter_types! {
    pub const ExitFee: Balance = 3 * BASE / 1000; // 0.3%
    pub const MinAssets: u16 = 2;
    pub const MaxAssets: u16 = MaxCategories::get() + 1;
    pub const MaxSwapFee: Balance = BASE / 10; // 10%
    pub const MaxTotalWeight: Balance = 50 * BASE;
    pub const MaxWeight: Balance = 50 * BASE;
    pub const MinWeight: Balance = BASE;
    pub const SwapsPalletId: PalletId = PalletId(*b"zge/swap");
}

// Orderbook parameters
parameter_types! {
    pub const OrderbookPalletId: PalletId = PalletId(*b"zge/ordb");
}

// Parimutuel parameters
parameter_types! {
    pub const ParimutuelPalletId: PalletId = PalletId(*b"zge/prmt");
    pub const MinBetSize: Balance = BASE;
}

// Shared within tests
// Balance
parameter_types! {
    pub const ExistentialDeposit: u128 = CENT;
    pub const MaxLocks: u32 = 50;
    pub const MaxReserves: u32 = 50;
}

// Treasury
parameter_types! {
    pub const MaxApprovals: u32 = 1;
    pub const TreasuryPalletId: PalletId = PalletId(*b"zge/tsry");
}

// ORML
parameter_types! {
    // ORML
    pub const GetNativeCurrencyId: Assets = Asset::Ztg;
}

parameter_type_with_key! {
    pub ExistentialDeposits: |_currency_id: Currencies| -> Balance {2};
}

parameter_type_with_key! {
    pub ExistentialDepositsAssets: |_asset_id: Assets| -> Balance {2};
}

// System
parameter_types! {
    pub const BlockHashCount: u64 = 250;
}

// Time
parameter_types! {
    pub const MinimumPeriod: u64 = MILLISECS_PER_BLOCK as u64 / 2;
}
