use frame_support::assert_err;
use substrate_fixed::{traits::ToFixed, types::extra::U64, FixedI128};

use super::{initial_outstanding_assets, ln_exp_sum, Rikiddo};
use crate::{
    constants::INITIAL_FEE,
    traits::Lmsr,
    types::{
        convert_to_signed, EmaMarketVolume, FeeSigmoid, RikiddoConfig, RikiddoFormulaComponents,
    },
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
fn rikiddo_initial_outstanding_assets_returns_correct_result() {
    let rikiddo = Rikiddo::default();
    let num_assets = 4u32;
    let subsidy = 1000u32;
    let outstanding_assets =
        rikiddo.initial_outstanding_assets(num_assets, subsidy.to_fixed()).unwrap();
    let outstanding_assets_f64 =
        initial_outstanding_assets(num_assets, subsidy.into(), rikiddo.config.initial_fee.to_num());
    let difference_abs = (outstanding_assets_f64 - outstanding_assets.to_num::<f64>()).abs();
    assert!(
        difference_abs <= 0.000001f64,
        "\nFixed result: {}\nFloat result: {}\nDifference: {}\nMax_Allowed_Difference: {}",
        outstanding_assets,
        outstanding_assets_f64,
        difference_abs,
        0.000001f64
    );
}

#[test]
fn rikiddo_optimized_ln_sum_exp_strategy_exponent_subtract_overflow() {
    let rikiddo = Rikiddo::default();
    let param = vec![<FixedI128<U64>>::from_num(1i64 << 63)];
    assert_err!(
        rikiddo.optimized_cost_strategy(
            &param,
            &<FixedI128<U64>>::from_num(1i64 << 62),
            &mut RikiddoFormulaComponents::default(),
            false
        ),
        "[RikiddoSigmoidFee] Overflow during calculation: current_exponent - biggest_exponent"
    );
}

#[test]
fn rikiddo_optimized_ln_sum_exp_strategy_sum_exp_i_overflow() {
    let rikiddo = Rikiddo::default();
    let exponent = <FixedI128<U64>>::from_num(42.7f64);
    let param = vec![exponent, exponent, exponent];
    assert_err!(
        rikiddo.optimized_cost_strategy(
            &param,
            &<FixedI128<U64>>::from_num(0),
            &mut RikiddoFormulaComponents::default(),
            false
        ),
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
        rikiddo.optimized_cost_strategy(
            &param,
            &biggest_exponent,
            &mut RikiddoFormulaComponents::default(),
            false
        ),
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
    // Evaluate the result of the opimized cost strategy
    let result_fixed = rikiddo.optimized_cost_strategy(
        &param_fixed,
        &param_fixed[2],
        &mut RikiddoFormulaComponents::default(),
        false,
    )?;
    let result_f64: f64 = ln_exp_sum(&param_f64);
    let result_fixed_f64: f64 = result_fixed.to_num();
    let difference_abs = (result_f64 - result_fixed_f64).abs();
    // The fixed calculation seems to be quite errorneous: Difference = 0.00000007886511177446209
    assert!(
        difference_abs <= 0.000001f64,
        "\nFixed result: {}\nFloat result: {}\nDifference: {}\nMax_Allowed_Difference: {}",
        result_fixed_f64,
        result_f64,
        difference_abs,
        0.000001f64
    );

    Ok(())
}
