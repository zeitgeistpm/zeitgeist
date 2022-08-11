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

//! Fuzz test: Conversion Balance -> FixedU
#![allow(
    // Mocks are only used for fuzzing and unit tests
    clippy::integer_arithmetic
)]
#![no_main]

use libfuzzer_sys::fuzz_target;
use zrml_rikiddo::traits::FromFixedToDecimal;

mod shared;
use shared::fixed_from_u128;

fuzz_target!(|fixed_num: u128| {
    let num = fixed_from_u128(fixed_num);

    for fractional_places in 0..12u8 {
        let _ = u128::from_fixed_to_fixed_decimal(num, fractional_places);
    }
});
