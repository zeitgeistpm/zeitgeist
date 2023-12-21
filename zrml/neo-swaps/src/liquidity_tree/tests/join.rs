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
fn join_in_place_works_root() {
    let mut tree = utility::create_test_tree();
    tree.nodes[0].lazy_fees = _36;
    let mut nodes = tree.nodes.clone().into_inner();
    let account_to_index = tree.account_to_index.clone().into_inner();
    let abandoned_nodes = tree.abandoned_nodes.clone().into_inner();
    let amount = _2;
    nodes[0].stake += amount;
    // Distribute lazy fees of node at index 0.
    nodes[0].fees += 15_000_000_000; // 1.5
    nodes[0].lazy_fees = Zero::zero();
    nodes[1].lazy_fees += 300_000_000_000; // 30
    nodes[2].lazy_fees += 45_000_000_000; // 4.5
    tree.join(&3, amount).unwrap();
    assert_liquidity_tree_state!(tree, nodes, account_to_index, abandoned_nodes);
}

#[test]
fn join_in_place_works_leaf() {
    let mut tree = utility::create_test_tree();
    let mut nodes = tree.nodes.clone().into_inner();
    let account_to_index = tree.account_to_index.clone().into_inner();
    let abandoned_nodes = tree.abandoned_nodes.clone().into_inner();
    let amount = _2;
    nodes[0].descendant_stake += amount;
    nodes[1].descendant_stake += amount;
    nodes[3].descendant_stake += amount;
    nodes[7].stake += amount;
    // Distribute lazy fees of node at index 1 and 3.
    nodes[1].lazy_fees = Zero::zero();
    nodes[3].fees += 12_000_000_000; // 1.2
    nodes[3].lazy_fees = Zero::zero();
    nodes[4].lazy_fees += _1;
    nodes[7].fees += 78_000_000_000; // 7.8 (4.8 propagated and 3 lazy fees in place)
    nodes[7].lazy_fees = Zero::zero();
    tree.join(&6, amount).unwrap();
    assert_liquidity_tree_state!(tree, nodes, account_to_index, abandoned_nodes);
}

#[test]
fn join_in_place_works_middle() {
    let mut tree = utility::create_test_tree();
    let mut nodes = tree.nodes.clone().into_inner();
    let account_to_index = tree.account_to_index.clone().into_inner();
    let abandoned_nodes = tree.abandoned_nodes.clone().into_inner();
    let amount = _2;
    nodes[0].descendant_stake += amount;
    nodes[1].descendant_stake += amount;
    nodes[3].stake += amount;
    // Distribute lazy fees of node at index 1 and 3.
    nodes[1].lazy_fees = Zero::zero();
    nodes[3].fees += 12_000_000_000; // 1.2
    nodes[3].lazy_fees = 0;
    nodes[4].lazy_fees += _1;
    nodes[7].lazy_fees += 48_000_000_000; // 4.8
    tree.join(&5, amount).unwrap();
    assert_liquidity_tree_state!(tree, nodes, account_to_index, abandoned_nodes);
}

#[test]
fn join_reassigned_works_middle() {
    let mut tree = utility::create_test_tree();
    // Manipulate which node is joined by changing the order of abandoned nodes.
    tree.abandoned_nodes[0] = 8;
    tree.abandoned_nodes[3] = 1;
    let mut nodes = tree.nodes.clone().into_inner();
    let account = 99;
    let amount = _2;

    // Add new account.
    nodes[0].descendant_stake += amount;
    nodes[1].account = Some(account);
    nodes[1].stake = amount;
    nodes[1].lazy_fees = Zero::zero();
    // Propagate fees of node at index 1.
    nodes[3].lazy_fees += _3;
    nodes[4].lazy_fees += _1;
    let mut account_to_index = tree.account_to_index.clone().into_inner();
    account_to_index.insert(account, 1);
    let mut abandoned_nodes = tree.abandoned_nodes.clone().into_inner();
    abandoned_nodes.pop();

    tree.join(&account, amount).unwrap();
    assert_liquidity_tree_state!(tree, nodes, account_to_index, abandoned_nodes);
}

#[test]
fn join_reassigned_works_root() {
    let mut tree = utility::create_test_tree();
    // Store original test tree.
    let mut nodes = tree.nodes.clone().into_inner();
    // Manipulate test tree so that it looks like root was abandoned.
    tree.nodes[0].account = None;
    tree.nodes[0].stake = Zero::zero();
    tree.nodes[0].fees = Zero::zero();
    tree.nodes[0].lazy_fees = 345_000_000_000; // 34.5
    tree.abandoned_nodes.try_push(0).unwrap();
    tree.account_to_index.remove(&3);

    // Prepare expected data. The only things that have changed are that the 34.5 units of
    // collateral are propagated to the nodes of depth 1; and the root.
    let account = 99;
    let amount = _3;
    nodes[0].account = Some(account);
    nodes[0].stake = amount;
    nodes[0].fees = Zero::zero();
    nodes[0].lazy_fees = Zero::zero();
    nodes[1].lazy_fees += _30;
    nodes[2].lazy_fees += 45_000_000_000; // 4.5
    let mut account_to_index = tree.account_to_index.clone().into_inner();
    account_to_index.insert(account, 0);
    let mut abandoned_nodes = tree.abandoned_nodes.clone().into_inner();
    abandoned_nodes.pop();

    tree.join(&account, amount).unwrap();
    assert_liquidity_tree_state!(tree, nodes, account_to_index, abandoned_nodes);
}

#[test]
fn join_reassigned_works_leaf() {
    let mut tree = utility::create_test_tree();
    let mut nodes = tree.nodes.clone().into_inner();
    let account = 99;
    let amount = _3;
    nodes[0].descendant_stake += amount;
    nodes[1].descendant_stake += amount;
    nodes[3].descendant_stake += amount;
    nodes[8].account = Some(account);
    nodes[8].stake = amount;
    // Distribute lazy fees of node at index 1, 3 and 7 (same as join_reassigned_works_middle).
    nodes[1].lazy_fees = Zero::zero();
    nodes[3].fees += 12_000_000_000; // 1.2
    nodes[3].lazy_fees = 0;
    nodes[4].lazy_fees += _1;
    nodes[7].lazy_fees += 48_000_000_000; // 4.8

    let mut account_to_index = tree.account_to_index.clone().into_inner();
    account_to_index.insert(account, 8);
    let mut abandoned_nodes = tree.abandoned_nodes.clone().into_inner();
    abandoned_nodes.pop();

    tree.join(&account, amount).unwrap();
    assert_liquidity_tree_state!(tree, nodes, account_to_index, abandoned_nodes);
}

#[test]
fn join_in_place_works_if_tree_is_full() {
    let mut tree = utility::create_full_tree();
    // Remove one node.
    tree.nodes[0].descendant_stake -= tree.nodes[2].stake;
    tree.nodes[2].account = None;
    tree.nodes[2].stake = Zero::zero();
    tree.account_to_index.remove(&2);
    tree.abandoned_nodes.try_push(2).unwrap();
    let mut nodes = tree.nodes.clone().into_inner();
    let account = 99;
    let stake = 2;
    nodes[2].account = Some(account);
    nodes[2].stake = stake;
    nodes[0].descendant_stake += stake;
    let mut account_to_index = tree.account_to_index.clone().into_inner();
    let mut abandoned_nodes = tree.abandoned_nodes.clone().into_inner();
    account_to_index.insert(account, 2);
    abandoned_nodes.pop();
    tree.join(&account, stake).unwrap();
    assert_liquidity_tree_state!(tree, nodes, account_to_index, abandoned_nodes);
}

#[test]
fn join_new_fails_if_tree_is_full() {
    let mut tree = utility::create_full_tree();
    assert_err!(
        tree.join(&99, _1),
        LiquidityTreeError::TreeIsFull.into_dispatch_error::<Runtime>()
    );
}
