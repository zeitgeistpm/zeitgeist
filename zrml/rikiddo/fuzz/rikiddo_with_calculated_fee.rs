#![no_main]
//! Fuzz test: Rikiddo is called with calculated fee
//! -> create, force EmaMarketVolume, cost, price, all_prices

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;

mod shared;
use shared::fixed_from_u128;

fuzz_target!(|data: Data| {});

#[derive(Debug, Arbitrary)]
struct Data {}
