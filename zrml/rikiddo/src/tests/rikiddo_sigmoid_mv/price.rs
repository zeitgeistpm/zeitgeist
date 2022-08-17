// Copyright 2021-2022 Zeitgeist PM LLC.
//
// This file is part of Zeitgeist.
//
// Zeitgeist is free software: you can redistribute it and/or modify it
// under the terms of the GNU General Public License as published by the
// Free Software Foundation, either version 3 of the License, or (at
// your option) any later version.
//
// Zeitgeist is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Zeitgeist. If not, see <https://www.gnu.org/licenses/>.

#![allow(clippy::needless_late_init)]
use frame_support::assert_err;
use substrate_fixed::{traits::ToFixed, types::extra::U64, FixedI128, FixedU128};

use super::{max_allowed_error, price_first_quotient, Rikiddo};
use crate::{
    tests::rikiddo_sigmoid_mv::{price, price_second_quotient},
    traits::Lmsr,
    types::RikiddoFormulaComponents,
};

fn check_if_exponent_in_formula_components(helper: u8) {
    let rikiddo = Rikiddo::default();
    let param = vec![<FixedI128<U64>>::from_num(100)];
    let result = {
        if helper == 1 {
            rikiddo.price_helper_first_quotient(
                &param,
                &param[0],
                &RikiddoFormulaComponents::default(),
            )
        } else {
            rikiddo.price_helper_second_quotient(&param, &RikiddoFormulaComponents::default())
        }
    };
    assert_err!(
        result,
        "[RikiddoSigmoidMV] Cannot find exponent of asset balance in question in \
         RikiddoFormulaComponents HashMap"
    );
}

fn check_price_helper_result(helper: u8) -> Result<(), &'static str> {
    let rikiddo = Rikiddo::default();
    let formula_components = &mut <RikiddoFormulaComponents<FixedI128<U64>>>::default();
    let param_f64 = vec![520.19, 480.81];
    let param =
        vec![<FixedI128<U64>>::from_num(param_f64[0]), <FixedI128<U64>>::from_num(param_f64[1])];
    let param_u =
        vec![<FixedU128<U64>>::from_num(param_f64[0]), <FixedU128<U64>>::from_num(param_f64[1])];
    // This fills the formula_components with the correct values
    let _ = rikiddo.cost_with_forumla(&param_u, formula_components, true, true);
    let rikiddo_price;
    let rikiddo_price_f64;
    let error_msg_function;

    if helper == 1 {
        rikiddo_price =
            rikiddo.price_helper_first_quotient(&param, &param[0], formula_components)?;
        rikiddo_price_f64 =
            price_first_quotient(rikiddo.config.initial_fee.to_num(), &param_f64, param_f64[0]);
        error_msg_function = "price_helper_first_quotient"
    } else {
        rikiddo_price = rikiddo.price_helper_second_quotient(&param, formula_components)?;
        rikiddo_price_f64 = price_second_quotient(rikiddo.config.initial_fee.to_num(), &param_f64);
        error_msg_function = "price_helper_second_quotient"
    }
    let difference_abs = (rikiddo_price.to_num::<f64>() - rikiddo_price_f64).abs();
    assert!(
        difference_abs <= max_allowed_error(63),
        "\n{} result (fixed): {}\n{} result (f64): {}\nDifference: {}\nMax_Allowed_Difference: {}",
        error_msg_function,
        rikiddo_price,
        error_msg_function,
        rikiddo_price_f64,
        difference_abs,
        max_allowed_error(64)
    );
    Ok(())
}

// --- Tests for price_helper_first_quotient ---

#[test]
fn rikiddo_price_helper_first_quotient_exponent_not_found() {
    check_if_exponent_in_formula_components(1)
}

#[test]
fn rikiddo_price_helper_first_quotient_overflow_exponent_sub_exp_qj() {
    let rikiddo = Rikiddo::default();
    let formula_components = &mut <RikiddoFormulaComponents<FixedI128<U64>>>::default();
    let param = vec![<FixedI128<U64>>::from(i64::MAX), <FixedI128<U64>>::from(-i64::MAX >> 5)];
    formula_components.exponents.insert(param[0], i64::MAX.to_fixed());
    formula_components.exponents.insert(param[1], <FixedI128<U64>>::from(-i64::MAX >> 5));
    formula_components.sum_times_fee = 0.1.to_fixed();
    assert_err!(
        rikiddo.price_helper_first_quotient(&param, &param[0], formula_components),
        "[RikiddoSigmoidMV] Overflow during calculation: exponent - exponent_balance_in_question"
    );
}

#[test]
fn rikiddo_price_helper_first_quotient_returns_correct_result() -> Result<(), &'static str> {
    check_price_helper_result(1)
}

// --- Tests for price_helper_second_quotient ---

#[test]
fn rikiddo_price_helper_second_quotient_exponent_not_found() {
    check_if_exponent_in_formula_components(2)
}

#[test]
fn rikiddo_price_helper_second_quotient_reduced_exp_not_found() {
    let rikiddo = Rikiddo::default();
    let param = vec![<FixedI128<U64>>::from_num(100)];
    let formula_components = &mut <RikiddoFormulaComponents<FixedI128<U64>>>::default();
    formula_components.exponents.insert(param[0], 1.to_fixed());
    assert_err!(
        rikiddo.price_helper_second_quotient(&param, formula_components),
        "[RikiddoSigmoidMV] Cannot find reduced exponential result of current element"
    );
}

#[test]
fn rikiddo_price_helper_second_quotient_overflow_elem_times_reduced_exp() {
    let rikiddo = Rikiddo::default();
    let param = vec![<FixedI128<U64>>::from_num(i64::MAX)];
    let formula_components = &mut <RikiddoFormulaComponents<FixedI128<U64>>>::default();
    formula_components.exponents.insert(param[0], 1.to_fixed());
    formula_components.reduced_exponential_results.insert(1.to_fixed(), 2.to_fixed());
    assert_err!(
        rikiddo.price_helper_second_quotient(&param, formula_components),
        "[RikiddoSigmoidMV] Overflow during calculation: element * reduced_exponential_result"
    );
}

#[test]
fn rikiddo_price_helper_second_quotient_overflow_sum_j_plus_elem_time_reduced_exp() {
    let rikiddo = Rikiddo::default();
    let param = vec![<FixedI128<U64>>::from_num(i64::MAX), <FixedI128<U64>>::from_num(1)];
    let formula_components = &mut <RikiddoFormulaComponents<FixedI128<U64>>>::default();
    formula_components.exponents.insert(param[0], 1.to_fixed());
    formula_components.exponents.insert(param[1], 1.to_fixed());
    formula_components.reduced_exponential_results.insert(1.to_fixed(), 1.to_fixed());
    assert_err!(
        rikiddo.price_helper_second_quotient(&param, formula_components),
        "[RikiddoSigmoidMV] Overflow during calculation: sum_j += \
         elem_times_reduced_exponential_result"
    );
}

#[test]
fn rikiddo_price_helper_second_quotient_overflow_sum_balances_times_sum_exp() {
    let rikiddo = Rikiddo::default();
    let param = vec![<FixedI128<U64>>::from_num(1)];
    let formula_components = &mut <RikiddoFormulaComponents<FixedI128<U64>>>::default();
    formula_components.exponents.insert(param[0], 1.to_fixed());
    formula_components.reduced_exponential_results.insert(param[0], 1.to_fixed());
    formula_components.sum_exp = 2.to_fixed();
    formula_components.sum_balances = <FixedI128<U64>>::from_num(i64::MAX);
    assert_err!(
        rikiddo.price_helper_second_quotient(&param, formula_components),
        "[RikiddoSigmoidMV] Overflow during calculation: sum_balances * sum_exp"
    );
}

#[test]
fn rikiddo_price_helper_second_quotient_overflow_numerator_div_denominator() {
    let rikiddo = Rikiddo::default();
    let param = vec![<FixedI128<U64>>::from_num(1)];
    let formula_components = &mut <RikiddoFormulaComponents<FixedI128<U64>>>::default();
    formula_components.exponents.insert(param[0], 1.to_fixed());
    formula_components.reduced_exponential_results.insert(param[0], 1.to_fixed());
    formula_components.sum_balances = 1.to_fixed();
    // The following parameter will lead to a zero division error
    formula_components.sum_exp = 0.to_fixed();
    assert_err!(
        rikiddo.price_helper_second_quotient(&param, formula_components),
        "[RikiddoSigmoidMV] Overflow during calculation (price helper 2): numerator / denominator"
    );
}

#[test]
fn rikiddo_price_helper_second_quotient_returns_correct_result() -> Result<(), &'static str> {
    check_price_helper_result(2)
}

// --- Tests for price ---
#[test]
fn rikiddo_price_asset_in_question_not_contained_in_balance_array() {
    let rikiddo = Rikiddo::default();
    let param = vec![<FixedU128<U64>>::from_num(1)];
    assert_err!(
        rikiddo.price(&param, &2.to_fixed()),
        "[RikiddoSigmoidMV] asset_in_question_balance not found in asset_balances"
    )
}

#[test]
fn rikiddo_price_returns_correct_result() -> Result<(), &'static str> {
    let rikiddo = Rikiddo::default();
    let param_f64 = vec![520.19, 500.00, 480.81];
    let param: Vec<FixedU128<U64>> =
        vec![param_f64[0].to_fixed(), param_f64[1].to_fixed(), param_f64[2].to_fixed()];
    let rikiddo_price = rikiddo.price(&param, &param[1])?;
    let rikiddo_price_f64 = price(rikiddo.config.initial_fee.to_num(), &param_f64, param_f64[1]);
    let difference_abs = (rikiddo_price.to_num::<f64>() - rikiddo_price_f64).abs();
    assert!(
        difference_abs <= max_allowed_error(34),
        "\nRikiddo price result (fixed): {}\nRikiddo price result (f64): {}\nDifference: \
         {}\nMax_Allowed_Difference: {}",
        rikiddo_price,
        rikiddo_price_f64,
        difference_abs,
        max_allowed_error(64)
    );
    Ok(())
}

#[test]
fn rikiddo_all_prices_returns_correct_result() -> Result<(), &'static str> {
    let rikiddo = Rikiddo::default();
    let param_f64 = vec![520.19, 500.00, 480.81];
    let param: Vec<FixedU128<U64>> =
        vec![param_f64[0].to_fixed(), param_f64[1].to_fixed(), param_f64[2].to_fixed()];
    let rikiddo_prices = rikiddo.all_prices(&param)?;
    let rikiddo_prices_f64 = vec![
        price(rikiddo.config.initial_fee.to_num(), &param_f64, param_f64[0]),
        price(rikiddo.config.initial_fee.to_num(), &param_f64, param_f64[1]),
        price(rikiddo.config.initial_fee.to_num(), &param_f64, param_f64[2]),
    ];
    for (idx, price) in rikiddo_prices.iter().enumerate() {
        let difference_abs = (price.to_num::<f64>() - rikiddo_prices_f64[idx]).abs();
        assert!(
            difference_abs <= max_allowed_error(34),
            "\nRikiddo price result (fixed): {}\nRikiddo price result (f64): {}\nDifference: \
             {}\nMax_Allowed_Difference: {}",
            price,
            rikiddo_prices_f64[idx],
            difference_abs,
            max_allowed_error(64)
        );
    }

    Ok(())
}
