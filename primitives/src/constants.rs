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
pub const BLOCKS_PER_YEAR: BlockNumber = (BLOCKS_PER_DAY * 36525) / 100;
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

// Global Disputes
pub const GLOBAL_DISPUTES_PALLET_ID: PalletId = PalletId(*b"zge/gldp");
/// Lock identifier, mainly used for the locks on the accounts.
pub const GLOBAL_DISPUTES_LOCK_ID: [u8; 8] = *b"zge/gdlk";

// Liqudity Mining
/// Pallet identifier, mainly used for named balance reserves.
pub const LM_PALLET_ID: PalletId = PalletId(*b"zge/lymg");

// Prediction Markets
/// The maximum allowed market life time, measured in blocks.
pub const MAX_MARKET_LIFETIME: BlockNumber = 365 * BLOCKS_PER_DAY;
/// Max. categories in a prediction market.
pub const MAX_CATEGORIES: u16 = 64;
/// The dispute_duration is time where users can dispute the outcome.
/// Minimum block period for a dispute.
pub const MIN_DISPUTE_DURATION: BlockNumber = 12 * BLOCKS_PER_HOUR;
/// Minimum block period for oracle_duration.
pub const MIN_ORACLE_DURATION: BlockNumber = BLOCKS_PER_HOUR;
/// Maximum block period for a dispute.
pub const MAX_DISPUTE_DURATION: BlockNumber = 30 * BLOCKS_PER_DAY;
/// Maximum block period for an grace_period.
/// The grace_period is a delay between the point where the market closes and the point where the oracle may report.
pub const MAX_GRACE_PERIOD: BlockNumber = 365 * BLOCKS_PER_DAY;
/// Maximum block period for an oracle_duration.
/// The oracle_duration is a duration where the oracle has to submit its report.
pub const MAX_ORACLE_DURATION: BlockNumber = 14 * BLOCKS_PER_DAY;

/// Pallet identifier, mainly used for named balance reserves.
pub const PM_PALLET_ID: PalletId = PalletId(*b"zge/pred");

// Simple Disputes
pub const SD_PALLET_ID: PalletId = PalletId(*b"zge/sedp");

// Swaps
/// Max. assets in a swap pool.
pub const MAX_ASSETS: u16 = MAX_CATEGORIES + 1;
/// Pallet identifier, mainly used for named balance reserves.
pub const SWAPS_PALLET_ID: PalletId = PalletId(*b"zge/swap");

// Treasury
/// Pallet identifier, used to derive treasury account
pub const TREASURY_PALLET_ID: PalletId = PalletId(*b"zge/tsry");
