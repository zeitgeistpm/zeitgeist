#![allow(
    // Constants parameters inside `parameter_types!` already check
    // arithmetic operations at compile time
    clippy::integer_arithmetic
)]

#[cfg(feature = "mock")]
pub mod mock;
pub mod ztg;

use crate::types::{Balance, BlockNumber};
use frame_support::{parameter_types, PalletId};

// Definitions for time
pub const BLOCKS_PER_DAY: BlockNumber = BLOCKS_PER_HOUR * 24;
pub const MILLISECS_PER_BLOCK: u32 = 12000;
pub const BLOCKS_PER_MINUTE: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const BLOCKS_PER_HOUR: BlockNumber = BLOCKS_PER_MINUTE * 60;

// Definitions for currency
pub const BASE: u128 = 10_000_000_000;
pub const CENT: Balance = BASE / 100; // 100_000_000
pub const MILLI: Balance = CENT / 10; //  10_000_000
pub const MICRO: Balance = MILLI / 1000; // 10_000

pub const fn deposit(items: u32, bytes: u32) -> Balance {
    items as Balance * 20 * BASE + (bytes as Balance) * 100 * MILLI
}

// Rikiddo and TokensConfig
parameter_types! {
    pub const BalanceFractionalDecimals: u8 = {
        let mut base = BASE;
        let mut counter: u8 = 0;

        while base >= 10 {
            base /= 10;
            counter += 1;
        }

        counter
    };
}

// Authorized
/// Pallet identifier, mainly used for named balance reserves.
pub const AUTHORIZED_PALLET_ID: PalletId = PalletId(*b"zge/atzd");

// Court
/// Pallet identifier, mainly used for named balance reserves.
pub const COURT_PALLET_ID: PalletId = PalletId(*b"zge/cout");

// Liqudity Mining
/// Pallet identifier, mainly used for named balance reserves.
pub const LM_PALLET_ID: PalletId = PalletId(*b"zge/lymg");

// Prediction Markets
/// Max. categories in a prediction market.
pub const MAX_CATEGORIES: u16 = 10;
/// Pallet identifier, mainly used for named balance reserves.
pub const PM_PALLET_ID: PalletId = PalletId(*b"zge/pred");

// Simple Disputes
pub const SD_PALLET_ID: PalletId = PalletId(*b"zge/sedp");

// Swaps
/// Max. assets in a swap pool.
pub const MAX_ASSETS: u16 = MAX_CATEGORIES + 1;
/// Pallet identifier, mainly used for named balance reserves.
pub const SWAPS_PALLET_ID: PalletId = PalletId(*b"zge/swap");
