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

#![cfg(all(feature = "mock", test))]

/// Creates an `alloc::collections::BTreeMap` from the pattern `{ key => value, ... }`.
///
/// ```rust
/// // Example:
/// let m = create_b_tree_map!({ 0 => 1, 2 => 3 });
/// assert_eq!(m[2], 3);
///
/// // Overwriting a key:
/// let m = create_b_tree_map!({ 0 => "foo", 0 => "bar" });
/// assert_eq!(m[0], "bar");
/// ```
#[macro_export]
macro_rules! create_b_tree_map {
    ({ $($key:expr => $value:expr),* $(,)? } $(,)?) => {
        [$(($key, $value),)*].iter().cloned().collect::<alloc::collections::BTreeMap<_, _>>()
    }
}

/// Asserts that a market's LMSR liquidity pool has the specified state.
///
/// In addition to verifying the specified state, the macro also ensures that the pool's trading
/// function is (approximately) equal to `1`.
///
/// Parameters:
///
/// - `market_id`: The ID of the market that the pool belongs to.
/// - `reserves`: The expected reserves of the pool.
/// - `spot_prices`: The expected spot prices of outcomes in the pool.
/// - `liquidity_parameter`: The expected liquidity parameter of the pool.
/// - `liquidity_shares`: An `alloc::collections::BTreeMap` which maps each liquidity provider to
///   their expected stake.
/// - `total_fees`: The sum of all fees (both lazy and distributed) in the pool's liquidity tree.
#[macro_export]
#[cfg(test)]
macro_rules! assert_pool_state {
    (
        $market_id:expr,
        $reserves:expr,
        $spot_prices:expr,
        $liquidity_parameter:expr,
        $liquidity_shares:expr,
        $total_fees:expr
        $(,)?
    ) => {
        let pool = Pools::<Runtime>::get($market_id).unwrap();
        assert_eq!(
            pool.reserves.values().cloned().collect::<Vec<_>>(),
            $reserves,
            "assert_pool_state: Reserves mismatch"
        );
        let actual_spot_prices = pool
            .assets()
            .iter()
            .map(|&a| pool.calculate_spot_price(a).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(actual_spot_prices, $spot_prices, "assert_pool_state: Spot price mismatch");
        assert_eq!(
            pool.liquidity_parameter, $liquidity_parameter,
            "assert_pool_state: Liquidity parameter mismatch"
        );
        let actual_liquidity_shares = pool
            .liquidity_shares_manager
            .account_to_index
            .keys()
            .map(|&account| {
                (
                    account,
                    pool.liquidity_shares_manager.shares_of(&account).expect(
                        format!("assert_pool_state: No shares found for {:?}", account).as_str(),
                    ),
                )
            })
            .collect::<alloc::collections::BTreeMap<_, _>>();
        assert_eq!(
            actual_liquidity_shares, $liquidity_shares,
            "assert_pool_state: Liquidity shares mismatch"
        );
        let actual_total_fees = pool
            .liquidity_shares_manager
            .nodes
            .iter()
            .fold(0u128, |acc, node| acc + node.fees + node.lazy_fees);
        assert_eq!(actual_total_fees, $total_fees);
        let invariant = actual_spot_prices.iter().sum::<u128>();
        assert_approx!(invariant, _1, 1);
    };
}

/// Asserts that `account` has the specified `balances` of `assets`.
#[macro_export]
#[cfg(test)]
macro_rules! assert_balances {
    ($account:expr, $assets:expr, $balances:expr $(,)?) => {
        assert_eq!(
            $assets.len(),
            $balances.len(),
            "assert_balances: Assets and balances length mismatch"
        );
        for (&asset, &expected_balance) in $assets.iter().zip($balances.iter()) {
            let actual_balance = AssetManager::free_balance(asset.try_into().unwrap(), &$account);
            assert_eq!(
                actual_balance, expected_balance,
                "assert_balances: Balance mismatch for asset {:?}",
                asset,
            );
        }
    };
}

/// Asserts that `account` has the specified `balance` of `asset`.
#[macro_export]
#[cfg(test)]
macro_rules! assert_balance {
    ($account:expr, $asset:expr, $balance:expr $(,)?) => {
        assert_balances!($account, [$asset], [$balance]);
    };
}

/// Asserts that `abs(left - right) < precision`.
#[macro_export]
#[cfg(test)]
macro_rules! assert_approx {
    ($left:expr, $right:expr, $precision:expr $(,)?) => {
        match (&$left, &$right, &$precision) {
            (left_val, right_val, precision_val) => {
                let diff = if *left_val > *right_val {
                    *left_val - *right_val
                } else {
                    *right_val - *left_val
                };
                if diff > *precision_val {
                    panic!(
                        "assertion `left approx== right` failed\n      left: {}\n     right: {}\n \
                         precision: {}\ndifference: {}",
                        *left_val, *right_val, *precision_val, diff
                    );
                }
            }
        }
    };
}
