#![cfg(test)]

use substrate_fixed::FixedI128;
use substrate_fixed::types::extra::U64;

use crate::mock::*;
use crate::traits::LsdlmsrFee;
use crate::types::{FeeSigmoid, FeeSigmoidConfig};

// TODO: Test fee calculation + different overflow scenarios + default values

fn max_allowed_error(fixed_point_bits: u8) -> f64 {
    1.0 / (2u128 << (fixed_point_bits - 1)) as f64
}

fn sigmoid_fee(m: f64, n: f64, p: f64, r: f64) -> f64 {
    (m * (r - n)) / (p + (r - n).powi(2)).sqrt()
}

#[test]
fn fee_sigmoid_overflow_r_minus_n() {

}

#[test]
fn fee_sigmoid_overflow_m_times_r_minus_n() {

}

#[test]
fn fee_sigmoid_overflow_r_minus_n_squared() {

}

#[test]
/// Some text
fn fee_sigmoid_overflow_p_plus_r_minus_n_squared() {

}

#[test]
fn fee_sigmoid_overflow_numerator_div_denominator() {

}

#[test]
fn fee_sigmoid_correct_result() -> Result<(), &'static str> {
    let r = 1.5f64;
    let m = 0.01f64;
    let n = 0f64;
    let p = 2.0f64;

    let config = FeeSigmoidConfig {
        m: <FixedI128<U64>>::from_num(m),
        n: <FixedI128<U64>>::from_num(n),
        p: <FixedI128<U64>>::from_num(p),
    };

    let fee = FeeSigmoid { config };
    let fee_f64 = sigmoid_fee(m, n, p, r);
    let fee_fixed = fee.calculate(<FixedI128<U64>>::from_num(r))?;
    let fee_fixed_f64: f64 = fee_fixed.to_num();
    let difference_abs = (fee_f64 - fee_fixed_f64).abs();

    assert!(difference_abs <= max_allowed_error(64), "\nFixed result: {}\nFloat result: {}\n\
        Difference: {}\nMax_Allowed_Difference: {}", fee_f64, fee_fixed_f64, difference_abs,
        max_allowed_error(64));

    Ok(())
}

#[test]
fn it_is_a_dummy_test() {
    ExtBuilder::default().build().execute_with(|| {
        assert!(true);
    });
}
