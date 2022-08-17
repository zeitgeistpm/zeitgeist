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

//! Fuzz test: FeeSigmoid.calculate() is called
#![allow(
    // Mocks are only used for fuzzing and unit tests
    clippy::integer_arithmetic
)]
#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use substrate_fixed::{types::extra::U33, FixedI128};
use zrml_rikiddo::{traits::Fee, types::FeeSigmoid};

mod shared;
use shared::fixed_from_i128;

fuzz_target!(|data: Data| {
    let _ = data.sigmoid_fee.calculate_fee(fixed_from_i128(data.sigmoid_fee_calculate_r));
});

#[derive(Debug, Arbitrary)]
struct Data {
    sigmoid_fee_calculate_r: i128,
    sigmoid_fee: FeeSigmoid<FixedI128<U33>>,
}
