pub mod ztg;

use crate::types::{Balance, BlockNumber};
use frame_support::{parameter_types, PalletId};

// General
pub const BLOCK_HASH_COUNT: BlockNumber = 250;

// Definitions for time
pub const BLOCKS_PER_DAY: BlockNumber = BLOCKS_PER_HOUR * 24;
pub const MILLISECS_PER_BLOCK: u64 = 6000;
pub const BLOCKS_PER_MINUTE: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const BLOCKS_PER_HOUR: BlockNumber = BLOCKS_PER_MINUTE * 60;

// Definitions for currency
pub const BASE: u128 = 10_000_000_000;
pub const DOLLARS: Balance = BASE / 100; // 100_000_000
pub const CENTS: Balance = DOLLARS / 100; // 1_000_000
pub const MILLICENTS: Balance = CENTS / 1000; // 1_000

// Court parameters
parameter_types! {
    pub const CourtPalletId: PalletId = PalletId(*b"zge/cout");
}

// Prediction Market parameters
parameter_types! {
    pub const AdvisoryBond: Balance = 25 * DOLLARS;
    pub const DisputeBond: Balance = 5 * BASE;
    pub const DisputeFactor: Balance = 2 * BASE;
    pub const DisputePeriod: BlockNumber = BLOCKS_PER_DAY;
    pub const MaxCategories: u16 = 10;
    pub const MaxDisputes: u16 = 6;
    pub const MinCategories: u16 = 2;
    pub const OracleBond: Balance = 50 * DOLLARS;
    pub const PmPalletId: PalletId = PalletId(*b"zge/pred");
    pub const ReportingPeriod: BlockNumber = BLOCKS_PER_DAY;
    pub const ValidityBond: Balance = 50 * DOLLARS;
}

// Staking parameters
parameter_types! {
    pub const DefaultBlocksPerRound: u32 = 2 * BLOCKS_PER_MINUTE as u32;
}

// Swaps parameters
parameter_types! {
    pub const ExitFee: Balance = 0;
    pub const MaxAssets: usize = MaxCategories::get() as usize + 1;
    pub const MaxInRatio: Balance = BASE / 2;
    pub const MaxOutRatio: Balance = (BASE / 3) + 1;
    pub const MaxTotalWeight: Balance = 50 * BASE;
    pub const MaxWeight: Balance = 50 * BASE;
    pub const MinLiquidity: Balance = 100 * BASE;
    pub const MinWeight: Balance = BASE;
    pub const SwapsPalletId: PalletId = PalletId(*b"zge/swap");
}
