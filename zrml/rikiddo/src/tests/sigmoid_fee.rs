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

use super::max_allowed_error;
use crate::{
    traits::Fee,
    types::{FeeSigmoid, FeeSigmoidConfig},
};
use frame_support::assert_err;
use substrate_fixed::{types::extra::U64, FixedI128};

fn sigmoid_fee(m: f64, n: f64, p: f64, r: f64) -> f64 {
    (m * (r - n)) / (p + (r - n).powi(2)).sqrt()
}

fn init_default_sigmoid_fee_struct() -> (FeeSigmoid<FixedI128<U64>>, f64, f64, f64) {
    let m = 0.01f64;
    let n = 0f64;
    let p = 2.0f64;
    let initial_fee = 0.005;
    let min_revenue = 0.0035;

    let config = FeeSigmoidConfig {
        m: <FixedI128<U64>>::from_num(m),
        n: <FixedI128<U64>>::from_num(n),
        p: <FixedI128<U64>>::from_num(p),
        initial_fee: <FixedI128<U64>>::from_num(initial_fee),
        min_revenue: <FixedI128<U64>>::from_num(min_revenue),
    };

    let fee = FeeSigmoid { config };
    (fee, m, n, p)
}

#[test]
fn fee_sigmoid_overflow_r_minus_n() {
    let (mut fee, _, _, _) = init_default_sigmoid_fee_struct();
    let r = <FixedI128<U64>>::from_num(i64::MIN);
    fee.config.n = <FixedI128<U64>>::from_num(i64::MAX);
    assert_err!(fee.calculate_fee(r), "[FeeSigmoid] Overflow during calculation: r - n");
}

#[test]
fn fee_sigmoid_overflow_m_times_r_minus_n() {
    let (mut fee, _, _, _) = init_default_sigmoid_fee_struct();
    let r = <FixedI128<U64>>::from_num(i64::MIN);
    fee.config.n = <FixedI128<U64>>::from_num(0);
    fee.config.m = <FixedI128<U64>>::from_num(i64::MAX);
    assert_err!(fee.calculate_fee(r), "[FeeSigmoid] Overflow during calculation: m * (r-n)");
}

#[test]
fn fee_sigmoid_overflow_r_minus_n_squared() {
    let (mut fee, _, _, _) = init_default_sigmoid_fee_struct();
    let r = <FixedI128<U64>>::from_num(i64::MIN);
    fee.config.n = <FixedI128<U64>>::from_num(0);
    assert_err!(fee.calculate_fee(r), "[FeeSigmoid] Overflow during calculation: (r-n)^2");
}

#[test]
fn fee_sigmoid_overflow_p_plus_r_minus_n_squared() {
    let (mut fee, _, _, _) = init_default_sigmoid_fee_struct();
    let r = <FixedI128<U64>>::from_num(0);
    fee.config.n = <FixedI128<U64>>::from_num(1);
    fee.config.m = <FixedI128<U64>>::from_num(0);
    fee.config.p = <FixedI128<U64>>::from_num(i64::MAX);
    assert_err!(fee.calculate_fee(r), "[FeeSigmoid] Overflow during calculation: p + (r-n)^2");
}

#[test]
fn fee_sigmoid_overflow_numerator_div_denominator() {
    let (mut fee, _, _, _) = init_default_sigmoid_fee_struct();
    let r = <FixedI128<U64>>::from_num(0.1);
    fee.config.n = <FixedI128<U64>>::from_num(0);
    fee.config.m = <FixedI128<U64>>::from_num(i64::MAX);
    fee.config.p = <FixedI128<U64>>::from_num(-0.0099);
    assert_err!(
        fee.calculate_fee(r),
        "[FeeSigmoid] Overflow during calculation: numerator / denominator"
    );
}

#[test]
fn fee_sigmoid_correct_result() -> Result<(), &'static str> {
    let r = 1.5f64;
    let (mut fee, m, n, p) = init_default_sigmoid_fee_struct();
    let fee_f64 = fee.config.initial_fee.to_num::<f64>() + sigmoid_fee(m, n, p, r);
    let r_fixed = <FixedI128<U64>>::from_num(r);
    let fee_fixed = fee.calculate_fee(r_fixed)?;
    let fee_fixed_f64: f64 = fee_fixed.to_num();
    let difference_abs = (fee_f64 - fee_fixed_f64).abs();

    assert!(
        difference_abs <= max_allowed_error(64),
        "\nFixed result: {}\nFloat result: {}\nDifference: {}\nMax_Allowed_Difference: {}",
        fee_fixed_f64,
        fee_f64,
        difference_abs,
        max_allowed_error(64)
    );

    fee.config.min_revenue = <FixedI128<U64>>::from_num(1u64 << 62);
    assert_eq!(fee.calculate_fee(r_fixed)?, fee.config.min_revenue);
    Ok(())
}
