use frame_support::assert_err;
use substrate_fixed::{traits::ToFixed, types::extra::U64, FixedI128, FixedU128};

use super::{ema_create_test_struct, max_allowed_error, price_first_quotient, Rikiddo};
use crate::{
    constants::INITIAL_FEE,
    traits::{Lmsr, MarketAverage, RikiddoMV},
    types::{
        convert_to_signed, EmaMarketVolume, FeeSigmoid, RikiddoConfig, RikiddoFormulaComponents,
        RikiddoSigmoidMV, TimestampedVolume,
    },
};

#[test]
fn price_helper_first_quotient_exponent_not_found() -> Result<(), &'static str> {
    let rikiddo = Rikiddo::default();
    let param = vec![<FixedI128<U64>>::from_num(100u64)];
    assert_err!(
        rikiddo.price_helper_first_quotient(&param, param[0], &RikiddoFormulaComponents::default()),
        "[RikiddoSigmoidMV] Cannot find exponent of asset balance in question \
         RikiddoFormulaComponents HashMap"
    );
    Ok(())
}

#[test]
fn price_helper_first_quotient_overflow_exponent_sub_exp_qj() -> Result<(), &'static str> {
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
    Ok(())
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
