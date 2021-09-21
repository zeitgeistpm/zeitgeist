//! Fuzz test: FeeSigmoid.calculate() is called
#![allow(
    // Mocks are only used for fuzzing and unit tests
    clippy::integer_arithmetic
)]
#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use substrate_fixed::{types::extra::U33, FixedI128};
use zrml_rikiddo::{traits::Fee, types::FeeSigmoid};

mod shared;
use shared::fixed_from_i128;

fuzz_target!(|data: Data| {
    let _ = data.sigmoid_fee.calculate_fee(fixed_from_i128(data.sigmoid_fee_calculate_r));
});

#[derive(Debug, Arbitrary)]
struct Data {
    sigmoid_fee_calculate_r: i128,
    sigmoid_fee: FeeSigmoid<FixedI128<U33>>,
}
