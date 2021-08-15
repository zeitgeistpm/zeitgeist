#![no_main]
//! Fuzz test: Conversion FixedI -> FixedU

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;

mod shared;
use shared::fixed_from_i128;

fuzz_target!(|data: Data| {
});

#[derive(Debug, Arbitrary)]
struct Data {
}
