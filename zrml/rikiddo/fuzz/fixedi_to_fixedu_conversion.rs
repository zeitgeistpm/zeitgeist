#![no_main]
//! Fuzz test: Conversion FixedI -> FixedU

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use substrate_fixed::{FixedI128, FixedU128, types::extra::U33};
use zrml_rikiddo::types::convert_to_unsigned;

fuzz_target!(|data: Data| {
    let num = <FixedI128<U33>>::from_ne_bytes(data.fixed_number.to_ne_bytes());
    let _: Result<FixedU128<U33>, &'static str> = convert_to_unsigned(num);
});

#[derive(Debug, Arbitrary)]
struct Data {
    fixed_number: i128
}
