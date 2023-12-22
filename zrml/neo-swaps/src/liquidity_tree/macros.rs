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

/// Asserts that a liquidity tree has the specified state.
///
/// Parameters:
///
/// - `tree`: The `LiquidityTree<T, U>` to check.
/// - `expected_nodes`: The expected `tree.nodes`.
/// - `expected_accounts_to_index`: The expected `tree.accounts_to_index`.
/// - `expected_abandoned_nodes`: The expected `tree.abandoned_nodes`.
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
