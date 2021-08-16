#![no_main]
//! Fuzz test: Conversion FixedU -> FixedI

use libfuzzer_sys::fuzz_target;
use substrate_fixed::{types::extra::U33, FixedI128};
use zrml_rikiddo::types::convert_to_signed;

mod shared;
use shared::fixed_from_u128;

fuzz_target!(|fixed_num: u128| {
    let _: Result<FixedI128<U33>, &'static str> = convert_to_signed(fixed_from_u128(fixed_num));
});
