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

#![allow(dead_code)]

use substrate_fixed::{types::extra::U33, FixedI128, FixedU128};

#[inline(always)]
pub(super) fn fixed_from_i128(from: i128) -> FixedI128<U33> {
    FixedI128::<U33>::from_ne_bytes(from.to_ne_bytes())
}

#[inline(always)]
pub(super) fn fixed_from_u128(from: u128) -> FixedU128<U33> {
    FixedU128::<U33>::from_ne_bytes(from.to_ne_bytes())
}
