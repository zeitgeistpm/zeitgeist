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

#[cfg(test)]
#[macro_export]
macro_rules! create_b_tree_map {
    ({ $($key:expr => $value:expr),* $(,)? } $(,)?) => {
        [$(($key, $value),)*].iter().cloned().collect::<std::collections::BTreeMap<_, _>>()
    }
}

#[cfg(test)]
#[macro_export]
macro_rules! assert_liquidity_tree_state {
    (
        $tree:expr,
        $expected_nodes:expr,
        { $($key:expr => $value:expr),* $(,)? },
        $expected_abandoned_nodes:expr $(,)?
    ) => {
        let nodes = &$tree.nodes;
        assert_eq!(nodes.len(), $expected_nodes.len());
        for (index, expected) in $expected_nodes.iter().enumerate() {
            assert_eq!(nodes[index], expected.clone());
            // assert_eq!(nodes[index].account, expected.account);
            // assert_eq!(nodes[index].stake, expected.stake);
            // assert_eq!(nodes[index].fees, expected.fees);
            // assert_eq!(nodes[index].descendant_stake, expected.descendant_stake);
            // assert_eq!(nodes[index].lazy_fees, expected.lazy_fees);
        }
        let expected_account_to_index = create_b_tree_map!({ $($key => $value),* });
        assert_eq!(expected_account_to_index, $tree.account_to_index.clone().into_inner());
        assert_eq!($expected_abandoned_nodes, $tree.abandoned_nodes.clone().into_inner());
    };
}
