use crate::types::{Balance, BlockNumber};
use frame_support::{parameter_types, PalletId};

// Definitions for time
pub const DAYS: BlockNumber = HOURS * 24;
pub const MILLISECS_PER_BLOCK: u64 = 6000;
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;

// Definitions for currency
pub const BASE: u128 = 10_000_000_000;
pub const DOLLARS: Balance = BASE / 100; // 100_000_000
pub const CENTS: Balance = DOLLARS / 100; // 1_000_000
pub const MILLICENTS: Balance = CENTS / 1000; // 1_000

// Prediction Market parameters
parameter_types! {
    pub const AdvisoryBond: Balance = 25 * DOLLARS;
    pub const DisputeBond: Balance = 5 * BASE;
    pub const DisputeFactor: Balance = 2 * BASE;
    pub const DisputePeriod: BlockNumber = DAYS;
    pub const MaxCategories: u16 = 8;
    pub const MaxDisputes: u16 = 6;
    pub const PmPalletId: PalletId = PalletId(*b"zge/pred");
    pub const OracleBond: Balance = 50 * DOLLARS;
    pub const ReportingPeriod: BlockNumber = DAYS;
    pub const ValidityBond: Balance = 50 * DOLLARS;
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
