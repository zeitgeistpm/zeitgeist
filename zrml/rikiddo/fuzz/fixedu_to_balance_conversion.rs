#![no_main]
//! Fuzz test: Conversion Balance -> FixedU

use libfuzzer_sys::fuzz_target;
use zrml_rikiddo::traits::FromFixedToDecimal;

mod shared;
use shared::fixed_from_u128;

fuzz_target!(|fixed_num: u128| {
    let num = fixed_from_u128(fixed_num);

    for fractional_places in 0..12u8 {
        let _ = u128::from_fixed_to_fixed_decimal(num, fractional_places);
    }
});
