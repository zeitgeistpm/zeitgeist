#![cfg(test)]

use frame_support::assert_err;
use substrate_fixed::{types::extra::U7, FixedI8, FixedU8};

use crate::{
    mock::*,
    types::{convert_to_signed, convert_to_unsigned},
};

mod ema_market_volume;
mod rikiddo_sigmoid_mv;
mod sigmoid_fee;

fn max_allowed_error(fractional_bits: u8) -> f64 {
    1.0 / (1u128 << (fractional_bits - 1)) as f64
}

#[test]
fn convert_to_unsigned_error_when_msb_is_set() {
    let num = <FixedI8<U7>>::from_num(-1.42f32);
    assert_err!(
        convert_to_unsigned::<FixedI8<U7>, FixedU8<U7>>(num),
        "Signed fixed point to unsigned fixed point number conversion failed: MSB is set"
    );
}

#[test]
fn it_is_a_dummy_test() {
    ExtBuilder::default().build().execute_with(|| {
        assert!(true);
    });
}
