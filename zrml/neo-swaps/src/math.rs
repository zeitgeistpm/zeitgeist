// Copyright 2023 Forecasting Technologies LTD.
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

use crate::{BalanceOf, Config, Error};
use alloc::vec::Vec;
use core::marker::PhantomData;
use fixed::FixedU128;
use hydra_dx_math::transcendental::{exp, ln};
use sp_runtime::{DispatchError, SaturatedConversion};
use typenum::U80;

type Fractional = U80;
type Fixed = FixedU128<Fractional>;

pub(crate) trait MathOps<T: Config> {
    fn calculate_swap_amount_out_for_buy(
        reserve: BalanceOf<T>,
        amount_in: BalanceOf<T>,
        liquidity: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError>;
    fn calculate_swap_amount_out_for_sell(
        reserve: BalanceOf<T>,
        amount_in: BalanceOf<T>,
        liquidity: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError>;
    fn calculate_spot_price(
        reserve: BalanceOf<T>,
        liquidity: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError>;
    fn calculate_reserves_from_spot_prices(
        amount: BalanceOf<T>,
        spot_prices: Vec<BalanceOf<T>>,
    ) -> Result<(BalanceOf<T>, Vec<BalanceOf<T>>), DispatchError>;
}

pub(crate) struct Math<T>(PhantomData<T>);

impl<T: Config> MathOps<T> for Math<T> {
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
}

mod detail {
    use super::*;
    use zeitgeist_primitives::{
        constants::DECIMALS,
        math::fixed::{IntoFixedDecimal, IntoFixedFromDecimal},
    };

    /// Calculate b * ln( e^(x/b) − 1 + e^(−r_i/b) ) + r_i − x
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

    /// Calculate –1 * b * ln( e^(-x/b) − 1 + e^(r_i/b) ) + r_i
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

    fn to_fixed<B>(value: B) -> Option<Fixed>
    where
        B: Into<u128> + From<u128>,
    {
        value.to_fixed_from_fixed_decimal(DECIMALS).ok()
    }

    fn from_fixed<B>(value: Fixed) -> Option<B>
    where
        B: Into<u128> + From<u128>,
    {
        value.to_fixed_decimal(DECIMALS).ok()
    }

    fn calculate_swap_amount_out_for_buy_fixed(
        reserve: Fixed,
        amount_in: Fixed,
        liquidity: Fixed,
    ) -> Option<Fixed> {
        // FIXME Defensive programming: Check for underflow in x/b and r_i/b.
        let exp_x_over_b: Fixed = exp(amount_in.checked_div(liquidity)?, false).ok()?;
        let exp_neg_r_over_b = exp(reserve.checked_div(liquidity)?, true).ok()?;
        // FIXME Defensive programming: Check for underflow in the exponential expressions.
        let inside_ln =
            exp_x_over_b.checked_add(exp_neg_r_over_b)?.checked_sub(Fixed::checked_from_num(1)?)?;
        let (ln_result, ln_neg) = ln(inside_ln).ok()?;
        let blob = liquidity.checked_mul(ln_result)?;
        let reserve_plus_blob =
            if ln_neg { reserve.checked_sub(blob)? } else { reserve.checked_add(blob)? };
        reserve_plus_blob.checked_sub(amount_in)
    }

    fn calculate_swap_amount_out_for_sell_fixed(
        reserve: Fixed,
        amount_in: Fixed,
        liquidity: Fixed,
    ) -> Option<Fixed> {
        // FIXME Defensive programming: Check for underflow in x/b and r_i/b.
        let exp_neg_x_over_b: Fixed = exp(amount_in.checked_div(liquidity)?, true).ok()?;
        let exp_r_over_b = exp(reserve.checked_div(liquidity)?, false).ok()?;
        // FIXME Defensive programming: Check for underflow in the exponential expressions.
        let inside_ln =
            exp_neg_x_over_b.checked_add(exp_r_over_b)?.checked_sub(Fixed::checked_from_num(1)?)?;
        let (ln_result, ln_neg) = ln(inside_ln).ok()?;
        let blob = liquidity.checked_mul(ln_result)?;
        if ln_neg { reserve.checked_add(blob) } else { reserve.checked_sub(blob) }
    }

    pub(crate) fn calculate_spot_price_fixed(reserve: Fixed, liquidity: Fixed) -> Option<Fixed> {
        exp(reserve.checked_div(liquidity)?, true).ok()
    }

    fn calculate_reserve_from_spot_prices_fixed(
        amount: Fixed,
        spot_prices: Vec<Fixed>,
    ) -> Option<(Fixed, Vec<Fixed>)> {
        // FIXME Defensive programming - ensure against underflows
        let tmp_reserves = spot_prices
            .iter()
            // Drop the bool (second tuple component) as ln(p) is always negative.
            .map(|&price| ln(price).map(|(value, _)| value))
            .collect::<Result<Vec<_>, _>>()
            .ok()?;
        let max_value = *tmp_reserves.iter().max()?;
        let liquidity = amount.checked_div(max_value)?;
        let reserves: Vec<Fixed> =
            tmp_reserves.iter().map(|&r| r.checked_mul(liquidity)).collect::<Option<Vec<_>>>()?;
        Some((liquidity, reserves))
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::{assert_approx, consts::*};
        use std::str::FromStr;
        use test_case::test_case;

        // Example taken from
        // https://docs.gnosis.io/conditionaltokens/docs/introduction3/#an-example-with-lmsr
        #[test]
        fn calculate_swap_amount_out_for_buy_works() {
            let liquidity = 144269504088;
            assert_eq!(
                calculate_swap_amount_out_for_buy(_10, _10, liquidity).unwrap(),
                58496250072
            );
        }

        #[test]
        fn calculate_swap_amount_out_for_sell_works() {
            let liquidity = 144269504088;
            assert_eq!(
                calculate_swap_amount_out_for_sell(_10, _10, liquidity).unwrap(),
                41503749928
            );
        }

        #[test]
        fn calcuate_spot_price_works() {
            let liquidity = 144269504088;
            assert_eq!(calculate_spot_price(_10, liquidity).unwrap(), _1_2);
            assert_eq!(calculate_spot_price(_10 - 58496250072, liquidity).unwrap(), _3_4);
            assert_eq!(calculate_spot_price(_20, liquidity).unwrap(), _1_4);
        }

        #[test]
        fn calculate_reserves_from_spot_prices_works() {
            let expected_liquidity = 144269504088;
            let (liquidity, reserves) =
                calculate_reserves_from_spot_prices(_10, vec![_1_2, _1_2]).unwrap();
            assert_approx!(liquidity, expected_liquidity, 1);
            assert_eq!(reserves, vec![_10, _10]);
        }

        // This test ensures that we don't mess anything up when we change precision.
        #[test_case(false, Fixed::from_str("10686474581524.462146990468650739308072").unwrap())]
        #[test_case(true, Fixed::from_str("0.000000000000093576229688").unwrap())]
        fn exp_does_not_overflow_or_underflow(neg: bool, expected: Fixed) {
            let value = 30;
            let result: Fixed = exp(Fixed::checked_from_num(value).unwrap(), neg).unwrap();
            assert_eq!(result, expected);
        }
    }
}
