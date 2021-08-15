#![no_main]
//! Fuzz test: Conversion FixedI -> FixedU

use libfuzzer_sys::fuzz_target;
use substrate_fixed::{types::extra::U33, FixedI128, FixedU128};
use zrml_rikiddo::types::convert_to_unsigned;

fuzz_target!(|fixed_number: i128| {
    let num = <FixedI128<U33>>::from_ne_bytes(fixed_number.to_ne_bytes());
    let _: Result<FixedU128<U33>, &'static str> = convert_to_unsigned(num);
});