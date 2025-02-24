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
    math::{traits::ComboMathOps, transcendental::ln, types::common::FixedType},
    BalanceOf, Config, Error,
};
use alloc::vec::Vec;
use core::marker::PhantomData;
use sp_runtime::{traits::Zero, DispatchError, SaturatedConversion};

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

    fn calculate_equalize_amount(
        buy: Vec<BalanceOf<T>>,
        sell: Vec<BalanceOf<T>>,
        amount_buy: BalanceOf<T>,
        amount_sell: BalanceOf<T>,
        liquidity: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        detail::calculate_equalize_amount(
            buy.into_iter().map(|x| x.saturated_into()).collect(),
            sell.into_iter().map(|x| x.saturated_into()).collect(),
            amount_buy.saturated_into(),
            amount_sell.saturated_into(),
            liquidity.saturated_into(),
        )
        .map(|result| result.saturated_into())
        .ok_or_else(|| Error::<T>::MathError.into())
    }

    fn calculate_swap_amount_out_for_sell(
        buy: Vec<BalanceOf<T>>,
        keep: Vec<BalanceOf<T>>,
        sell: Vec<BalanceOf<T>>,
        amount_buy: BalanceOf<T>,
        amount_keep: BalanceOf<T>,
        liquidity: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        detail::calculate_swap_amount_out_for_sell(
            buy.into_iter().map(|x| x.saturated_into()).collect(),
            keep.into_iter().map(|x| x.saturated_into()).collect(),
            sell.into_iter().map(|x| x.saturated_into()).collect(),
            amount_buy.saturated_into(),
            amount_keep.saturated_into(),
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
    use crate::math::types::common::{from_fixed, protected_exp, to_fixed};

    /// Converts `Vec<u128>` of fixed decimal numbers to a `Vec<FixedType>` of fixed point numbers;
    /// returns `None` if any of them fail.
    fn vec_to_fixed(vec: Vec<u128>) -> Option<Vec<FixedType>> {
        vec.into_iter().map(to_fixed).collect()
    }

    /// Returns `\sum_{r \in R} e^{-r/b}`, where `R` denotes `reserves` and `b` denotes `liquidity`.
    /// The result is `None` if and only if any of the `exp` calculations has failed.
    fn exp_sum(reserves: Vec<FixedType>, liquidity: FixedType) -> Option<FixedType> {
        reserves
            .iter()
            .map(|r| protected_exp(r.checked_div(liquidity)?, true))
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

    pub(super) fn calculate_equalize_amount(
        buy: Vec<u128>,
        sell: Vec<u128>,
        amount_buy: u128,
        amount_sell: u128,
        liquidity: u128,
    ) -> Option<u128> {
        let result_fixed = calculate_equalize_amount_fixed(
            vec_to_fixed(buy)?,
            vec_to_fixed(sell)?,
            to_fixed(amount_buy)?,
            to_fixed(amount_sell)?,
            to_fixed(liquidity)?,
        )?;
        from_fixed(result_fixed)
    }

    pub(super) fn calculate_swap_amount_out_for_sell(
        buy: Vec<u128>,
        keep: Vec<u128>,
        sell: Vec<u128>,
        amount_buy: u128,
        amount_keep: u128,
        liquidity: u128,
    ) -> Option<u128> {
        let result_fixed = calculate_swap_amount_out_for_sell_fixed(
            vec_to_fixed(buy)?,
            vec_to_fixed(keep)?,
            vec_to_fixed(sell)?,
            to_fixed(amount_buy)?,
            to_fixed(amount_keep)?,
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
        if buy.is_empty() || sell.is_empty() || amount_in.is_zero() {
            return None;
        }

        let exp_sum_buy = exp_sum(buy, liquidity)?;
        let exp_sum_sell = exp_sum(sell, liquidity)?;
        let amount_in_div_liquidity = amount_in.checked_div(liquidity)?;
        let exp_of_minus_amount_in: FixedType = protected_exp(amount_in_div_liquidity, true)?;
        let exp_of_minus_amount_in_times_exp_sum_sell =
            exp_of_minus_amount_in.checked_mul(exp_sum_sell)?;
        // Reminder from the documentation: `exp_sum_buy + exp_sum_sell = 1 - exp_sum_keep`
        let numerator = exp_sum_buy
            .checked_add(exp_sum_sell)?
            .checked_sub(exp_of_minus_amount_in_times_exp_sum_sell)?;
        let ln_arg = numerator.checked_div(exp_sum_buy)?;
        let (ln_val, _): (FixedType, _) = ln(ln_arg).ok()?;
        ln_val.checked_mul(liquidity)
    }

    fn calculate_equalize_amount_fixed(
        buy: Vec<FixedType>,
        sell: Vec<FixedType>,
        amount_buy: FixedType,
        amount_sell: FixedType,
        liquidity: FixedType,
    ) -> Option<FixedType> {
        if buy.is_empty() || sell.is_empty() || amount_buy.is_zero() {
            return None;
        }

        let exp_sum_buy = exp_sum(buy, liquidity)?;
        let exp_sum_sell = exp_sum(sell, liquidity)?;
        let numerator = exp_sum_buy.checked_add(exp_sum_sell)?;
        let delta = amount_buy.checked_sub(amount_sell)?;
        let delta_div_liquidity = delta.checked_div(liquidity)?;
        let exp_delta: FixedType = protected_exp(delta_div_liquidity, false)?;
        let exp_delta_times_exp_sum_sell = exp_delta.checked_mul(exp_sum_sell)?;
        let denominator = exp_sum_buy.checked_add(exp_delta_times_exp_sum_sell)?;
        let ln_arg = numerator.checked_div(denominator)?;
        let (ln_val, _): (FixedType, _) = ln(ln_arg).ok()?;
        ln_val.checked_mul(liquidity)
    }

    fn calculate_swap_amount_out_for_sell_fixed(
        buy: Vec<FixedType>,
        keep: Vec<FixedType>,
        sell: Vec<FixedType>,
        amount_buy: FixedType,
        amount_keep: FixedType,
        liquidity: FixedType,
    ) -> Option<FixedType> {
        // Ensure that either `keep` is empty and `amount_keep` is zero, or `keep` is non-empty and
        // `amount_keep` is non-zero.
        if (keep.is_empty() && !amount_keep.is_zero())
            || (!keep.is_empty() && amount_keep.is_zero())
        {
            return None;
        }

        // Reserves change after the first equalization. Since we do two equalization calculations
        // in one, we need to determine the intermediate reserves for the second calculation.
        let (amount_buy_keep, buy_keep) = if keep.is_empty() {
            (amount_buy, buy)
        } else {
            let delta_buy = calculate_equalize_amount_fixed(
                buy.clone(),
                keep.clone(),
                amount_buy,
                amount_keep,
                liquidity,
            )?;

            let delta_keep = amount_buy.checked_sub(delta_buy)?.checked_sub(amount_keep)?;

            let buy_intermediate =
                buy.into_iter().map(|x| x.checked_add(delta_buy)).collect::<Option<Vec<_>>>()?;
            let keep_intermediate =
                keep.into_iter().map(|x| x.checked_sub(delta_keep)).collect::<Option<Vec<_>>>()?;
            let buy_keep = buy_intermediate.into_iter().chain(keep_intermediate).collect();

            (amount_buy.checked_sub(delta_buy)?, buy_keep)
        };

        let delta_buy_keep = calculate_equalize_amount_fixed(
            buy_keep,
            sell,
            amount_buy_keep,
            FixedType::zero(),
            liquidity,
        )?;

        amount_buy_keep.checked_sub(delta_buy_keep)
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
    use crate::mock::Runtime as MockRuntime;
    use frame_support::assert_err;
    use test_case::test_case;
    use zeitgeist_primitives::constants::base_multiples::*;

    type MockBalance = BalanceOf<MockRuntime>;
    type MockMath = ComboMath<MockRuntime>;

    // Example taken from
    // https://github.com/gnosis/conditional-tokens-docs/blob/e73aa18ab82446049bca61df31fc88efd3cdc5cc/docs/intro3.md?plain=1#L78-L88
    #[test_case(vec![_10], vec![_10], _10, 144_269_504_088, 58_496_250_072)]
    #[test_case(vec![_1], vec![4_586_751_453], _1, _1, 7_353_256_641)]
    #[test_case(vec![_2], vec![9_173_502_907], _2, _2, 14_706_513_281; "positive ln")]
    #[test_case(vec![_1], vec![37_819_608_145], _1_10, _3, 386_589_943; "negative ln")]
    // Tests generated with Python.
    #[test_case(vec![_100, _100], vec![_100], _10, 721_347_520_444, 45_240_236_913)]
    #[test_case(vec![_100, _100, _100], vec![_100], _10, 721_347_520_444, 30_473_182_882)]
    #[test_case(vec![_100, _100], vec![_100, _100], _10, 721_347_520_444, 87_809_842_736)]
    #[test_case(vec![_100], vec![_100, _100, _100], _10, 721_347_520_444, 236_684_778_998)]
    #[test_case(
        vec![848_358_525_162, 482_990_395_533],
        vec![730_736_259_258],
        _10,
        527_114_788_714,
        36_648_762_089
    )]
    #[test_case(
        vec![848_358_525_162, _100, 482_990_395_533],
        vec![730_736_259_258],
        _10,
        527_114_788_714,
        29_520_025_573
    )]
    #[test_case(
        vec![848_358_525_162, 482_990_395_533, _100],
        vec![730_736_259_258],
        _10,
        527_114_788_714,
        29_520_025_573
    )]
    #[test_case(
        vec![848_358_525_162, 482_990_395_533],
        vec![730_736_259_258, _100],
        _10,
        527_114_788_714,
        57_474_148_073
    )]
    #[test_case(
        vec![482_990_395_533],
        vec![730_736_259_258, _100, 848_358_525_162],
        _10,
        527_114_788_714,
        121_489_297_813
    )]
    #[test_case(
        vec![848_358_525_162, 482_990_395_533],
        vec![730_736_259_258, _100],
        1_00,
        527_114_788_714,
        67
    )]
    #[test_case(
        vec![848_358_525_162, 482_990_395_533],
        vec![730_736_259_258, _100],
        1,
        527_114_788_714,
        1
    )]
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
    #[test_case(vec![], vec![_1], _1, _1)] // empty vector
    #[test_case(vec![_1], vec![], _1, _1)] // empty vector
    #[test_case(vec![_1], vec![_1], 0, _1)] // zero value
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

    // "Reversing" the tests for `calculate_swap_amount_for_buy`.
    #[test_case(vec![_11], vec![_12], _10, _10, 144_269_504_088, 0)]
    #[test_case(
        vec![_10 - 58_496_250_072],
        vec![_20],
        _10 + 58_496_250_072,
        0,
        144_269_504_088,
        58_496_250_072
    )]
    #[test_case(
        vec![_1 - 7_353_256_641],
        vec![14_586_751_453],
        17_353_256_641,
        0,
        _1,
        7_353_256_641
    )]
    #[test_case(
        vec![_2 - 14_706_513_281],
        vec![_2 + 9_173_502_907],
        _2 + 14_706_513_281,
        0,
        _2,
        14_706_513_281;
        "positive ln"
    )]
    #[test_case(
        vec![_1 - 386_589_943],
        vec![37_819_608_145 + _1_10],
        _1_10 + 386_589_943,
        0,
        _3,
        386_589_943;
        "negative ln"
    )]
    // Tests generated with Python
    #[test_case(
        vec![537_243_573_680, 305_865_360_520],
        vec![462_756_426_319],
        76_500_000_000,
        43_200_000_000,
        333_808_200_695,
        10_143_603_301
    )]
    #[test_case(
        vec![537_243_573_680, 305_865_360_520, 768_621_786_840],
        vec![462_756_426_319],
        232_000_000_000,
        112_300_000_000,
        333_808_200_695,
        35_887_802_365
    )]
    #[test_case(
        vec![537_243_573_680, 305_865_360_520],
        vec![462_756_426_319, _100],
        _10,
        _5,
        333_808_200_695,
        17_512_119_761
    )]
    #[test_case(
        vec![537_243_573_680, 305_865_360_520],
        vec![_100, 462_756_426_319],
        _10,
        _5,
        333_808_200_695,
        17_512_119_761
    )]
    #[test_case(
        vec![305_865_360_520, 537_243_573_680],
        vec![462_756_426_319, _100],
        _10,
        _5,
        333_808_200_695,
        17_512_119_761
    )]
    #[test_case(
        vec![305_865_360_520, 537_243_573_680],
        vec![_100, 462_756_426_319],
        _10,
        _5,
        333_808_200_695,
        17_512_119_761
    )]
    #[test_case(
        vec![305_865_360_520, 537_243_573_680],
        vec![_100, 462_756_426_319],
        _10,
        100,
        333_808_200_695,
        36_763_618_626
    )]
    #[test_case(
        vec![305_865_360_520, 537_243_573_680],
        vec![_100, 462_756_426_319],
        _10,
        1,
        333_808_200_695,
        36_763_618_666
    )]
    #[test_case(
        vec![305_865_360_520, 537_243_573_680],
        vec![_100, 462_756_426_319],
        2,
        1,
        333_808_200_695,
        0
    )]
    #[test_case(
        vec![305_865_360_520, 537_243_573_680],
        vec![_100, 462_756_426_319],
        1,
        0,
        333_808_200_695,
        0
    )]
    fn calculate_equalize_amount_works(
        buy: Vec<MockBalance>,
        sell: Vec<MockBalance>,
        amount_buy: MockBalance,
        amount_sell: MockBalance,
        liquidity: MockBalance,
        expected: MockBalance,
    ) {
        assert_eq!(
            MockMath::calculate_equalize_amount(buy, sell, amount_buy, amount_sell, liquidity)
                .unwrap(),
            expected
        );
    }

    #[test_case(vec![_1], vec![_1], _1, _1, 0)] // Division by zero
    #[test_case(vec![_1], vec![_1], 1_000 * _1, _1, _1)] // Overflow
    #[test_case(vec![_1], vec![_1], _1, 1_000 * _1, _1)] // Overflow
    #[test_case(vec![u128::MAX], vec![_1], _1, _1, _1)] // to_fixed error
    #[test_case(vec![_1], vec![u128::MAX], _1, _1, _1)] // to_fixed error
    #[test_case(vec![_1], vec![_1], u128::MAX, _1, _1)] // to_fixed error
    #[test_case(vec![_1], vec![_1], _1, u128::MAX, _1)] // to_fixed error
    #[test_case(vec![_1], vec![_1], _1, _1, u128::MAX)] // to_fixed error
    #[test_case(vec![], vec![_1], _1, _1, _1)] // empty vector
    #[test_case(vec![_1], vec![], _1, _1, _1)] // empty vector
    #[test_case(vec![_1], vec![_1], 0, _1, _1)] // zero value
    fn calculate_equalize_amount_throws_error(
        buy: Vec<MockBalance>,
        sell: Vec<MockBalance>,
        amount_buy: MockBalance,
        amount_sell: MockBalance,
        liquidity: MockBalance,
    ) {
        assert_err!(
            MockMath::calculate_equalize_amount(buy, sell, amount_buy, amount_sell, liquidity),
            Error::<MockRuntime>::MathError
        );
    }

    // Tests for `calculate_equalize`.
    #[test_case(
        vec![_10 - 58_496_250_072],
        vec![],
        vec![_20],
        _10 + 58_496_250_072,
        0,
        144_269_504_088,
        _10
    )]
    #[test_case(
        vec![_1 - 7_353_256_641],
        vec![],
        vec![14_586_751_453],
        17_353_256_641,
        0,
        _1,
        _1
    )]
    #[test_case(
        vec![_2 - 14_706_513_281],
        vec![],
        vec![_2 + 9_173_502_907],
        _2 + 14_706_513_281,
        0,
        _2,
        _2;
        "positive ln"
    )]
    #[test_case(
        vec![_1 - 386_589_943],
        vec![],
        vec![37_819_608_145 + _1_10],
        _1_10 + 386_589_943,
        0,
        _3,
        _1_10;
        "negative ln"
    )]
    // Tests generated by Python.
    #[test_case(
        vec![_100, 305_865_360_520],
        vec![768_621_786_840, _100, 768_621_786_840, _100],
        vec![462_756_426_319],
        76_500_000_000,
        43_200_000_000,
        333_808_200_695,
        45_943_057_520
    )]
    #[test_case(
        vec![_100, 305_865_360_520],
        vec![768_621_786_840, _100, 768_621_786_840, _100],
        vec![462_756_426_319],
        _2,
        _1,
        333_808_200_695,
        11_900_842_524
    )]
    #[test_case(
        vec![_100, 305_865_360_520, 768_621_786_840],
        vec![_100, 768_621_786_840],
        vec![462_756_426_319, _100],
        123_400_000_000,
        _1,
        333_808_200_695,
        63_972_215_306
    )]
    #[test_case(
        vec![_100, 305_865_360_520, 768_621_786_840],
        vec![_100],
        vec![462_756_426_319, _100, 768_621_786_840],
        123_400_000_000,
        1,
        333_808_200_695,
        62_187_083_257
    )]
    #[test_case(
        vec![_100, 305_865_360_520, 768_621_786_840],
        vec![_100],
        vec![462_756_426_319, _100, 768_621_786_840],
        2,
        1,
        333_808_200_695,
        1
    )]
    #[test_case(
        vec![_100, 305_865_360_520, 768_621_786_840],
        vec![],
        vec![462_756_426_319, _100, 768_621_786_840, _100],
        123_400_000_000,
        0,
        333_808_200_695,
        62_187_083_257
    )]
    fn calculate_swap_amount_out_for_sell_works(
        buy: Vec<MockBalance>,
        keep: Vec<MockBalance>,
        sell: Vec<MockBalance>,
        amount_buy: MockBalance,
        amount_sell: MockBalance,
        liquidity: MockBalance,
        expected: MockBalance,
    ) {
        assert_eq!(
            MockMath::calculate_swap_amount_out_for_sell(
                buy,
                keep,
                sell,
                amount_buy,
                amount_sell,
                liquidity
            )
            .unwrap(),
            expected
        );
    }

    #[test_case(vec![_1], vec![_1], vec![_1], _1, _1, 0)] // Division by zero
    #[test_case(vec![_1], vec![_1], vec![_1], 1_000 * _1, _1, _1)] // Overflow
    #[test_case(vec![_1], vec![_1], vec![_1], _1, 1_000 * _1, _1)] // Overflow
    #[test_case(vec![u128::MAX], vec![_1], vec![_1], _1, _1, _1)] // to_fixed error
    #[test_case(vec![_1], vec![u128::MAX], vec![_1], _1, _1, _1)] // to_fixed error
    #[test_case(vec![_1], vec![_1], vec![u128::MAX], u128::MAX, _1, _1)] // to_fixed error
    #[test_case(vec![_1], vec![_1], vec![_1], _1, u128::MAX, _1)] // to_fixed error
    #[test_case(vec![_1], vec![_1], vec![_1], _1, _1, u128::MAX)] // to_fixed error
    #[test_case(vec![], vec![_1], vec![_1], _1, _1, _1)] // empty vector
    #[test_case(vec![_1], vec![_1], vec![], _1, _1, _1)] // empty vector
    #[test_case(vec![_1], vec![], vec![_1], _1, _1, _1)] // empty vector
    #[test_case(vec![_1], vec![_1], vec![_1], 0, _1, _1)] // zero value
    #[test_case(vec![_1], vec![_1], vec![_1], _1, 0, _1)] // zero value
    fn calculate_swap_amount_out_for_sell_throws_error(
        buy: Vec<MockBalance>,
        keep: Vec<MockBalance>,
        sell: Vec<MockBalance>,
        amount_buy: MockBalance,
        amount_keep: MockBalance,
        liquidity: MockBalance,
    ) {
        assert_err!(
            MockMath::calculate_swap_amount_out_for_sell(
                buy,
                keep,
                sell,
                amount_buy,
                amount_keep,
                liquidity
            ),
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
