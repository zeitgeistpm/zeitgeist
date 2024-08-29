// Copyright 2023-2024 Forecasting Technologies LTD.
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
        traits::ComboMathOps,
        transcendental::{exp, ln},
    },
    BalanceOf, Config, Error,
};
use alloc::vec::Vec;
use core::marker::PhantomData;
use fixed::FixedU128;
use sp_runtime::{
    traits::{One, Zero},
    DispatchError, SaturatedConversion,
};
use typenum::U80;

type Fractional = U80;
type FixedType = FixedU128<Fractional>;

/// The point at which 32.44892769177272
const EXP_OVERFLOW_THRESHOLD: FixedType = FixedType::from_bits(0x20_72EC_ECDA_6EBE_EACC_40C7);

pub(crate) struct ComboMath<T>(PhantomData<T>);

impl<T> ComboMathOps<T> for ComboMath<T>
where
    T: Config,
{
    fn calculate_swap_amount_out_for_buy(
        buy: Vec<BalanceOf<T>>,
        sell: Vec<BalanceOf<T>>,
        amount_in: BalanceOf<T>,
        liquidity: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        detail::calculate_swap_amount_out_for_buy(
            buy.into_iter().map(|x| x.saturated_into()).collect(),
            sell.into_iter().map(|x| x.saturated_into()).collect(),
            amount_in.saturated_into(),
            liquidity.saturated_into(),
        )
        .map(|result| result.saturated_into())
        .ok_or_else(|| Error::<T>::MathError.into())
    }

    fn calculate_swap_amount_out_for_sell(
        buy: Vec<BalanceOf<T>>,
        sell: Vec<BalanceOf<T>>,
        amount_in: BalanceOf<T>,
        liquidity: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        detail::calculate_swap_amount_out_for_sell(
            buy.into_iter().map(|x| x.saturated_into()).collect(),
            sell.into_iter().map(|x| x.saturated_into()).collect(),
            amount_in.saturated_into(),
            liquidity.saturated_into(),
        )
        .map(|result| result.saturated_into())
        .ok_or_else(|| Error::<T>::MathError.into())
    }

    fn calculate_spot_price(
        buy: Vec<BalanceOf<T>>,
        sell: Vec<BalanceOf<T>>,
        liquidity: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        detail::calculate_spot_price(
            buy.into_iter().map(|x| x.saturated_into()).collect(),
            sell.into_iter().map(|x| x.saturated_into()).collect(),
            liquidity.saturated_into(),
        )
        .map(|result| result.saturated_into())
        .ok_or_else(|| Error::<T>::MathError.into())
    }
}

mod detail {
    use super::*;
    use zeitgeist_primitives::{
        constants::DECIMALS,
        math::fixed::{IntoFixedDecimal, IntoFixedFromDecimal},
    };

    fn to_fixed(value: u128) -> Option<FixedType> {
        value.to_fixed_from_fixed_decimal(DECIMALS).ok()
    }

    /// Converts `Vec<u128>` of fixed decimal numbers to a `Vec<FixedType>` of fixed point numbers;
    /// returns `None` if any of them fail.
    fn vec_to_fixed(vec: Vec<u128>) -> Option<Vec<FixedType>> {
        vec.into_iter().map(to_fixed).collect()
    }

    fn from_fixed<B>(value: FixedType) -> Option<B>
    where
        B: Into<u128> + From<u128>,
    {
        value.to_fixed_decimal(DECIMALS).ok()
    }

    /// Returns `\sum_{r \in R} e^{-r/b}`, where `R` denotes `reserves` and `b` denotes `liquidity`.
    /// The result is `None` if and only if one of the `exp` calculations has failed.
    fn exp_sum(reserves: Vec<FixedType>, liquidity: FixedType) -> Option<FixedType> {
        reserves
            .iter()
            .map(|r| exp(r.checked_div(liquidity)?, true).ok())
            .collect::<Option<Vec<_>>>()?
            .iter()
            .try_fold(FixedType::zero(), |acc, &val| acc.checked_add(val))
    }

    pub(super) fn calculate_swap_amount_out_for_buy(
        buy: Vec<u128>,
        sell: Vec<u128>,
        amount_in: u128,
        liquidity: u128,
    ) -> Option<u128> {
        let result_fixed = calculate_swap_amount_out_for_buy_fixed(
            vec_to_fixed(buy)?,
            vec_to_fixed(sell)?,
            to_fixed(amount_in)?,
            to_fixed(liquidity)?,
        )?;
        from_fixed(result_fixed)
    }

    pub(super) fn calculate_swap_amount_out_for_sell(
        buy: Vec<u128>,
        sell: Vec<u128>,
        amount_in: u128,
        liquidity: u128,
    ) -> Option<u128> {
        let result_fixed = calculate_swap_amount_out_for_sell_fixed(
            vec_to_fixed(buy)?,
            vec_to_fixed(sell)?,
            to_fixed(amount_in)?,
            to_fixed(liquidity)?,
        )?;
        from_fixed(result_fixed)
    }

    pub(super) fn calculate_spot_price(
        buy: Vec<u128>,
        sell: Vec<u128>,
        liquidity: u128,
    ) -> Option<u128> {
        let result_fixed = calculate_spot_price_fixed(
            vec_to_fixed(buy)?,
            vec_to_fixed(sell)?,
            to_fixed(liquidity)?,
        )?;
        from_fixed(result_fixed)
    }

    fn calculate_swap_amount_out_for_buy_fixed(
        buy: Vec<FixedType>,
        sell: Vec<FixedType>,
        amount_in: FixedType,
        liquidity: FixedType,
    ) -> Option<FixedType> {
        let exp_sum_buy = exp_sum(buy, liquidity)?;
        let exp_sum_sell = exp_sum(sell, liquidity)?;
        let amount_in_div_liquidity = amount_in.checked_div(liquidity)?;
        let exp_of_minus_amount_in: FixedType = exp(amount_in_div_liquidity, true).ok()?;
        let exp_of_minus_amount_in_times_exp_sum_sell =
            exp_of_minus_amount_in.checked_mul(exp_sum_sell)?;
        let numerator = exp_sum_buy
            .checked_add(exp_sum_sell)?
            .checked_sub(exp_of_minus_amount_in_times_exp_sum_sell)?;
        let ln_arg = numerator.checked_div(exp_sum_buy)?;
        println!("{}", ln_arg);
        let (ln_val, _): (FixedType, _) = ln(ln_arg).ok()?;
        ln_val.checked_mul(liquidity)
    }

    fn calculate_swap_amount_out_for_sell_fixed(
        _buy: Vec<FixedType>,
        _sell: Vec<FixedType>,
        _amount_in: FixedType,
        _liquidity: FixedType,
    ) -> Option<FixedType> {
        None
    }

    fn calculate_spot_price_fixed(
        buy: Vec<FixedType>,
        sell: Vec<FixedType>,
        liquidity: FixedType,
    ) -> Option<FixedType> {
        let exp_sum_buy = exp_sum(buy, liquidity)?;
        let exp_sum_sell = exp_sum(sell, liquidity)?;
        let denominator = exp_sum_buy.checked_add(exp_sum_sell)?;
        exp_sum_buy.checked_div(denominator)
    }
}

#[cfg(test)]
mod tests {
    // TODO(#1328): Remove after rustc nightly-2024-04-22
    #![allow(clippy::duplicated_attributes)]

    use super::*;
    use crate::{mock::Runtime as MockRuntime, MAX_SPOT_PRICE, MIN_SPOT_PRICE};
    use alloc::str::FromStr;
    use frame_support::assert_err;
    use test_case::test_case;
    use zeitgeist_primitives::constants::base_multiples::*;

    type MockBalance = BalanceOf<MockRuntime>;
    type MockMath = ComboMath<MockRuntime>;

    // Example taken from
    // https://docs.gnosis.io/conditionaltokens/docs/introduction3/#an-example-with-lmsr
    #[test_case(vec![_10], vec![_10], _10, 144_269_504_088, 58_496_250_072)]
    #[test_case(vec![_1], vec![4_586_751_453], _1, _1, 7_353_256_641)]
    #[test_case(vec![_2], vec![9_173_502_907], _2, _2, 14_706_513_281; "positive ln")]
    #[test_case(vec![_1], vec![37_819_608_145], _1_10, _3, 386_589_943; "negative ln")]
    fn calculate_swap_amount_out_for_buy_works(
        buy: Vec<MockBalance>,
        sell: Vec<MockBalance>,
        amount_in: MockBalance,
        liquidity: MockBalance,
        expected: MockBalance,
    ) {
        assert_eq!(
            MockMath::calculate_swap_amount_out_for_buy(buy, sell, amount_in, liquidity).unwrap(),
            expected
        );
    }

    #[test_case(vec![_1], vec![_1], _1, 0)] // Division by zero
    #[test_case(vec![_1], vec![_1], 1_000 * _1, _1)] // Overflow
    #[test_case(vec![u128::MAX], vec![_1], _1, _1)] // to_fixed error
    #[test_case(vec![_1], vec![u128::MAX], _1, _1)] // to_fixed error
    #[test_case(vec![_1], vec![_1], u128::MAX, _1)] // to_fixed error
    #[test_case(vec![_1], vec![_1], _1, u128::MAX)] // to_fixed error
    fn calculate_swap_amount_out_for_buy_throws_math_error(
        buy: Vec<MockBalance>,
        sell: Vec<MockBalance>,
        amount_in: MockBalance,
        liquidity: MockBalance,
    ) {
        assert_err!(
            MockMath::calculate_swap_amount_out_for_buy(buy, sell, amount_in, liquidity),
            Error::<MockRuntime>::MathError
        );
    }

    #[test_case(vec![_10], vec![_10], 144_269_504_088, _1_2)]
    #[test_case(vec![_10 - 58_496_250_072], vec![_20], 144_269_504_088, _3_4)]
    #[test_case(vec![_20], vec![_10 - 58_496_250_072], 144_269_504_088, _1_4)]
    fn calcuate_spot_price_works(
        buy: Vec<MockBalance>,
        sell: Vec<MockBalance>,
        liquidity: MockBalance,
        expected: MockBalance,
    ) {
        assert_eq!(MockMath::calculate_spot_price(buy, sell, liquidity).unwrap(), expected);
    }

    #[test_case(vec![_1], vec![_1], 0)] // Division by zero
    #[test_case(vec![1_000 * _1], vec![_1], _1)] // Overflow
    #[test_case(vec![_1], vec![1_000 * _1], _1)] // Overflow
    #[test_case(vec![u128::MAX], vec![_1], _1)] // to_fixed error
    #[test_case(vec![_1], vec![u128::MAX], _1)] // to_fixed error
    #[test_case(vec![_1], vec![_1], u128::MAX)] // to_fixed error
    fn calculate_spot_price_throws_math_error(
        buy: Vec<MockBalance>,
        sell: Vec<MockBalance>,
        liquidity: MockBalance,
    ) {
        assert_err!(
            MockMath::calculate_spot_price(buy, sell, liquidity),
            Error::<MockRuntime>::MathError
        );
    }
}
