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

//! Fuzz test: EmaMarketVolume is called with estimation mode activated
//! -> Configure the struct in a way that it estimates the ema at the second update, update
#![allow(
    // Mocks are only used for fuzzing and unit tests
    clippy::integer_arithmetic
)]
#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;

mod shared;
use shared::fixed_from_u128;
use substrate_fixed::{
    traits::LossyInto,
    types::extra::{U127, U33},
    FixedU128,
};
use zrml_rikiddo::{
    traits::MarketAverage,
    types::{EmaMarketVolume, Timespan, TimestampedVolume},
};

fuzz_target!(|data: Data| {
    let mut emv = data.ema_market_volume;
    // We need smoothing-1 volumes to get into the third state
    let between_zero_and_two = <FixedU128<U127>>::from_ne_bytes(data.smoothing.to_ne_bytes());
    emv.config.smoothing = between_zero_and_two.lossy_into();
    emv.config.ema_period_estimate_after = Some(Timespan::Seconds(0));
    let first_timestamped_volume =
        TimestampedVolume { timestamp: 0, volume: fixed_from_u128(data.first_update_volume) };
    let second_timestamped_volume =
        TimestampedVolume { timestamp: 1, volume: fixed_from_u128(data.first_update_volume) };
    let _ = emv.update_volume(&first_timestamped_volume);
    let _ = emv.update_volume(&second_timestamped_volume);
    let _ = emv.update_volume(&data.third_update_volume);
});

#[derive(Debug, Arbitrary)]
struct Data {
    first_update_volume: u128,
    smoothing: u128,
    third_update_volume: TimestampedVolume<FixedU128<U33>>,
    ema_market_volume: EmaMarketVolume<FixedU128<U33>>,
}
