#![no_main]
//! Fuzz test: EmaMarketVolume is called with estimation mode activated
//! -> create, set estimate to true, update two times, get ema, clear

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;

mod shared;
use shared::fixed_from_u128;

fuzz_target!(|data: Data| {});

#[derive(Debug, Arbitrary)]
struct Data {}
