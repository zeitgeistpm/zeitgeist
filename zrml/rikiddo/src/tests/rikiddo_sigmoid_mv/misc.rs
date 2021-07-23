use frame_support::assert_err;
use substrate_fixed::{types::extra::U64, FixedI128};

use super::{ln_exp_sum, max_allowed_error, Rikiddo};
use crate::{
    constants::INITIAL_FEE,
    types::{convert_to_signed, EmaMarketVolume, FeeSigmoid, RikiddoConfig},
};

#[test]
fn rikiddo_default_does_not_panic() -> Result<(), &'static str> {
    let default_rikiddo = Rikiddo::default();
    let rikiddo = Rikiddo::new(
        RikiddoConfig::new(convert_to_signed(INITIAL_FEE)?),
        FeeSigmoid::default(),
        EmaMarketVolume::default(),
        EmaMarketVolume::default(),
    );
    assert_eq!(default_rikiddo, rikiddo);
    Ok(())
}

#[test]
fn rikiddo_default_ln_sum_exp_strategy_exp_i_overflow() {
    let rikiddo = Rikiddo::default();
    let param = vec![<FixedI128<U64>>::from_num(100u64)];
    assert_err!(
        rikiddo.default_cost_strategy(&param),
        "[RikiddoSigmoidMV] Error during calculation: exp(i) in ln sum_i(exp^i)"
    );
}

#[test]
fn rikiddo_default_ln_sum_exp_strategy_sum_exp_i_overflow() {
    let rikiddo = Rikiddo::default();
    let exponent = <FixedI128<U64>>::from_num(42.7f64);
    let param = vec![exponent, exponent, exponent];
    assert_err!(
        rikiddo.default_cost_strategy(&param),
        "[RikiddoSigmoidMV] Overflow during calculation: sum_i(e^i)"
    );
}

#[test]
fn rikiddo_default_ln_sum_exp_strategy_ln_zero() {
    let rikiddo = Rikiddo::default();
    let param = vec![];
    assert_err!(
        rikiddo.default_cost_strategy(&param),
        "[RikiddoSigmoidMV] ln(exp_sum), exp_sum <= 0"
    );
}

#[test]
fn rikiddo_optimized_ln_sum_exp_strategy_exponent_subtract_overflow() {
    let rikiddo = Rikiddo::default();
    let param = vec![<FixedI128<U64>>::from_num(1i64 << 63)];
    assert_err!(
        rikiddo.optimized_cost_strategy(&param, &<FixedI128<U64>>::from_num(1i64 << 62)),
        "[RikiddoSigmoidFee] Overflow during calculation: current_exponent - biggest_exponent"
    );
}

#[test]
fn rikiddo_optimized_ln_sum_exp_strategy_sum_exp_i_overflow() {
    let rikiddo = Rikiddo::default();
    let exponent = <FixedI128<U64>>::from_num(42.7f64);
    let param = vec![exponent, exponent, exponent];
    assert_err!(
        rikiddo.optimized_cost_strategy(&param, &<FixedI128<U64>>::from_num(0)),
        "[RikiddoSigmoidFee] Overflow during calculation: sum_i(e^(i - biggest_exponent))"
    );
}

#[test]
fn rikiddo_optimized_ln_sum_exp_strategy_result_overflow() {
    let rikiddo = Rikiddo::default();
    let biggest_exponent = <FixedI128<U64>>::from_num(i64::MAX);
    let exponent = biggest_exponent - <FixedI128<U64>>::from_num(0.0000001f64);
    let param = vec![exponent, exponent, exponent];
    assert_err!(
        rikiddo.optimized_cost_strategy(&param, &biggest_exponent),
        "[RikiddoSigmoidMV] Overflow during calculation: biggest_exponent + ln(exp_sum) \
         (optimized)"
    );
}

#[test]
fn rikiddo_ln_sum_exp_strategies_return_correct_results() -> Result<(), &'static str> {
    let rikiddo = Rikiddo::default();
    let exponent0 = 3.5f64;
    let exponent1 = 4.5f64;
    let exponent2 = 5.5f64;
    let param_f64 = vec![exponent0, exponent1, exponent2];
    let param_fixed = vec![
        <FixedI128<U64>>::from_num(exponent0),
        <FixedI128<U64>>::from_num(exponent1),
        <FixedI128<U64>>::from_num(exponent2),
    ];
    // Evaluate the result of the default cost strategy
    let mut result_fixed = rikiddo.default_cost_strategy(&param_fixed)?;
    let result_f64: f64 = ln_exp_sum(&param_f64);
    let mut result_fixed_f64: f64 = result_fixed.to_num();
    let mut difference_abs = (result_f64 - result_fixed_f64).abs();
    // The fixed calculation seems to be quite errorneous: Difference = 0.00000007886511177446209
    assert!(
        difference_abs <= 0.000001f64,
        "\nFixed result: {}\nFloat result: {}\nDifference: {}\nMax_Allowed_Difference: {}",
        result_fixed_f64,
        result_f64,
        difference_abs,
        max_allowed_error(64)
    );

    // Evaluate the result of the optimize cost strategy
    result_fixed = rikiddo.optimized_cost_strategy(&param_fixed, &param_fixed[2])?;
    result_fixed_f64 = result_fixed.to_num();
    difference_abs = (result_f64 - result_fixed_f64).abs();
    assert!(
        difference_abs <= 0.00000001f64,
        "\nFixed result: {}\nFloat result: {}\nDifference: {}\nMax_Allowed_Difference: {}",
        result_fixed_f64,
        result_f64,
        difference_abs,
        max_allowed_error(64)
    );

    Ok(())
}
