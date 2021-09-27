use substrate_fixed::{types::extra::U64, FixedI128, FixedU128};

use super::{ema_market_volume::ema_create_test_struct, max_allowed_error};
use crate::types::{EmaMarketVolume, FeeSigmoid, RikiddoSigmoidMV};

mod cost;
mod fee;
mod market_volume;
mod misc;
mod price;

type Rikiddo = RikiddoSigmoidMV<
    FixedU128<U64>,
    FixedI128<U64>,
    FeeSigmoid<FixedI128<U64>>,
    EmaMarketVolume<FixedU128<U64>>,
>;

pub(super) fn initial_outstanding_assets(num_assets: u32, subsidy: f64, minimum_fee: f64) -> f64 {
    let num_assets_f64: f64 = num_assets.into();
    let fee_times_num = minimum_fee * num_assets_f64;
    subsidy / (fee_times_num * num_assets_f64.ln() + 1f64)
}

fn ln_exp_sum(exponents: &Vec<f64>) -> f64 {
    exponents.iter().fold(0f64, |acc, val| acc + val.exp()).ln()
}

pub(super) fn cost(fee: f64, balances: &Vec<f64>) -> f64 {
    let fee_times_sum = fee * balances.iter().sum::<f64>();
    let exponents = balances.iter().map(|e| e / fee_times_sum).collect();
    fee_times_sum * ln_exp_sum(&exponents)
}

fn price_first_quotient(fee: f64, balances: &Vec<f64>, balance_in_question: f64) -> f64 {
    let balance_sum = balances.iter().sum::<f64>();
    let fee_times_sum = fee * balance_sum;
    let balance_exponential_results: Vec<f64> =
        balances.iter().map(|qj| (qj / fee_times_sum).exp()).collect();
    let denominator: f64 = balance_exponential_results.iter().sum::<f64>() * balance_sum;
    ((balance_in_question / fee_times_sum).exp() * balance_sum) / denominator
}

fn price_second_quotient(fee: f64, balances: &Vec<f64>) -> f64 {
    let balance_sum = balances.iter().sum::<f64>();
    let fee_times_sum = fee * balance_sum;
    let balance_exponential_results: Vec<f64> =
        balances.iter().map(|qj| (qj / fee_times_sum).exp()).collect();
    let denominator: f64 = balance_exponential_results.iter().sum::<f64>() * balance_sum;
    balance_exponential_results
        .iter()
        .enumerate()
        .map(|(idx, val)| balances[idx] * val)
        .sum::<f64>()
        / denominator
}

pub(super) fn price(fee: f64, balances: &Vec<f64>, balance_in_question: f64) -> f64 {
    let balance_sum = balances.iter().sum::<f64>();
    let fee_times_sum = fee * balance_sum;
    let balance_exponential_results: Vec<f64> =
        balances.iter().map(|qj| (qj / fee_times_sum).exp()).collect();
    let left_from_addition = cost(fee, balances) / balance_sum;
    let numerator_left_from_minus = (balance_in_question / fee_times_sum).exp() * balance_sum;
    let numerator_right_from_minus: f64 =
        balance_exponential_results.iter().enumerate().map(|(idx, val)| balances[idx] * val).sum();
    let numerator = numerator_left_from_minus - numerator_right_from_minus;
    let denominator: f64 = balance_exponential_results.iter().sum::<f64>() * balance_sum;
    left_from_addition + (numerator / denominator)
}
