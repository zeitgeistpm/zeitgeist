#![cfg(test)]

use frame_support::assert_err;
use substrate_fixed::{
    types::extra::{U112, U64},
    FixedI128,
};

use crate::{
    mock::*,
    traits::RikiddoFee,
    types::{FeeSigmoid, FeeSigmoidConfig},
};

fn max_allowed_error(fixed_point_bits: u8) -> f64 {
    1.0 / (2u128 << (fixed_point_bits - 1)) as f64
}

fn sigmoid_fee(m: f64, n: f64, p: f64, r: f64) -> f64 {
    (m * (r - n)) / (p + (r - n).powi(2)).sqrt()
}

fn init_default_sigmoid_fee_struct() -> (FeeSigmoid<FixedI128<U64>>, f64, f64, f64) {
    let m = 0.01f64;
    let n = 0f64;
    let p = 2.0f64;

    let config = FeeSigmoidConfig {
        m: <FixedI128<U64>>::from_num(m),
        n: <FixedI128<U64>>::from_num(n),
        p: <FixedI128<U64>>::from_num(p),
    };

    let fee = FeeSigmoid { config };
    (fee, m, n, p)
}

#[test]
fn fee_sigmoid_overflow_r_minus_n() {
    let (mut fee, _, _, _) = init_default_sigmoid_fee_struct();
    let r = <FixedI128<U64>>::from_num(i64::MIN);
    fee.config.n = <FixedI128<U64>>::from_num(i64::MAX);
    assert_err!(fee.calculate(r), "[FeeSigmoid] Overflow during calculation: r - n");
}

#[test]
fn fee_sigmoid_overflow_m_times_r_minus_n() {
    let (mut fee, _, _, _) = init_default_sigmoid_fee_struct();
    let r = <FixedI128<U64>>::from_num(i64::MIN);
    fee.config.n = <FixedI128<U64>>::from_num(0);
    fee.config.m = <FixedI128<U64>>::from_num(i64::MAX);
    assert_err!(fee.calculate(r), "[FeeSigmoid] Overflow during calculation: m * (r-n)");
}

#[test]
fn fee_sigmoid_overflow_r_minus_n_squared() {
    let (mut fee, _, _, _) = init_default_sigmoid_fee_struct();
    let r = <FixedI128<U64>>::from_num(i64::MIN);
    fee.config.n = <FixedI128<U64>>::from_num(0);
    assert_err!(fee.calculate(r), "[FeeSigmoid] Overflow during calculation: (r-n)^2");
}

#[test]
fn fee_sigmoid_overflow_p_plus_r_minus_n_squared() {
    let (mut fee, _, _, _) = init_default_sigmoid_fee_struct();
    let r = <FixedI128<U64>>::from_num(0);
    fee.config.n = <FixedI128<U64>>::from_num(1);
    fee.config.m = <FixedI128<U64>>::from_num(0);
    fee.config.p = <FixedI128<U64>>::from_num(i64::MAX);
    assert_err!(fee.calculate(r), "[FeeSigmoid] Overflow during calculation: p + (r-n)^2");
}

#[test]
fn fee_sigmoid_overflow_numerator_div_denominator() {
    let (mut fee, _, _, _) = init_default_sigmoid_fee_struct();
    let r = <FixedI128<U64>>::from_num(0.1);
    fee.config.n = <FixedI128<U64>>::from_num(0);
    fee.config.m = <FixedI128<U64>>::from_num(i64::MAX);
    fee.config.p = <FixedI128<U64>>::from_num(-0.0099);
    assert_err!(
        fee.calculate(r),
        "[FeeSigmoid] Overflow during calculation: numerator / denominator"
    );
}

#[test]
fn fee_sigmoid_correct_result() -> Result<(), &'static str> {
    let r = 1.5f64;
    let (fee, m, n, p) = init_default_sigmoid_fee_struct();
    let fee_f64 = sigmoid_fee(m, n, p, r);
    let fee_fixed = fee.calculate(<FixedI128<U64>>::from_num(r))?;
    let fee_fixed_f64: f64 = fee_fixed.to_num();
    let difference_abs = (fee_f64 - fee_fixed_f64).abs();

    assert!(
        difference_abs <= max_allowed_error(64),
        "\nFixed result: {}\nFloat result: {}\nDifference: {}\nMax_Allowed_Difference: {}",
        fee_f64,
        fee_fixed_f64,
        difference_abs,
        max_allowed_error(64)
    );

    Ok(())
}

#[test]
fn it_is_a_dummy_test() {
    ExtBuilder::default().build().execute_with(|| {
        assert!(true);
    });
}
