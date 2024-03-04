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
