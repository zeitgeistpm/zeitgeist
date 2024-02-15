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

use super::*;

#[test]
fn withdraw_fees_works_root() {
    let mut tree = utility::create_test_tree();
    tree.nodes[0].lazy_fees = _36;
    let mut nodes = tree.nodes.clone().into_inner();
    let account_to_index = tree.account_to_index.clone().into_inner();
    let abandoned_nodes = tree.abandoned_nodes.clone().into_inner();

    // Distribute lazy fees of node at index 0.
    nodes[0].fees = Zero::zero();
    nodes[0].lazy_fees = Zero::zero();
    nodes[1].lazy_fees += 300_000_000_000; // 30
    nodes[2].lazy_fees += 45_000_000_000; // 4.5

    assert_eq!(tree.withdraw_fees(&3).unwrap(), 35_000_000_000); // 2 (fees) + 1.5 (lazy)
    assert_liquidity_tree_state!(tree, nodes, account_to_index, abandoned_nodes);
}

#[test]
fn withdraw_fees_works_middle() {
    let mut tree = utility::create_test_tree();
    let mut nodes = tree.nodes.clone().into_inner();
    let account_to_index = tree.account_to_index.clone().into_inner();
    let abandoned_nodes = tree.abandoned_nodes.clone().into_inner();

    // Distribute lazy fees of node at index 1, 3 and 7 (same as join_reassigned_works_middle).
    nodes[1].lazy_fees = Zero::zero();
    nodes[3].fees = Zero::zero();
    nodes[3].lazy_fees = Zero::zero();
    nodes[4].lazy_fees += _1;
    nodes[7].lazy_fees += 48_000_000_000; // 4.8

    assert_eq!(tree.withdraw_fees(&5).unwrap(), 22_000_000_000); // 1 (fees) + 1.2 (lazy)
    assert_liquidity_tree_state!(tree, nodes, account_to_index, abandoned_nodes);
}

#[test]
fn withdraw_fees_works_leaf() {
    let mut tree = utility::create_test_tree();
    let mut nodes = tree.nodes.clone().into_inner();
    let account_to_index = tree.account_to_index.clone().into_inner();
    let abandoned_nodes = tree.abandoned_nodes.clone().into_inner();

    // Distribute lazy fees of node at index 1, 3 and 7 (same as join_reassigned_works_middle).
    nodes[1].lazy_fees = Zero::zero();
    nodes[3].fees += 12_000_000_000; // 1.2
    nodes[3].lazy_fees = Zero::zero();
    nodes[4].lazy_fees += _1;
    nodes[7].fees = Zero::zero();
    nodes[7].lazy_fees = Zero::zero();

    assert_eq!(tree.withdraw_fees(&6).unwrap(), 88_000_000_000); // 1 (fees) + 7.8 (lazy)
    assert_liquidity_tree_state!(tree, nodes, account_to_index, abandoned_nodes);
}
