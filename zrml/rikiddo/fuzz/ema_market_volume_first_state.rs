//! Fuzz test: EmaMarketVolume is called during first state
//! -> update_volume
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

fuzz_target!(|data: Data| {
    let mut emv = data.ema_market_volume;
    let _ = emv.update_volume(&data.update_volume);
});

#[derive(Debug, Arbitrary)]
struct Data {
    update_volume: TimestampedVolume<FixedU128<U33>>,
    ema_market_volume: EmaMarketVolume<FixedU128<U33>>,
}
