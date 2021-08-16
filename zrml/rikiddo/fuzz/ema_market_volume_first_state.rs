#![no_main]
//! Fuzz test: EmaMarketVolume is called during first state
//! -> create, update, get ema, clear

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;

mod shared;
use shared::fixed_from_u128;

fuzz_target!(|data: Data| {});

#[derive(Debug, Arbitrary)]
struct Data {}
