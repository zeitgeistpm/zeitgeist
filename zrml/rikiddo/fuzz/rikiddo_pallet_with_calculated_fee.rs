#![no_main]
//! Fuzz test: Rikiddo pallet is called with calculated fee
//!   -> create, force fee by multiple update_volume, cost, price, all_prices, clear, destroy

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: Data| {});

#[derive(Debug, Arbitrary)]
struct Data {}
