use frame_support::assert_err;
use substrate_fixed::{traits::ToFixed, types::extra::U64, FixedI128, FixedU128};

use super::{ema_market_volume::ema_create_test_struct, max_allowed_error};
use crate::{
    constants::INITIAL_FEE,
    traits::{Lmsr, MarketAverage, RikiddoMV},
    types::{
        convert_to_signed, EmaMarketVolume, FeeSigmoid, RikiddoConfig, RikiddoSigmoidMV,
        TimestampedVolume,
    },
};

type Rikiddo = RikiddoSigmoidMV<
    FixedU128<U64>,
    FixedI128<U64>,
    FeeSigmoid<FixedI128<U64>>,
    EmaMarketVolume<FixedU128<U64>>,
>;

fn ln_exp_sum(exponents: &Vec<f64>) -> f64 {
    exponents.iter().fold(0f64, |acc, val| acc + val.exp()).ln()
}

fn cost(fee: f64, balances: &Vec<f64>) -> f64 {
    let fee_times_sum = fee * balances.iter().sum::<f64>();
    let exponents = balances.iter().map(|e| e / fee_times_sum).collect();
    fee_times_sum * ln_exp_sum(&exponents)
}

#[test]
fn rikiddo_updates_mv_and_returns_some() {
    let emv = ema_create_test_struct(1, 2.0);
    let mut rikiddo =
        Rikiddo::new(RikiddoConfig::default(), FeeSigmoid::default(), emv.clone(), emv);
    rikiddo.update(&TimestampedVolume { timestamp: 0, volume: 1u32.into() }).unwrap();
    let res = rikiddo.update(&TimestampedVolume { timestamp: 2, volume: 2u32.into() }).unwrap();
    assert_eq!(res, Some(1u32.into()));
}

#[test]
fn rikiddo_updates_mv_and_returns_none() {
    let mut rikiddo = Rikiddo::default();
    let vol = TimestampedVolume::default();
    assert_eq!(rikiddo.update(&vol).unwrap(), None);
}

#[test]
fn rikiddo_clear_clears_market_data() {
    let mut rikiddo = Rikiddo::default();
    let rikiddo_clone = rikiddo.clone();
    let _ = rikiddo.update(&<TimestampedVolume<FixedU128<U64>>>::default());
    assert_ne!(rikiddo, rikiddo_clone);
    rikiddo.clear();
    assert_eq!(rikiddo, rikiddo_clone);
}

#[test]
fn rikiddo_get_fee_catches_zero_divison() {
    let emv = ema_create_test_struct(1, 0.0);
    let mut rikiddo =
        Rikiddo::new(RikiddoConfig::default(), FeeSigmoid::default(), emv.clone(), emv);
    let _ = rikiddo.update(&TimestampedVolume { timestamp: 0, volume: 0u32.into() });
    let _ = rikiddo.update(&TimestampedVolume { timestamp: 2, volume: 0u32.into() });
    assert_err!(
        rikiddo.get_fee(),
        "[RikiddoSigmoidMV] Zero division error during calculation: ma_short / ma_long"
    );
}

#[test]
fn rikiddo_get_fee_overflows_during_ratio_calculation() {
    let emv = ema_create_test_struct(1, 2.0);
    let mut rikiddo =
        Rikiddo::new(RikiddoConfig::default(), FeeSigmoid::default(), emv.clone(), emv);
    let _ = rikiddo.update(&TimestampedVolume { timestamp: 0, volume: 0u32.into() });
    let _ = rikiddo.update(&TimestampedVolume { timestamp: 2, volume: 0u32.into() });
    rikiddo.ma_short.ema = <FixedU128<U64>>::from_num(u64::MAX);
    rikiddo.ma_long.ema = <FixedU128<U64>>::from_num(0.1f64);
    assert_err!(
        rikiddo.get_fee(),
        "[RikiddoSigmoidMV] Overflow during calculation: ma_short / ma_long"
    );
}

#[test]
fn rikiddo_get_fee_ratio_does_not_fit_in_type() {
    let emv = ema_create_test_struct(1, 2.0);
    let mut rikiddo =
        Rikiddo::new(RikiddoConfig::default(), FeeSigmoid::default(), emv.clone(), emv);
    let _ = rikiddo.update(&TimestampedVolume { timestamp: 0, volume: 0u32.into() });
    let _ = rikiddo.update(&TimestampedVolume { timestamp: 2, volume: 0u32.into() });
    rikiddo.ma_short.ema = <FixedU128<U64>>::from_num(u64::MAX);
    rikiddo.ma_long.ema = <FixedU128<U64>>::from_num(1u64);
    assert_err!(
        rikiddo.get_fee(),
        "Fixed point conversion failed: FROM type does not fit in TO type"
    );
}

#[test]
fn rikiddo_get_fee_returns_the_correct_result() {
    let emv_short = ema_create_test_struct(1, 2.0);
    let emv_long = ema_create_test_struct(2, 2.0);
    let mut rikiddo =
        Rikiddo::new(RikiddoConfig::default(), FeeSigmoid::default(), emv_short, emv_long);
    assert_eq!(rikiddo.ma_short.get(), None);
    assert_eq!(rikiddo.ma_long.get(), None);
    assert_eq!(rikiddo.get_fee().unwrap(), rikiddo.config.initial_fee);
    let _ = rikiddo.update(&TimestampedVolume { timestamp: 0, volume: 100u32.into() });
    let _ = rikiddo.update(&TimestampedVolume { timestamp: 2, volume: 100u32.into() });
    assert_ne!(rikiddo.ma_short.get(), None);
    assert_eq!(rikiddo.ma_long.get(), None);
    assert_eq!(rikiddo.get_fee().unwrap(), rikiddo.config.initial_fee);
    let _ = rikiddo.update(&TimestampedVolume { timestamp: 3, volume: 100u32.into() });
    assert_ne!(rikiddo.ma_short.get(), None);
    assert_ne!(rikiddo.ma_long.get(), None);
    // We don't want to test the exact result (that is the responsibility of the fee module),
    // but rather if rikiddo toggles properly between initial fee and the calculated fee
    assert_ne!(rikiddo.get_fee().unwrap(), rikiddo.config.initial_fee);
}

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
    rikiddo.config.initial_fee = <FixedI128<U64>>::from_num(2);
    let param = <FixedU128<U64>>::from_num(u64::MAX);
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
    let param = <FixedU128<U64>>::from_num(u64::MAX);
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
    rikiddo.config.log2_e = <FixedI128<U64>>::from_num(i64::MAX);
    let param = <FixedU128<U64>>::from_num(i64::MAX as u64);
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
