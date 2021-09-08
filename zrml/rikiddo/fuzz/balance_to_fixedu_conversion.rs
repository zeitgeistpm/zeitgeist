//! Fuzz test: Conversion Balance -> FixedU
#![allow(
    // Mocks are only used for fuzzing and unit tests
    clippy::integer_arithmetic
)]
#![no_main]

use libfuzzer_sys::fuzz_target;
use substrate_fixed::{types::extra::U33, FixedU128};
use zrml_rikiddo::traits::FromFixedDecimal;

fuzz_target!(|balance: u128| {
    for i in 0..12u8 {
        let _ = <FixedU128<U33>>::from_fixed_decimal(balance, i);
    }
});
