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

// TODO Document all these macros

#[cfg(test)]
#[macro_export]
macro_rules! create_b_tree_map {
    ({ $($key:expr => $value:expr),* $(,)? } $(,)?) => {
        [$(($key, $value),)*].iter().cloned().collect::<alloc::collections::BTreeMap<_, _>>()
    }
}

#[cfg(test)]
#[macro_export]
macro_rules! assert_liquidity_tree_state {
    (
        $tree:expr,
        $expected_nodes:expr,
        $expected_account_to_index:expr,
        $expected_abandoned_nodes:expr
        $(,)?
    ) => {
        let actual_nodes = $tree.nodes.clone().into_inner();
        let max_len = std::cmp::max($expected_nodes.len(), actual_nodes.len());
        let mut error = false;
        for index in 0..max_len {
            match ($expected_nodes.get(index), actual_nodes.get(index)) {
                (Some(exp), Some(act)) => {
                    if exp != act {
                        error = true;
                        eprintln!(
                            "assert_liquidity_tree_state: Mismatched node at index {}",
                            index,
                        );
                        eprintln!("    Expected node: {:?}", exp);
                        eprintln!("    Actual node:   {:?}", act);
                    }
                }
                (None, Some(act)) => {
                    error = true;
                    eprintln!("assert_liquidity_tree_state: Extra node at index {}", index);
                    eprintln!("    {:?}", act);
                }
                (Some(exp), None) => {
                    error = true;
                    eprintln!("assert_liquidity_tree_state: Missing node at index {}", index);
                    eprintln!("    {:?}", exp);
                }
                (None, None) => break,
            }
        }
        if error {
            panic!();
        }
        assert_eq!($expected_account_to_index, $tree.account_to_index.clone().into_inner());
        assert_eq!($expected_abandoned_nodes, $tree.abandoned_nodes.clone().into_inner());
    };
}

#[cfg(test)]
#[macro_export]
macro_rules! assert_pool_status {
    (
        $market_id:expr,
        $reserves:expr,
        $spot_prices:expr,
        $liquidity_parameter:expr,
        $liquidity_shares:expr
        $(,)?
    ) => {
        let pool = Pools::<Runtime>::get($market_id).unwrap();
        assert_eq!(
            pool.reserves.values().cloned().collect::<Vec<_>>(),
            $reserves,
            "assert_pool_status: Reserves mismatch"
        );
        let actual_spot_prices = pool
            .assets()
            .iter()
            .map(|&a| pool.calculate_spot_price(a).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(actual_spot_prices, $spot_prices, "assert_pool_status: Spot price mismatch");
        let invariant = actual_spot_prices.iter().sum::<u128>();
        assert_eq!(
            pool.liquidity_parameter, $liquidity_parameter,
            "assert_pool_status: Liquidity parameter mismatch"
        );
        let actual_liquidity_shares = pool
            .liquidity_shares_manager
            .account_to_index
            .keys()
            .map(|&account| {
                (
                    account,
                    pool.liquidity_shares_manager.shares_of(&account).expect(
                        format!("assert_pool_status: No shares found for {:?}", account).as_str(),
                    ),
                )
            })
            .collect::<alloc::collections::BTreeMap<_, _>>();
        assert_eq!(
            actual_liquidity_shares, $liquidity_shares,
            "assert_pool_status: Liquidity shares mismatch"
        );
        assert_approx!(invariant, _1, 1);
    };
}

#[cfg(test)]
#[macro_export]
macro_rules! assert_balances {
    ($account:expr, $assets:expr, $balances:expr $(,)?) => {
        assert_eq!(
            $assets.len(),
            $balances.len(),
            "assert_balances: Assets and balances length mismatch"
        );
        for (&asset, &expected_balance) in $assets.iter().zip($balances.iter()) {
            let actual_balance = AssetManager::free_balance(asset, &$account);
            assert_eq!(
                actual_balance, expected_balance,
                "assert_balances: Balance mismatch for asset {:?}",
                asset,
            );
        }
    };
}
