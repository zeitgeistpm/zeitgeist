#![cfg(feature = "mock")]

pub use super::*;
use crate::{
    asset::Asset,
    types::{Balance, CurrencyId, Moment},
};
use frame_support::{parameter_types, traits::LockIdentifier, PalletId};
use orml_traits::parameter_type_with_key;

// Authorized
parameter_types! {
    pub const AuthorizedPalletId: PalletId = PalletId(*b"zge/atzd");
    pub const CorrectionPeriod: BlockNumber = 4;
}

// Court
parameter_types! {
    pub const CourtCaseDuration: u64 = BLOCKS_PER_DAY;
    pub const CourtPalletId: PalletId = PalletId(*b"zge/cout");
    pub const StakeWeight: u128 = 2 * BASE;
}

// Global disputes parameters
parameter_types! {
    pub const GlobalDisputeLockId: LockIdentifier = *b"zge/vote";
    pub const GlobalDisputesPalletId: PalletId = PalletId(*b"zge/gldp");
    pub const MaxGlobalDisputeVotes: u32 = 50;
    pub const MaxOwners: u32 = 10;
    pub const MinOutcomeVoteAmount: Balance = 10 * CENT;
    pub const RemoveKeysLimit: u32 = 250;
    pub const VotingOutcomeFee: Balance = 100 * CENT;
}

// Liquidity Mining parameters
parameter_types! {
    pub const LiquidityMiningPalletId: PalletId = PalletId(*b"zge/lymg");
}

// Prediction Market parameters
parameter_types! {
    pub const AdvisoryBond: Balance = 25 * CENT;
    pub const DisputeBond: Balance = 5 * BASE;
    pub const DisputeFactor: Balance = 2 * BASE;
    pub const GlobalDisputePeriod: BlockNumber = 7 * BLOCKS_PER_DAY;
    pub const MaxCategories: u16 = 10;
    pub const MaxDisputeDuration: BlockNumber = 50;
    pub const MaxDisputes: u16 = 6;
    pub const MaxEditReasonLen: u32 = 1024;
    pub const MaxGracePeriod: BlockNumber = 20;
    pub const MaxMarketLifetime: BlockNumber = 1_000_000;
    pub const MaxOracleDuration: BlockNumber = 30;
    pub const MaxRejectReasonLen: u32 = 1024;
    // 2_678_400_000 = 31 days.
    pub const MaxSubsidyPeriod: Moment = 2_678_400_000;
    pub const MinCategories: u16 = 2;
    pub const MinDisputeDuration: BlockNumber = 2;
    pub const MinOracleDuration: BlockNumber = 2;
    // 60_000 = 1 minute. Should be raised to something more reasonable in the future.
    pub const MinSubsidyPeriod: Moment = 60_000;
    pub const OracleBond: Balance = 50 * CENT;
    pub const OutsiderBond: Balance = 2 * OracleBond::get();
    pub const PmPalletId: PalletId = PalletId(*b"zge/pred");
    pub const ValidityBond: Balance = 50 * CENT;
}

// Simple disputes parameters
parameter_types! {
    pub const SimpleDisputesPalletId: PalletId = PalletId(*b"zge/sedp");
}

// Swaps parameters
parameter_types! {
    pub const ExitFee: Balance = 3 * BASE / 1000; // 0.3%
    pub const MinAssets: u16 = 2;
    pub const MaxAssets: u16 = MaxCategories::get() + 1;
    pub const MaxInRatio: Balance = (BASE / 3) + 1;
    pub const MaxOutRatio: Balance = (BASE / 3) + 1;
    pub const MaxSwapFee: Balance = BASE / 10; // 10%
    pub const MaxTotalWeight: Balance = 50 * BASE;
    pub const MaxWeight: Balance = 50 * BASE;
    pub const MinLiquidity: Balance = 100 * BASE;
    pub const MinSubsidy: Balance = MinLiquidity::get();
    pub const MinSubsidyPerAccount: Balance = MinSubsidy::get();
    pub const MinWeight: Balance = BASE;
    pub const SwapsPalletId: PalletId = PalletId(*b"zge/swap");
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
    pub const GetNativeCurrencyId: CurrencyId = Asset::Ztg;
}

parameter_type_with_key! {
    // Well, not every asset is a currency ¯\_(ツ)_/¯
    pub ExistentialDeposits: |currency_id: CurrencyId| -> Balance {
        match currency_id {
            Asset::Ztg => ExistentialDeposit::get(),
            _ => 0
        }
    };
}

// System
parameter_types! {
    pub const BlockHashCount: u64 = 250;
}

// Time
parameter_types! {
    pub const MinimumPeriod: u64 = MILLISECS_PER_BLOCK as u64 / 2;
}
