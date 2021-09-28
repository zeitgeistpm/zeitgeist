//! Fuzz test: EmaMarketVolume is called during third and last state.
//! -> change state (two updates with specific configuration), update
#![allow(
    // Mocks are only used for fuzzing and unit tests
    clippy::integer_arithmetic
)]
#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use substrate_fixed::{
    traits::LossyInto,
    types::extra::{U127, U33},
    FixedU128,
};
use zrml_rikiddo::{
    traits::MarketAverage,
    types::{EmaMarketVolume, Timespan, TimestampedVolume},
};

mod shared;
use shared::fixed_from_u128;

fuzz_target!(|data: Data| {
    let mut emv = data.ema_market_volume;
    // We need smoothing-1 volumes to get into the third state
    let between_zero_and_two = <FixedU128<U127>>::from_ne_bytes(data.smoothing.to_ne_bytes());
    emv.config.smoothing = between_zero_and_two.lossy_into();
    emv.config.ema_period = Timespan::Seconds(0);
    emv.config.ema_period_estimate_after = None;
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
