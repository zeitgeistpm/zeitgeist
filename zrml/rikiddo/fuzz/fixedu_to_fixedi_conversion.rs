#![no_main]
//! Fuzz test: Conversion FixedU -> FixedI

use libfuzzer_sys::fuzz_target;
use substrate_fixed::{types::extra::U33, FixedI128, FixedU128};
use zrml_rikiddo::types::convert_to_signed;

fuzz_target!(|fixed_number: u128| {
    let num = <FixedU128<U33>>::from_ne_bytes(fixed_number.to_ne_bytes());
    let _: Result<FixedI128<U33>, &'static str> = convert_to_signed(num);
});