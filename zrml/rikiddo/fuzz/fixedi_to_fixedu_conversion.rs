//! Fuzz test: Conversion FixedI -> FixedU
#![allow(
    // Mocks are only used for fuzzing and unit tests
    clippy::integer_arithmetic
)]
#![no_main]

use libfuzzer_sys::fuzz_target;
use substrate_fixed::{types::extra::U33, FixedU128};
use zrml_rikiddo::types::convert_to_unsigned;

mod shared;
use shared::fixed_from_i128;

fuzz_target!(|fixed_num: i128| {
    let _: Result<FixedU128<U33>, &'static str> = convert_to_unsigned(fixed_from_i128(fixed_num));
});
