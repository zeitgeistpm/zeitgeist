#![no_main]
//! Fuzz test: EmaMarketVolume is called with estimation mode activated
//! -> Configure the struct in a way that it estimates the ema at the second update, update

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
    second_update_volume: u128,
    smoothing: u128,
    third_update_volume: TimestampedVolume<FixedU128<U33>>,
    ema_market_volume: EmaMarketVolume<FixedU128<U33>>,
}
