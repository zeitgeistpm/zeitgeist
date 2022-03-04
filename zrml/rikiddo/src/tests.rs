#![cfg(all(feature = "mock", test))]

use frame_support::assert_err;
use substrate_fixed::{
    traits::ToFixed,
    types::extra::{U1, U2, U3, U33, U7, U8},
    FixedI8, FixedU128, FixedU8,
};

use crate::{
    traits::{FromFixedDecimal, FromFixedToDecimal, IntoFixedDecimal, IntoFixedFromDecimal},
    utils::{convert_to_signed, convert_to_unsigned},
};

mod ema_market_volume;
mod pallet;
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
fn fixed_point_decimal_to_fixed_type_returns_correct_result() {
    // This vector contains tuples of (fixed_point_decimal, fractional_decimal_places, correct_number)
    let test_vector: Vec<(u128, u8, f64)> = vec![
        (0, 0, 0.0),
        (10_000_000_000, 10, 1.0),
        (1, 10, 0.0_000_000_001),
        (123_456_789, 10, 0.0_123_456_789),
        (9_999, 2, 99.99),
        (736_101, 2, 7_361.01),
        (133_733_333_333, 8, 1_337.33_333_333),
        (1, 1, 0.1),
        (55, 11, 0.0_000_000_006), // Rounding behavior
        (34, 11, 0.0_000_000_003), // Rounding behavior
    ];

    for (fixed_point_decimal, places, expected) in test_vector.iter() {
        let converted: FixedU128<U33> =
            fixed_point_decimal.to_fixed_from_fixed_decimal(*places).unwrap();
        assert_eq!(converted, <FixedU128<U33>>::from_num(*expected));
    }
}

#[test]
fn fixed_point_decimal_from_fixed_type_returns_correct_result() {
    // This vector contains tuples of (Fixed type, places, expected)
    // The tuples tests every logical path
    let test_vector: Vec<(FixedU128<U33>, u8, u128)> = vec![
        (32.5f64.to_fixed(), 0, 33),
        (32.25f64.to_fixed(), 0, 32),
        (200.to_fixed(), 8, 20_000_000_000),
        (200.1234f64.to_fixed(), 8, 20_012_340_000),
        (200.1234f64.to_fixed(), 2, 20_012),
        (200.1254f64.to_fixed(), 2, 20_013),
        (123.456f64.to_fixed(), 3, 123_456),
        (123.to_fixed(), 0, 123), // No decimal places in float, and places = 0
        (0.to_fixed(), 0, 0),
    ];

    for (fixed, places, expected) in test_vector.iter() {
        let converted: u128 = u128::from_fixed_to_fixed_decimal(*fixed, *places).unwrap();
        assert_eq!(converted, *expected);
    }
}

#[test]
fn fixed_type_to_fixed_point_decimal_returns_correct_result() {
    // This vector contains tuples of (Fixed type, places, expected)
    // The tuples tests every logical path
    let test_vector: Vec<(FixedU128<U33>, u8, u128)> = vec![
        (32.5f64.to_fixed(), 0, 33),
        (32.25f64.to_fixed(), 0, 32),
        (200.to_fixed(), 8, 20_000_000_000),
        (200.1234f64.to_fixed(), 8, 20_012_340_000),
        (200.1234f64.to_fixed(), 2, 20_012),
        (200.1254f64.to_fixed(), 2, 20_013),
        (123.456f64.to_fixed(), 3, 123_456),
        (123.to_fixed(), 0, 123), // No decimal places in float, and places = 0
        (0.to_fixed(), 0, 0),
    ];

    for (fixed, places, expected) in test_vector.iter() {
        let converted: u128 = fixed.to_fixed_decimal(*places).unwrap();
        assert_eq!(converted, *expected);
    }
}

#[test]
fn fixed_type_from_fixed_point_decimal_returns_correct_result() {
    // This vector contains tuples of (fixed_point_decimal, fractional_decimal_places, correct_number)
    let test_vector: Vec<(u128, u8, f64)> = vec![
        (0, 0, 0.0),
        (10_000_000_000, 10, 1.0),
        (1, 10, 0.0_000_000_001),
        (123_456_789, 10, 0.0_123_456_789),
        (9_999, 2, 99.99),
        (736_101, 2, 7_361.01),
        (133_733_333_333, 8, 1_337.33_333_333),
        (1, 1, 0.1),
        (55, 11, 0.0_000_000_006), // Rounding behavior
        (34, 11, 0.0_000_000_003), // Rounding behavior
    ];

    for (fixed_point_decimal, places, expected) in test_vector.iter() {
        let converted =
            <FixedU128<U33>>::from_fixed_decimal(*fixed_point_decimal, *places).unwrap();
        assert_eq!(converted, <FixedU128<U33>>::from_num(*expected));
    }
}
