// Copyright 2023-2025 Forecasting Technologies LTD.
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

use crate::{
    math::{
        traits::MathOps,
        transcendental::ln,
        types::common::{FixedType, EXP_NUMERICAL_THRESHOLD},
    },
    BalanceOf, Config, Error,
};
use alloc::vec::Vec;
use core::marker::PhantomData;
use sp_runtime::{
    traits::{One, Zero},
    DispatchError, SaturatedConversion,
};

pub(crate) struct Math<T>(PhantomData<T>);

impl<T> MathOps<T> for Math<T>
where
    T: Config,
{
    fn calculate_swap_amount_out_for_buy(
        reserve: BalanceOf<T>,
        amount_in: BalanceOf<T>,
        liquidity: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        let reserve = reserve.saturated_into();
        let amount_in = amount_in.saturated_into();
        let liquidity = liquidity.saturated_into();
        detail::calculate_swap_amount_out_for_buy(reserve, amount_in, liquidity)
            .map(|result| result.saturated_into())
            .ok_or_else(|| Error::<T>::MathError.into())
    }

    fn calculate_swap_amount_out_for_sell(
        reserve: BalanceOf<T>,
        amount_in: BalanceOf<T>,
        liquidity: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        let reserve = reserve.saturated_into();
        let amount_in = amount_in.saturated_into();
        let liquidity = liquidity.saturated_into();
        detail::calculate_swap_amount_out_for_sell(reserve, amount_in, liquidity)
            .map(|result| result.saturated_into())
            .ok_or_else(|| Error::<T>::MathError.into())
    }

    fn calculate_spot_price(
        reserve: BalanceOf<T>,
        liquidity: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        let reserve = reserve.saturated_into();
        let liquidity = liquidity.saturated_into();
        detail::calculate_spot_price(reserve, liquidity)
            .map(|result| result.saturated_into())
            .ok_or_else(|| Error::<T>::MathError.into())
    }

    fn calculate_reserves_from_spot_prices(
        amount: BalanceOf<T>,
        spot_prices: Vec<BalanceOf<T>>,
    ) -> Result<(BalanceOf<T>, Vec<BalanceOf<T>>), DispatchError> {
        let amount = amount.saturated_into();
        let spot_prices = spot_prices.into_iter().map(|p| p.saturated_into()).collect();
        let (liquidity, spot_prices) =
            detail::calculate_reserves_from_spot_prices(amount, spot_prices)
                .ok_or_else(|| -> DispatchError { Error::<T>::MathError.into() })?;
        let liquidity = liquidity.saturated_into();
        let spot_prices = spot_prices.into_iter().map(|p| p.saturated_into()).collect();
        Ok((liquidity, spot_prices))
    }

    fn calculate_buy_ln_argument(
        reserve: BalanceOf<T>,
        amount_in: BalanceOf<T>,
        liquidity: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        let reserve = reserve.saturated_into();
        let amount_in = amount_in.saturated_into();
        let liquidity = liquidity.saturated_into();
        detail::calculate_buy_ln_argument(reserve, amount_in, liquidity)
            .map(|result| result.saturated_into())
            .ok_or_else(|| Error::<T>::MathError.into())
    }

    fn calculate_buy_amount_until(
        until: BalanceOf<T>,
        liquidity: BalanceOf<T>,
        spot_price: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        let until = until.saturated_into();
        let liquidity = liquidity.saturated_into();
        let spot_price = spot_price.saturated_into();
        detail::calculate_buy_amount_until(until, liquidity, spot_price)
            .map(|result| result.saturated_into())
            .ok_or_else(|| Error::<T>::MathError.into())
    }

    fn calculate_sell_amount_until(
        until: BalanceOf<T>,
        liquidity: BalanceOf<T>,
        spot_price: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        let until = until.saturated_into();
        let liquidity = liquidity.saturated_into();
        let spot_price = spot_price.saturated_into();
        detail::calculate_sell_amount_until(until, liquidity, spot_price)
            .map(|result| result.saturated_into())
            .ok_or_else(|| Error::<T>::MathError.into())
    }
}

mod detail {
    use super::*;
    use crate::math::types::common::{from_fixed, protected_exp, to_fixed};

    /// Calculate b * ln( e^(x/b) − 1 + e^(−r_i/b) ) + r_i − x.
    pub(super) fn calculate_swap_amount_out_for_buy(
        reserve: u128,
        amount_in: u128,
        liquidity: u128,
    ) -> Option<u128> {
        let result_fixed = calculate_swap_amount_out_for_buy_fixed(
            to_fixed(reserve)?,
            to_fixed(amount_in)?,
            to_fixed(liquidity)?,
        )?;
        from_fixed(result_fixed)
    }

    /// Calculate –1 * b * ln( e^(-x/b) − 1 + e^(r_i/b) ) + r_i.
    pub(super) fn calculate_swap_amount_out_for_sell(
        reserve: u128,
        amount_in: u128,
        liquidity: u128,
    ) -> Option<u128> {
        let result_fixed = calculate_swap_amount_out_for_sell_fixed(
            to_fixed(reserve)?,
            to_fixed(amount_in)?,
            to_fixed(liquidity)?,
        )?;
        from_fixed(result_fixed)
    }

    /// Calculate e^(-r_i/b).
    pub(super) fn calculate_spot_price(reserve: u128, liquidity: u128) -> Option<u128> {
        let result_fixed = calculate_spot_price_fixed(to_fixed(reserve)?, to_fixed(liquidity)?)?;
        from_fixed(result_fixed)
    }

    pub(super) fn calculate_reserves_from_spot_prices(
        amount: u128,
        spot_prices: Vec<u128>,
    ) -> Option<(u128, Vec<u128>)> {
        let (liquidity_fixed, reserve_fixed) = calculate_reserve_from_spot_prices_fixed(
            to_fixed(amount)?,
            spot_prices.into_iter().map(to_fixed).collect::<Option<Vec<_>>>()?,
        )?;
        let liquidity = from_fixed(liquidity_fixed)?;
        let reserve = reserve_fixed.into_iter().map(from_fixed).collect::<Option<Vec<_>>>()?;
        Some((liquidity, reserve))
    }

    /// Calculate e^(x/b) − 1 + e^(−r_i/b).
    pub(super) fn calculate_buy_ln_argument(
        reserve: u128,
        amount_in: u128,
        liquidity: u128,
    ) -> Option<u128> {
        let result_fixed = calculate_buy_ln_argument_fixed(
            to_fixed(reserve)?,
            to_fixed(amount_in)?,
            to_fixed(liquidity)?,
        )?;
        from_fixed(result_fixed)
    }

    pub(super) fn calculate_buy_amount_until(
        until: u128,
        liquidity: u128,
        spot_price: u128,
    ) -> Option<u128> {
        let result_fixed = calculate_buy_amount_until_fixed(
            to_fixed(until)?,
            to_fixed(liquidity)?,
            to_fixed(spot_price)?,
        )?;
        from_fixed(result_fixed)
    }

    pub(super) fn calculate_sell_amount_until(
        until: u128,
        liquidity: u128,
        spot_price: u128,
    ) -> Option<u128> {
        let result_fixed = calculate_sell_amount_until_fixed(
            to_fixed(until)?,
            to_fixed(liquidity)?,
            to_fixed(spot_price)?,
        )?;
        from_fixed(result_fixed)
    }

    fn calculate_swap_amount_out_for_buy_fixed(
        reserve: FixedType,
        amount_in: FixedType,
        liquidity: FixedType,
    ) -> Option<FixedType> {
        let inside_ln = calculate_buy_ln_argument_fixed(reserve, amount_in, liquidity)?;
        let (ln_result, ln_neg) = ln(inside_ln).ok()?;
        let blob = liquidity.checked_mul(ln_result)?;
        let reserve_plus_blob =
            if ln_neg { reserve.checked_sub(blob)? } else { reserve.checked_add(blob)? };
        reserve_plus_blob.checked_sub(amount_in)
    }

    fn calculate_swap_amount_out_for_sell_fixed(
        reserve: FixedType,
        amount_in: FixedType,
        liquidity: FixedType,
    ) -> Option<FixedType> {
        if reserve.is_zero() {
            // Ensure that if the reserve is zero, we don't accidentally return a non-zero value.
            return None;
        }
        let exp_neg_x_over_b: FixedType = protected_exp(amount_in.checked_div(liquidity)?, true)?;
        let exp_r_over_b = protected_exp(reserve.checked_div(liquidity)?, false)?;
        let inside_ln = exp_neg_x_over_b
            .checked_add(exp_r_over_b)?
            .checked_sub(FixedType::checked_from_num(1)?)?;
        let (ln_result, ln_neg) = ln(inside_ln).ok()?;
        let blob = liquidity.checked_mul(ln_result)?;
        if ln_neg { reserve.checked_add(blob) } else { reserve.checked_sub(blob) }
    }

    pub(crate) fn calculate_spot_price_fixed(
        reserve: FixedType,
        liquidity: FixedType,
    ) -> Option<FixedType> {
        protected_exp(reserve.checked_div(liquidity)?, true)
    }

    fn calculate_reserve_from_spot_prices_fixed(
        amount: FixedType,
        spot_prices: Vec<FixedType>,
    ) -> Option<(FixedType, Vec<FixedType>)> {
        if amount.is_zero() {
            // Ensure that if the amount is zero, we don't accidentally return meaningless results.
            return None;
        }
        let tmp_reserves = spot_prices
            .iter()
            // Drop the bool (second tuple component) as ln(p) is always negative.
            .map(|&price| ln(price).map(|(value, _)| value))
            .collect::<Result<Vec<_>, _>>()
            .ok()?;
        let max_value = *tmp_reserves.iter().max()?;
        let liquidity = amount.checked_div(max_value)?;
        let reserves: Vec<FixedType> =
            tmp_reserves.iter().map(|&r| r.checked_mul(liquidity)).collect::<Option<Vec<_>>>()?;
        Some((liquidity, reserves))
    }

    /// Calculate e^(x/b) − 1 + e^(−r_i/b).
    pub(super) fn calculate_buy_ln_argument_fixed(
        reserve: FixedType,
        amount_in: FixedType,
        liquidity: FixedType,
    ) -> Option<FixedType> {
        let exp_x_over_b: FixedType = protected_exp(amount_in.checked_div(liquidity)?, false)?;
        let r_over_b = reserve.checked_div(liquidity)?;
        let exp_neg_r_over_b = if r_over_b < EXP_NUMERICAL_THRESHOLD {
            protected_exp(r_over_b, true)?
        } else {
            FixedType::checked_from_num(0)? // Underflow to zero.
        };
        exp_x_over_b.checked_add(exp_neg_r_over_b)?.checked_sub(FixedType::checked_from_num(1)?)
    }

    /// Calculate `-b * ln( (1-q) / (1-p_i(r)) )` where `q = until` if `q > p_i(r)`; otherwise,
    /// return zero.
    pub(super) fn calculate_buy_amount_until_fixed(
        until: FixedType,
        liquidity: FixedType,
        spot_price: FixedType,
    ) -> Option<FixedType> {
        let numerator = FixedType::one().checked_sub(until)?;
        let denominator = FixedType::one().checked_sub(spot_price)?;
        let ln_arg = numerator.checked_div(denominator)?;
        let (ln_result, ln_neg) = ln(ln_arg).ok()?;
        if !ln_neg {
            return Some(FixedType::zero());
        }
        liquidity.checked_mul(ln_result)
    }

    /// Calculate `b * ln( (1 / (1 / p_i(r) - 1)) - (1 / q * (1 / p_i(r) - 1)) )` where `q = until`
    /// if `q < p_i(r)`; otherwise, return zero.
    pub(super) fn calculate_sell_amount_until_fixed(
        until: FixedType,
        liquidity: FixedType,
        spot_price: FixedType,
    ) -> Option<FixedType> {
        let first_numerator = FixedType::one();
        let first_denominator =
            (FixedType::one().checked_div(spot_price)?).checked_sub(FixedType::one())?;
        let second_numerator = FixedType::one();
        let second_denominator = until.checked_mul(
            FixedType::one().checked_div(spot_price)?.checked_sub(FixedType::one())?,
        )?;
        let first_term = first_numerator.checked_div(first_denominator)?;
        let second_term = second_numerator.checked_div(second_denominator)?;
        let ln_arg = second_term.checked_sub(first_term)?;
        let (ln_result, ln_neg) = ln(ln_arg).ok()?;
        if ln_neg {
            return Some(FixedType::zero());
        }
        liquidity.checked_mul(ln_result)
    }
}

#[cfg(test)]
mod tests {
    // TODO(#1328): Remove after rustc nightly-2024-04-22
    #![allow(clippy::duplicated_attributes)]

    use super::*;
    use crate::{
        math::transcendental::exp, mock::Runtime as MockRuntime, MAX_SPOT_PRICE, MIN_SPOT_PRICE,
    };
    use alloc::str::FromStr;
    use frame_support::assert_err;
    use test_case::test_case;
    use zeitgeist_primitives::constants::base_multiples::*;

    type MockBalance = BalanceOf<MockRuntime>;
    type MockMath = Math<MockRuntime>;

    // Example taken from
    // https://github.com/gnosis/conditional-tokens-docs/blob/e73aa18ab82446049bca61df31fc88efd3cdc5cc/docs/intro3.md?plain=1#L78-L88
    #[test_case(_10, _10, 144_269_504_088, 58_496_250_072)]
    #[test_case(_1, _1, _1, 7_353_256_641)]
    #[test_case(_2, _2, _2, 14_706_513_281; "positive ln")]
    #[test_case(_1, _1_10, _3, 386_589_943; "negative ln")]
    #[test_case(_100, _10, _3, 998_910_224_189; "underflow to zero, positive ln")]
    #[test_case(_100, _1_10, _3, 897_465_467_426; "underflow to zero, negative ln")]
    // Limit value tests; functions shouldn't be called with these values, but these tests
    // demonstrate they can be called without risk.
    #[test_case(0, _1, _1, 0)]
    #[test_case(_1, 0, _1, 0)]
    #[test_case(_30, _30, _1 - 100_000, _30)]
    #[test_case(_1_10, _30, _1 - 100_000, _1_10)]
    #[test_case(_30, _1_10, _1 - 100_000, 276_478_645_689)]
    #[test_case(10_000_000 * _1, _1, 144_269_504_088_896_352, 9_999_999_307)]
    #[test_case(
        100_000_000 * _1,
        100_000_000 * _1,
        434_294_481_903_251_840,
        959_041_392_321_093_596
    )]
    #[test_case(
        45_757_490_560_675_120,
        100_000_000 * _1,
        434_294_481_903_251_840,
        41_392_685_158_225_036
    )]
    fn calculate_swap_amount_out_for_buy_works(
        reserve: MockBalance,
        amount_in: MockBalance,
        liquidity: MockBalance,
        expected: MockBalance,
    ) {
        assert_eq!(
            MockMath::calculate_swap_amount_out_for_buy(reserve, amount_in, liquidity).unwrap(),
            expected
        );
    }

    #[test_case(_1, _1, 0)] // Division by zero
    #[test_case(_1, 1_000 * _1, _1)] // Overflow
    #[test_case(u128::MAX, _1, _1)] // to_fixed error
    #[test_case(_1, u128::MAX, _1)] // to_fixed error
    #[test_case(_1, _1, u128::MAX)] // to_fixed error
    fn calculate_swap_amount_out_for_buy_throws_math_error(
        reserve: MockBalance,
        amount_in: MockBalance,
        liquidity: MockBalance,
    ) {
        assert_err!(
            MockMath::calculate_swap_amount_out_for_buy(reserve, amount_in, liquidity),
            Error::<MockRuntime>::MathError
        );
    }

    #[test_case(_10, _10, 144_269_504_088, 41_503_749_928)]
    #[test_case(_1, _1, _1, 2_646_743_359)]
    #[test_case(_2, _2, _2, 5_293_486_719)]
    #[test_case(_17, _8, _7, 4_334_780_553; "positive ln")]
    #[test_case(_1, _11, 33_000_000_000, 41_104_447_891; "negative ln")]
    // Limit value tests; functions shouldn't be called with these values, but these tests
    // demonstrate they can be called without risk.
    #[test_case(_1, 0, _1, 0)]
    #[test_case(_30, _30, _1 - 100_000, 0)]
    #[test_case(_1_10, _30, _1 - 100_000, 23_521_354_311)]
    #[test_case(_30, _1_10, _1 - 100_000, 0)]
    #[test_case(10_000_000 * _1, _1, 144_269_504_088_896_352, 4_999_999_913)]
    #[test_case(
        100_000_000 * _1,
        100_000_000 * _1,
        434_294_481_903_251_840,
        40_958_607_678_906_404
    )]
    #[test_case(
        45_757_490_560_675_120,
        100_000_000 * _1,
        434_294_481_903_251_840,
        721_246_399_047_171_074
    )]
    fn calculate_swap_amount_out_for_sell_works(
        reserve: MockBalance,
        amount_in: MockBalance,
        liquidity: MockBalance,
        expected: MockBalance,
    ) {
        assert_eq!(
            MockMath::calculate_swap_amount_out_for_sell(reserve, amount_in, liquidity).unwrap(),
            expected
        );
    }

    #[test_case(0, _1, _1)]
    #[test_case(_1, _1, 0)] // Division by zero
    #[test_case(1000 * _1, _1, _1)] // Overflow
    #[test_case(u128::MAX, _1, _1)] // to_fixed error
    #[test_case(_1, u128::MAX, _1)] // to_fixed error
    #[test_case(_1, _1, u128::MAX)] // to_fixed error
    fn calculate_swap_amount_out_for_sell_throws_math_error(
        reserve: MockBalance,
        amount_in: MockBalance,
        liquidity: MockBalance,
    ) {
        assert_err!(
            MockMath::calculate_swap_amount_out_for_sell(reserve, amount_in, liquidity),
            Error::<MockRuntime>::MathError
        );
    }

    #[test_case(_10, 144_269_504_088, _1_2)]
    #[test_case(_10 - 58_496_250_072, 144_269_504_088, _3_4)]
    #[test_case(_20, 144_269_504_088, _1_4)]
    fn calcuate_spot_price_works(
        reserve: MockBalance,
        liquidity: MockBalance,
        expected: MockBalance,
    ) {
        assert_eq!(MockMath::calculate_spot_price(reserve, liquidity).unwrap(), expected);
    }

    #[test_case(_1, 0)] // Division by zero
    #[test_case(1_000 * _1, _1)] // Overflow
    #[test_case(u128::MAX, _1)] // to_fixed error
    #[test_case(_1, u128::MAX)] // to_fixed error
    fn calculate_spot_price_throws_math_error(reserve: MockBalance, liquidity: MockBalance) {
        assert_err!(
            MockMath::calculate_spot_price(reserve, liquidity),
            Error::<MockRuntime>::MathError
        );
    }

    #[test_case(_10, vec![_1_2, _1_2], vec![_10, _10], 144_269_504_089)]
    #[test_case(_20, vec![_3_4, _1_4], vec![_10 - 58_496_250_072, _20], 144_269_504_089)]
    #[test_case(
        _444,
        vec![_1_10, _2_10, _3_10, _4_10],
        vec![_444, 3_103_426_819_252, 2_321_581_629_045, 1_766_853_638_504],
        1_928_267_499_650
    )]
    #[test_case(
        _100,
        vec![50_000_000, 50_000_000, 50_000_000, 8_500_000_000],
        vec![_100, _100, _100, 30_673_687_183],
        188_739_165_818
    )]
    #[test_case(
        100_000_000 * _1,
        vec![_1_10, _9_10],
        vec![100_000_000 * _1, 45_757_490_560_675_125],
        434_294_481_903_251_828
    )]
    fn calculate_reserves_from_spot_prices_works(
        amount: MockBalance,
        spot_prices: Vec<MockBalance>,
        expected_reserves: Vec<MockBalance>,
        expected_liquidity: MockBalance,
    ) {
        let (liquidity, reserves) =
            MockMath::calculate_reserves_from_spot_prices(amount, spot_prices).unwrap();
        assert_eq!(liquidity, expected_liquidity);
        assert_eq!(reserves, expected_reserves);
    }

    #[test_case(0, vec![_1_10, _2_10, _3_10, _4_10])]
    #[test_case(u128::MAX, vec![_1_10, _2_10, _3_10, _4_10])] // to_fixed error
    #[test_case(_1, vec![u128::MAX, 0, 0])] // to_fixed error
    #[test_case(_1, vec![_1, 0])] // ln out of range
    fn calculate_reserves_from_spot_prices_throws_math_error(
        amount: MockBalance,
        spot_prices: Vec<MockBalance>,
    ) {
        assert_err!(
            MockMath::calculate_reserves_from_spot_prices(amount, spot_prices),
            Error::<MockRuntime>::MathError
        );
    }

    #[test_case(_1, _1, 0)] // Division by zero
    #[test_case(_1, 1_000 * _1, _1)] // Overflow
    #[test_case(u128::MAX, _1, _1)] // to_fixed error
    #[test_case(_1, u128::MAX, _1)] // to_fixed error
    #[test_case(_1, _1, u128::MAX)] // to_fixed error
    fn calculate_buy_ln_argument_fixed_works(
        reserve: MockBalance,
        amount_in: MockBalance,
        liquidity: MockBalance,
    ) {
        assert_err!(
            MockMath::calculate_buy_ln_argument(reserve, amount_in, liquidity),
            Error::<MockRuntime>::MathError
        );
    }

    // This test ensures that we don't mess anything up when we change precision.
    #[test_case(false, FixedType::from_str("123705850708694.521074740553659523785099").unwrap())]
    #[test_case(true, FixedType::from_str("0.000000000000008083692034").unwrap())]
    fn exp_does_not_overflow_or_underflow(neg: bool, expected: FixedType) {
        let result: FixedType =
            exp(FixedType::checked_from_num(EXP_NUMERICAL_THRESHOLD).unwrap(), neg).unwrap();
        assert_eq!(result, expected);
    }

    #[test_case(_9_10, _10, _1_10, 219722457734)] // Large price shift
    #[test_case(_4_10, _10, _3_10, 15415067983)] // Small price shift
    #[test_case(_3_10, _10, _4_10, 0)] // Zero buy amount
    #[test_case(_4_10, _10, _4_10, 0)] // Zero buy amount
    #[test_case(MAX_SPOT_PRICE, 188_739_165_817, MIN_SPOT_PRICE, 999_053_937_034; "leap_up")]
    #[test_case(MIN_SPOT_PRICE, _10, MAX_SPOT_PRICE, 0; "leap_down")]
    #[test_case(
        MIN_SPOT_PRICE + 100_000,
        132_117_416_072,
        MIN_SPOT_PRICE,
        1_327_820;
        "step_up_low"
    )]
    #[test_case(
        MAX_SPOT_PRICE,
        11_324_349_949,
        MAX_SPOT_PRICE - 100_000,
        22_626_081;
        "step_up_high"
    )]
    #[test_case(MIN_SPOT_PRICE, _1, MIN_SPOT_PRICE + 100_000, 0; "step_down_low")]
    #[test_case(MAX_SPOT_PRICE - 100_000, _1, MAX_SPOT_PRICE, 0; "step_down_high")]
    fn calculate_buy_amount_until_works(
        until: MockBalance,
        liquidity: MockBalance,
        spot_price: MockBalance,
        expected: MockBalance,
    ) {
        assert_eq!(
            MockMath::calculate_buy_amount_until(until, liquidity, spot_price).unwrap(),
            expected
        );
    }

    #[test_case(_1, _10, _1_2; "until too large")]
    #[test_case(_1_2, _10, _1; "spot price too large")]
    #[test_case(u128::MAX, _10, _1_2; "until limit")]
    #[test_case(_3_4, u128::MAX, _1_2; "liquidity limit")]
    #[test_case(_3_4, _10, u128::MAX; "spot price limit")]
    fn calculate_buy_amount_until_throws_math_error(
        until: MockBalance,
        liquidity: MockBalance,
        spot_price: MockBalance,
    ) {
        assert_err!(
            MockMath::calculate_buy_amount_until(until, liquidity, spot_price),
            Error::<MockRuntime>::MathError
        );
    }

    #[test_case(_1_10, _10, _9_10, 439444915467)] // Large price shift
    #[test_case(_1_10, _10, _2_10, 81093021622)] // Small price shift
    #[test_case(_2_10, _10, _1_10, 0)] // Zero sell amount
    #[test_case(_1_10, _10, _1_10, 0)] // Zero sell amount
    #[test_case(_1_100, _10, _1_2, 459511985013)] // Very small until
    #[test_case(1_2, _10, 1_100, 451815891780)] // Very small spot_price
    #[test_case(MAX_SPOT_PRICE, 188_739_165_817, MIN_SPOT_PRICE, 0; "leap_up")]
    #[test_case(MIN_SPOT_PRICE, 11_324_349_949, MAX_SPOT_PRICE, 119_886_472_444; "leap_down")]
    #[test_case(MIN_SPOT_PRICE + 100_000, 132_117_416_072, MIN_SPOT_PRICE, 0; "step_up_low")]
    #[test_case(MAX_SPOT_PRICE, 11_324_349_949, MAX_SPOT_PRICE - 100_000, 0; "step_up_high")]
    #[test_case(
        MIN_SPOT_PRICE,
        186_922_262_798,
        MIN_SPOT_PRICE + 100_000,
        375_349_804;
        "step_down_low"
    )]
    #[test_case(
        MAX_SPOT_PRICE - 100_000,
        43_410_008_138,
        MAX_SPOT_PRICE,
        87_169_596;
        "step_down_high"
    )]
    fn calculate_sell_amount_until_fixed_works(
        until: MockBalance,
        liquidity: MockBalance,
        spot_price: MockBalance,
        expected: MockBalance,
    ) {
        assert_eq!(
            MockMath::calculate_sell_amount_until(until, liquidity, spot_price).unwrap(),
            expected
        );
    }

    #[test_case(0, _10, _1_2; "until too small")]
    #[test_case(_1_2, _10, _1; "spot price too large")]
    #[test_case(u128::MAX, _10, _3_4; "until limit")]
    #[test_case(_1_2, u128::MAX, _3_4; "liquidity limit")]
    #[test_case(_1_2, _10, u128::MAX; "spot price limit")]
    fn calculate_sell_amount_until_throws_math_error(
        until: MockBalance,
        liquidity: MockBalance,
        spot_price: MockBalance,
    ) {
        assert_err!(
            MockMath::calculate_sell_amount_until(until, liquidity, spot_price),
            Error::<MockRuntime>::MathError
        );
    }
}
