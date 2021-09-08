//! Fuzz test: EmaMarketVolume is called during second state.
//! -> change state (update), update, get ema, clear
#![allow(
    // Mocks are only used for fuzzing and unit tests
    clippy::integer_arithmetic
)]
#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use substrate_fixed::{types::extra::U33, FixedU128};
use zrml_rikiddo::{
    traits::MarketAverage,
    types::{EmaMarketVolume, TimestampedVolume},
};

mod shared;
use shared::fixed_from_u128;

fuzz_target!(|data: Data| {
    let mut emv = data.ema_market_volume;
    let first_timestamped_volume =
        TimestampedVolume { timestamp: 0, volume: fixed_from_u128(data.first_update_volume) };
    let _ = emv.update_volume(&first_timestamped_volume);
    let _ = emv.update_volume(&data.second_update_volume);
});

#[derive(Debug, Arbitrary)]
struct Data {
    first_update_volume: u128,
    second_update_volume: TimestampedVolume<FixedU128<U33>>,
    ema_market_volume: EmaMarketVolume<FixedU128<U33>>,
}
