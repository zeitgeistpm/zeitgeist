#![no_main]
//! Fuzz test: Conversion Balance -> FixedU

use libfuzzer_sys::fuzz_target;
use substrate_fixed::{FixedU128, types::extra::U33};
use zrml_rikiddo::traits::FromFixedToDecimal;

fuzz_target!(|fixed_number: u128| {
    let num = <FixedU128<U33>>::from_ne_bytes(fixed_number.to_ne_bytes());
    
    for fractional_places in 0..12u8 {
        let _ = u128::from_fixed_to_fixed_decimal(num, fractional_places);
    }
});