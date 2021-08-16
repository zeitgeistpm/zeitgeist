#![no_main]
//! Fuzz test: EmaMarketVolume is called during third and last state.
//! -> create, set to third state, update, get ema, clear

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use substrate_fixed::{types::extra::U33, FixedU128};

mod shared;
use shared::fixed_from_u128;
use zrml_rikiddo::types::EmaMarketVolume;

fuzz_target!(|data: Data| {});

#[derive(Debug, Arbitrary)]
struct Data {
    // ema_struct: EmaMarketVolume<FixedU128<U33>>,
    // update_value: FixedU128<U33>,
}
