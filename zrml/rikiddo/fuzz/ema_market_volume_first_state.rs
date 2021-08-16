#![no_main]
//! Fuzz test: EmaMarketVolume is called during first state
//! -> update_volume

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use substrate_fixed::{FixedU128, types::extra::U33};
use zrml_rikiddo::{traits::MarketAverage, types::{EmaMarketVolume, TimestampedVolume}};

fuzz_target!(|data: Data| {
    let mut emv = data.ema_market_volume;
    let _ = emv.update_volume(&data.update_volume);
});

#[derive(Debug, Arbitrary)]
struct Data {
    update_volume: TimestampedVolume<FixedU128<U33>>,
    ema_market_volume: EmaMarketVolume<FixedU128<U33>>
}
