use frame_support::{assert_err, weights::constants::WEIGHT_PER_MICROS};
use substrate_fixed::{FixedI128, FixedI32, FixedU128, traits::{FixedSigned, LossyFrom, ToFixed}, types::{I9F23, U1F127, extra::{U127, U31, U64}}};

use super::{ema_create_test_struct, max_allowed_error, price_first_quotient, Rikiddo};
use crate::{
    constants::INITIAL_FEE,
    traits::{Lmsr, MarketAverage, RikiddoMV},
    types::{
        convert_to_signed, EmaMarketVolume, FeeSigmoid, RikiddoConfig, RikiddoFormulaComponents,
        RikiddoSigmoidMV, TimestampedVolume,
    },
};

fn check_if_exponent_in_formula_components(helper: u8) {
    let rikiddo = Rikiddo::default();
    let param = vec![<FixedI128<U64>>::from_num(100)];
    let result = {
        if helper == 1 {
            rikiddo.price_helper_first_quotient(&param, param[0], &RikiddoFormulaComponents::default())
        } else {
            rikiddo.price_helper_second_quotient(&param, param[0], &RikiddoFormulaComponents::default())
        }
    };
    assert_err!(
        result,
        "[RikiddoSigmoidMV] Cannot find exponent of asset balance in question \
            in RikiddoFormulaComponents HashMap"
    );
}

// --- Tests for price_helper_first_quotient ---

#[test]
fn price_helper_first_quotient_exponent_not_found() {
    check_if_exponent_in_formula_components(1)
}

#[test]
fn price_helper_first_quotient_overflow_exponent_sub_exp_qj() {
    let rikiddo = Rikiddo::default();
    let formula_components = &mut <RikiddoFormulaComponents<FixedI128<U64>>>::default();
    let param = vec![<FixedI128<U64>>::from(i64::MAX), <FixedI128<U64>>::from(-i64::MAX >> 5)];
    formula_components.exponents.insert(param[0], i64::MAX.to_fixed());
    formula_components.exponents.insert(param[1], <FixedI128<U64>>::from(-i64::MAX >> 5));
    formula_components.sum_times_fee = 0.1.to_fixed();
    assert_err!(
        rikiddo.price_helper_first_quotient(&param, param[0], &formula_components),
        "[RikiddoSigmoidMV] Overflow during calculation: exponent - exponent_balance_in_question"
    );
}

#[test]
fn price_helper_first_quotient_returns_correct_result() -> Result<(), &'static str> {
    let rikiddo = Rikiddo::default();
    let formula_components = &mut <RikiddoFormulaComponents<FixedI128<U64>>>::default();
    let param_f64 = vec![520.19, 480.81];
    let param =
        vec![<FixedI128<U64>>::from_num(param_f64[0]), <FixedI128<U64>>::from_num(param_f64[1])];
    let param_u =
        vec![<FixedU128<U64>>::from_num(param_f64[0]), <FixedU128<U64>>::from_num(param_f64[1])];
    // This fills the formula_components with the correct values
    let _ = rikiddo.cost_with_forumla(&param_u, formula_components, true, true, true);
    let rikiddo_price =
        rikiddo.price_helper_first_quotient(&param, param[0], &formula_components)?;
    let rikiddo_price_f64 =
        price_first_quotient(rikiddo.config.initial_fee.to_num(), &param_f64, param_f64[0]);
    let difference_abs = (rikiddo_price.to_num::<f64>() - rikiddo_price_f64).abs();
    assert!(
        difference_abs <= max_allowed_error(63),
        "\nprice first quotient result (fixed): {}\nprice first quotient result (f64): \
         {}\nDifference: {}\nMax_Allowed_Difference: {}",
        rikiddo_price,
        rikiddo_price_f64,
        difference_abs,
        max_allowed_error(64)
    );
    Ok(())
}

// --- Tests for price_helper_second_quotient ---

#[test]
fn price_helper_second_quotient_exponent_not_found() {
    check_if_exponent_in_formula_components(2)
}

#[test]
fn price_helper_second_quotient_reduced_exp_not_found() {
    let rikiddo = Rikiddo::default();
    let param = vec![<FixedI128<U64>>::from_num(100)];
    let formula_components = &mut <RikiddoFormulaComponents<FixedI128<U64>>>::default();
    formula_components.exponents.insert(param[0], 1.to_fixed());
    assert_err!(
        rikiddo.price_helper_second_quotient(&param, param[0], &formula_components),
        "[RikiddoSigmoidMV] Cannot find reduced exponential result of current element"
    );
}

#[test]
fn price_helper_second_quotient_overflow_elem_times_reduced_exp() {
    let rikiddo = Rikiddo::default();
    let param = vec![<FixedI128<U64>>::from_num(i64::MAX)];
    let formula_components = &mut <RikiddoFormulaComponents<FixedI128<U64>>>::default();
    formula_components.exponents.insert(param[0], 1.to_fixed());
    formula_components.reduced_exponential_results.insert(param[0], 2.to_fixed());
    assert_err!(
        rikiddo.price_helper_second_quotient(&param, param[0], &formula_components),
        "[RikiddoSigmoidMV] Overflow during calculation: element * reduced_exponential_result"
    );
}

#[test]
fn price_helper_second_quotient_overflow_sum_j_plus_elem_time_reduced_exp() {
    let rikiddo = Rikiddo::default();
    let param = vec![<FixedI128<U64>>::from_num(i64::MAX), <FixedI128<U64>>::from_num(1)];
    let formula_components = &mut <RikiddoFormulaComponents<FixedI128<U64>>>::default();
    formula_components.exponents.insert(param[0], 1.to_fixed());
    formula_components.reduced_exponential_results.insert(param[0], 1.to_fixed());
    formula_components.exponents.insert(param[0], 1.to_fixed());
    formula_components.reduced_exponential_results.insert(param[1], 1.to_fixed());
    assert_err!(
        rikiddo.price_helper_second_quotient(&param, param[0], &formula_components),
        "[RikiddoSigmoidMV] Overflow during calculation: sum_j += elem_times_reduced_exponential_result"
    );
}

#[test]
fn price_helper_second_quotient_overflow_sum_balances_times_sum_exp() {
    let rikiddo = Rikiddo::default();
    let param = vec![<FixedI128<U64>>::from_num(1)];
    let formula_components = &mut <RikiddoFormulaComponents<FixedI128<U64>>>::default();
    formula_components.exponents.insert(param[0], 1.to_fixed());
    formula_components.reduced_exponential_results.insert(param[0], 1.to_fixed());
    formula_components.sum_exp = 2.to_fixed();
    formula_components.sum_balances = <FixedI128<U64>>::from_num(i64::MAX);
    assert_err!(
        rikiddo.price_helper_second_quotient(&param, param[0], &formula_components),
        "[RikiddoSigmoidMV] Overflow during calculation: sum_balances * sum_exp"
    );
}

#[test]
fn price_helper_second_quotient_overflow_numerator_div_denominator() {
    //HERE
    let rikiddo = Rikiddo::default();
    let param = vec![<FixedI128<U64>>::from_num(1)];
    let formula_components = &mut <RikiddoFormulaComponents<FixedI128<U64>>>::default();
    formula_components.exponents.insert(param[0], 1.to_fixed());
    formula_components.reduced_exponential_results.insert(param[0], 1.to_fixed());
    formula_components.sum_balances = 1.to_fixed();
    // The following paramter will lead to a zero division error
    formula_components.sum_exp = 0.to_fixed();
    assert_err!(
        rikiddo.price_helper_second_quotient(&param, param[0], &formula_components),
        "[RikiddoSigmoidMV] Overflow during calculation (price helper 2): numerator / denominator"
    );
}

#[test]
fn price_helper_second_quotient_returns_correct_result() -> Result<(), &'static str> {
    Err("Unimplemented!")
}