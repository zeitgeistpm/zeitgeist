#![cfg(test)]

use frame_support::assert_err;
use substrate_fixed::{
    types::extra::{U1, U2, U3, U7, U8},
    FixedI8, FixedU8,
};

use crate::{mock::ExtBuilder, types::{convert_to_signed, convert_to_unsigned}};

mod ema_market_volume;
mod rikiddo_sigmoid_mv;
mod sigmoid_fee;

fn max_allowed_error(fractional_bits: u8) -> f64 {
    1.0 / (1u128 << (fractional_bits - 1)) as f64
}

#[test]
fn convert_signed_to_unsigned_fails() {
    let num = <FixedI8<U8>>::from_num(-0.5f32);
    assert_err!(
        convert_to_unsigned::<FixedI8<U8>, FixedU8<U8>>(num),
        "Cannot convert negative signed number into unsigned number"
    );
}

#[test]
fn convert_number_does_not_fit_in_destination_type() {
    let num = <FixedU8<U7>>::from_num(1);
    assert_err!(
        convert_to_signed::<FixedU8<U7>, FixedI8<U7>>(num),
        "Fixed point conversion failed: FROM type does not fit in TO type"
    );
}

#[test]
fn convert_unsigned_to_signed_returns_correct_result() -> Result<(), &'static str> {
    // lossless - exact fit
    let num1 = <FixedU8<U2>>::from_num(4.75);
    let num1_converted: FixedI8<U3> = convert_to_signed(num1)?;
    assert_eq!(num1_converted, num1);
    // lossy - loses fractional bits
    let num2 = <FixedU8<U2>>::from_num(4.75);
    let num2_converted: FixedI8<U1> = convert_to_signed(num2)?;
    assert_eq!(num2_converted.to_num::<f32>(), 4.5f32);
    Ok(())
}

#[test]
fn convert_signed_to_unsigned_returns_correct_result() -> Result<(), &'static str> {
    // lossless - exact fit
    let num1 = <FixedI8<U2>>::from_num(4.75);
    let num1_converted: FixedU8<U3> = convert_to_unsigned(num1)?;
    assert_eq!(num1_converted, num1);
    // lossy - loses fractional bits
    let num2 = <FixedI8<U2>>::from_num(4.75);
    let num2_converted: FixedU8<U1> = convert_to_unsigned(num2)?;
    assert_eq!(num2_converted.to_num::<f32>(), 4.5f32);
    Ok(())
}

#[test]
fn it_is_a_dummy_test() {
    ExtBuilder::default().build().execute_with(|| {
        assert!(true);
    });
}