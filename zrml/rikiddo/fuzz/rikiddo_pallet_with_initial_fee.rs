#![no_main]
//! Fuzz test: Rikiddo pallet is called with initial fee
//! -> create, fee, cost, price, all_prices, clear, destroy

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;

mod shared;
use shared::fixed_from_u128;

fuzz_target!(|data: Data| {});

#[derive(Debug, Arbitrary)]
struct Data {}
