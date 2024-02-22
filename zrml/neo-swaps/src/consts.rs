// Copyright 2023-2024 Forecasting Technologies LTD.
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

use zeitgeist_primitives::constants::BASE;

/// Numerical limit for absolute value of exp arguments (not a fixed point number).
pub(crate) const EXP_NUMERICAL_LIMIT: u128 = 10;
/// Numerical lower limit for ln arguments (fixed point number).
pub(crate) const LN_NUMERICAL_LIMIT: u128 = BASE / 10;
/// The maximum number of assets allowed in a pool.
pub(crate) const MAX_ASSETS: u16 = 128;

pub(crate) const _1: u128 = BASE;
pub(crate) const _2: u128 = 2 * _1;
pub(crate) const _3: u128 = 3 * _1;
pub(crate) const _4: u128 = 4 * _1;
pub(crate) const _5: u128 = 5 * _1;
pub(crate) const _6: u128 = 6 * _1;
pub(crate) const _7: u128 = 7 * _1;
pub(crate) const _8: u128 = 8 * _1;
pub(crate) const _9: u128 = 9 * _1;
pub(crate) const _10: u128 = 10 * _1;
pub(crate) const _11: u128 = 11 * _1;
pub(crate) const _12: u128 = 12 * _1;
pub(crate) const _14: u128 = 14 * _1;
pub(crate) const _17: u128 = 17 * _1;
pub(crate) const _20: u128 = 20 * _1;
pub(crate) const _23: u128 = 23 * _1;
pub(crate) const _24: u128 = 24 * _1;
pub(crate) const _30: u128 = 30 * _1;
pub(crate) const _36: u128 = 36 * _1;
pub(crate) const _40: u128 = 40 * _1;
pub(crate) const _70: u128 = 70 * _1;
pub(crate) const _80: u128 = 80 * _1;
pub(crate) const _100: u128 = 100 * _1;
pub(crate) const _101: u128 = 101 * _1;
pub(crate) const _444: u128 = 444 * _1;
pub(crate) const _500: u128 = 500 * _1;
pub(crate) const _777: u128 = 777 * _1;
pub(crate) const _1000: u128 = 1_000 * _1;

pub(crate) const _1_2: u128 = _1 / 2;

pub(crate) const _1_3: u128 = _1 / 3;
pub(crate) const _2_3: u128 = _2 / 3;

pub(crate) const _1_4: u128 = _1 / 4;
pub(crate) const _3_4: u128 = _3 / 4;

pub(crate) const _1_5: u128 = _1 / 5;

pub(crate) const _1_6: u128 = _1 / 6;
pub(crate) const _5_6: u128 = _5 / 6;

pub(crate) const _1_10: u128 = _1 / 10;
pub(crate) const _2_10: u128 = _2 / 10;
pub(crate) const _3_10: u128 = _3 / 10;
pub(crate) const _4_10: u128 = _4 / 10;
pub(crate) const _9_10: u128 = _9 / 10;
