use frame_support::assert_err;
use hashbrown::HashMap;
use substrate_fixed::{traits::ToFixed, types::extra::U64, FixedI128, FixedU128};

use super::{cost, max_allowed_error, Rikiddo};
use crate::{
    traits::Lmsr,
    types::{convert_to_signed, RikiddoFormulaComponents},
};

#[test]
fn rikiddo_cost_function_rejects_empty_list() {
    let rikiddo = Rikiddo::default();
    assert_err!(rikiddo.cost(&vec![]), "[RikiddoSigmoidMV] No asset balances provided");
}

#[test]
fn rikiddo_cost_function_overflow_during_summation_of_balances() {
    let rikiddo = Rikiddo::default();
    let exponent = <FixedU128<U64>>::from_num(u64::MAX);
    let param = vec![exponent, exponent];
    assert_err!(
        rikiddo.cost(&param),
        "[RikiddoSigmoidMV] Overflow during summation of asset balances"
    );
}

#[test]
fn rikiddo_cost_function_overflow_during_fee_times_balance_sum() {
    let mut rikiddo = Rikiddo::default();
    rikiddo.config.initial_fee = <FixedI128<U64>>::from_num(i64::MAX);
    let param = <FixedU128<U64>>::from_num(i64::MAX);
    assert_err!(
        rikiddo.cost(&vec![param]),
        "[RikiddoSigmoidMV] Overflow during calculation: fee * total_asset_balance"
    );
}

#[test]
fn rikiddo_cost_function_overflow_during_calculation_of_exponent() {
    let mut rikiddo = Rikiddo::default();
    rikiddo.config.initial_fee =
        <FixedI128<U64>>::from_bits(0x0000_0000_0000_0000_0000_0000_0000_0001);
    let param = <FixedU128<U64>>::from_num(i64::MAX);
    assert_err!(
        rikiddo.cost(&vec![param]),
        "[RikiddoSigmoidMV] Overflow during calculation: expontent_i = asset_balance_i / \
         denominator"
    );
}

#[test]
fn rikiddo_cost_function_overflow_during_log2e_times_biggest_exponent() {
    let mut rikiddo = Rikiddo::default();
    rikiddo.config.initial_fee =
        <FixedI128<U64>>::from_bits(0x0000_0000_0000_0000_0000_0000_0000_0003);
    rikiddo.config.log2_e = <FixedI128<U64>>::from_num(i64::MAX);
    let param = <FixedU128<U64>>::from_num(i64::MAX as u64);
    assert_err!(
        rikiddo.cost(&vec![param]),
        "[RikiddoSigmoidMV] Overflow during calculation: log2_e * biggest_exponent"
    );
}

#[test]
fn rikiddo_cost_function_overflow_during_calculation_of_required_bits_minus_one() {
    let mut rikiddo = Rikiddo::default();
    rikiddo.config.initial_fee = <FixedI128<U64>>::from_num(1);
    rikiddo.config.log2_e = <FixedI128<U64>>::from_num(i64::MAX);
    let param = <FixedU128<U64>>::from_num(i64::MAX as u64);
    let zero = <FixedU128<U64>>::from_num(0);
    assert_err!(
        rikiddo.cost(&vec![param, zero]),
        "[RikiddoSigmoidMV] Overflow during calculation: biggest_exp * log2(e) + log2(num_assets)"
    );
}

#[test]
fn rikiddo_cost_function_overflow_during_ceil_required_bits_minus_one() {
    let mut rikiddo = Rikiddo::default();
    rikiddo.config.initial_fee = <FixedI128<U64>>::from_num(1);
    rikiddo.config.log2_e = <FixedI128<U64>>::from_num(i64::MAX) + <FixedI128<U64>>::from_num(0.1);
    let param = <FixedU128<U64>>::from_num(i64::MAX as u64);
    assert_err!(
        rikiddo.cost(&vec![param]),
        "[RikiddoSigmoidMV] Overflow during calculation: ceil(biggest_exp * log2(e) + \
         log2(num_assets))"
    );
}

#[test]
fn rikiddo_cost_function_overflow_during_calculation_of_result() {
    let mut rikiddo = Rikiddo::default();
    rikiddo.config.initial_fee = <FixedI128<U64>>::from_num(1);
    rikiddo.config.log2_e = <FixedI128<U64>>::from_num(i64::MAX >> 1);
    let param = <FixedU128<U64>>::from_num(i64::MAX as u64 >> 1);
    assert_err!(
        rikiddo.cost(&vec![param, param]),
        "[RikiddoSigmoidMV] Overflow during calculation: fee * total_asset_balance * \
         ln(sum_i(e^i))"
    );
}

#[test]
fn rikiddo_cost_function_correct_result() -> Result<(), &'static str> {
    let mut rikiddo = Rikiddo::default();
    // Evaluate the cost using the optimized strategy
    let balance0 = 3.5f64;
    let balance1 = 3.6f64;
    let balance2 = 3.7f64;
    let param_f64 = vec![balance0, balance1, balance2];
    let param_fixed = vec![
        <FixedU128<U64>>::from_num(balance0),
        <FixedU128<U64>>::from_num(balance1),
        <FixedU128<U64>>::from_num(balance2),
    ];
    let mut result_fixed = rikiddo.cost(&param_fixed)?;
    let mut result_f64: f64 = cost(rikiddo.config.initial_fee.to_num(), &param_f64);
    let mut result_fixed_f64: f64 = result_fixed.to_num();
    let mut difference_abs = (result_f64 - result_fixed_f64).abs();
    assert!(
        difference_abs <= 0.0000001f64,
        "\nFixed result: {}\nFloat result: {}\nDifference: {}\nMax_Allowed_Difference: {}",
        result_fixed_f64,
        result_f64,
        difference_abs,
        max_allowed_error(64)
    );

    // Evaluate the cost using the default strategy
    rikiddo.config.initial_fee = 0.1.to_fixed();
    result_f64 = cost(rikiddo.config.initial_fee.to_num(), &param_f64);
    result_fixed = rikiddo.cost(&param_fixed)?;
    result_fixed_f64 = result_fixed.to_num();
    difference_abs = (result_f64 - result_fixed_f64).abs();
    assert!(
        difference_abs <= 0.0000001f64,
        "\nFixed result: {}\nFloat result: {}\nDifference: {}\nMax_Allowed_Difference: {}",
        result_fixed_f64,
        result_f64,
        difference_abs,
        max_allowed_error(64)
    );
    Ok(())
}

#[test]
fn rikiddo_cost_helper_does_set_all_values() -> Result<(), &'static str> {
    let rikiddo = Rikiddo::default();
    let param = <FixedU128<U64>>::from_num(1);
    let mut formula_components = RikiddoFormulaComponents::default();
    let _ = rikiddo.cost_with_forumla(&vec![param, param], &mut formula_components, true, true)?;
    let zero: FixedI128<U64> = 0.to_fixed();
    assert_ne!(formula_components.one, zero);
    assert_ne!(formula_components.fee, zero);
    assert_ne!(formula_components.sum_balances, zero);
    assert_ne!(formula_components.sum_times_fee, zero);
    assert_ne!(formula_components.emax, zero);
    assert_ne!(formula_components.sum_exp, zero);
    assert_ne!(formula_components.ln_sum_exp, zero);
    assert_ne!(formula_components.exponents, HashMap::new());
    Ok(())
}

#[test]
fn rikiddo_cost_helper_does_return_cost_minus_sum_quantities() -> Result<(), &'static str> {
    let rikiddo = Rikiddo::default();
    let param = <FixedU128<U64>>::from_num(1);
    let mut formula_components = RikiddoFormulaComponents::default();
    let quantities = &vec![param, param];
    let cost_without_sum_quantities =
        rikiddo.cost_with_forumla(&quantities, &mut formula_components, true, true)?;
    let cost_from_price_formula_times_sum_quantities =
        cost_without_sum_quantities * formula_components.sum_balances;
    let cost: FixedI128<U64> = convert_to_signed(rikiddo.cost(&quantities)?)?;
    let difference_abs = (cost - cost_from_price_formula_times_sum_quantities).abs();
    assert!(
        difference_abs <= max_allowed_error(64),
        "\nDirect cost result: {}\nReconstructed cost result: {}\nDifference: \
         {}\nMax_Allowed_Difference: {}",
        cost,
        cost_from_price_formula_times_sum_quantities,
        difference_abs,
        max_allowed_error(64)
    );
    Ok(())
}
