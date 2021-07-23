use substrate_fixed::{types::extra::U64, FixedI128, FixedU128};

use super::{ema_market_volume::ema_create_test_struct, max_allowed_error};
use crate::types::{EmaMarketVolume, FeeSigmoid, RikiddoSigmoidMV};

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

mod cost;
mod fee;
mod market_volume;
mod misc;
mod price;
