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
use crate::{AccountIdOf, BalanceOf};
use test_case::test_case;

#[test_case(false)]
#[test_case(true)]
fn exit_root_works(withdraw_all: bool) {
    let mut tree = utility::create_test_tree();
    // Remove lazy fees on the path to the node (and actual fees from the node).
    tree.nodes[0].lazy_fees = Zero::zero();
    tree.nodes[0].fees = Zero::zero();

    let mut nodes = tree.nodes.clone().into_inner();
    let amount = if withdraw_all { _1 } else { _1_2 };
    let account = 3;
    nodes[0].stake -= amount;
    let mut account_to_index = tree.account_to_index.clone().into_inner();
    let mut abandoned_nodes = tree.abandoned_nodes.clone().into_inner();
    if withdraw_all {
        nodes[0].account = None;
        account_to_index.remove(&account);
        abandoned_nodes.push(0);
    }

    tree.exit(&account, amount).unwrap();
    assert_liquidity_tree_state!(tree, nodes, account_to_index, abandoned_nodes);
}

#[test_case(false)]
#[test_case(true)]
fn exit_middle_works(withdraw_all: bool) {
    let mut tree = utility::create_test_tree();
    // Remove lazy fees on the path to the node (and actual fees from the node).
    tree.nodes[0].lazy_fees = Zero::zero();
    tree.nodes[1].lazy_fees = Zero::zero();
    tree.nodes[3].lazy_fees = Zero::zero();
    tree.nodes[3].fees = Zero::zero();

    let mut nodes = tree.nodes.clone().into_inner();
    let amount = if withdraw_all { _3 } else { _1 };
    let account = 5;
    nodes[0].descendant_stake -= amount;
    nodes[1].descendant_stake -= amount;
    nodes[3].stake -= amount;
    let mut account_to_index = tree.account_to_index.clone().into_inner();
    let mut abandoned_nodes = tree.abandoned_nodes.clone().into_inner();
    if withdraw_all {
        nodes[3].account = None;
        account_to_index.remove(&account);
        abandoned_nodes.push(3);
    }

    tree.exit(&account, amount).unwrap();
    assert_liquidity_tree_state!(tree, nodes, account_to_index, abandoned_nodes);
}

#[test_case(false)]
#[test_case(true)]
fn exit_leaf_works(withdraw_all: bool) {
    let mut tree = utility::create_test_tree();
    // Remove lazy fees on the path to the node (and actual fees from the node).
    tree.nodes[0].lazy_fees = Zero::zero();
    tree.nodes[1].lazy_fees = Zero::zero();
    tree.nodes[3].lazy_fees = Zero::zero();
    tree.nodes[7].lazy_fees = Zero::zero();
    tree.nodes[7].fees = Zero::zero();

    let mut nodes = tree.nodes.clone().into_inner();
    let amount = if withdraw_all { _12 } else { _1 };
    let account = 6;
    nodes[0].descendant_stake -= amount;
    nodes[1].descendant_stake -= amount;
    nodes[3].descendant_stake -= amount;
    nodes[7].stake -= amount;
    let mut account_to_index = tree.account_to_index.clone().into_inner();
    let mut abandoned_nodes = tree.abandoned_nodes.clone().into_inner();
    if withdraw_all {
        nodes[7].account = None;
        account_to_index.remove(&account);
        abandoned_nodes.push(7);
    }

    tree.exit(&account, amount).unwrap();
    assert_liquidity_tree_state!(tree, nodes, account_to_index, abandoned_nodes);
}

#[test_case(3, _1 + 1)]
#[test_case(9, _3 + 1)]
#[test_case(5, _3 + 1)]
#[test_case(7, _1 + 1)]
#[test_case(6, _12 + 1)]
#[test_case(8, _4 + 1)]
fn exit_fails_on_insufficient_stake(account: AccountIdOf<Runtime>, amount: BalanceOf<Runtime>) {
    let mut tree = utility::create_test_tree();
    // Clear unclaimed fees.
    for node in tree.nodes.iter_mut() {
        node.fees = Zero::zero();
        node.lazy_fees = Zero::zero();
    }
    assert_err!(
        tree.exit(&account, amount),
        LiquidityTreeError::InsufficientStake.into_dispatch_error::<Runtime>(),
    );
}

#[test]
fn exit_fails_on_unclaimed_fees_at_root() {
    let mut tree = utility::create_test_tree();
    // Clear unclaimed fees except for root.
    tree.nodes[0].lazy_fees = _1;
    tree.nodes[1].lazy_fees = Zero::zero();
    tree.nodes[3].fees = Zero::zero();
    tree.nodes[3].lazy_fees = Zero::zero();
    assert_err!(
        tree.exit(&5, 1),
        LiquidityTreeError::UnwithdrawnFees.into_dispatch_error::<Runtime>()
    );
}

#[test]
fn exit_fails_on_unclaimed_fees_on_middle_of_path() {
    let mut tree = utility::create_test_tree();
    // Clear unclaimed fees except for the middle node.
    tree.nodes[3].fees = Zero::zero();
    tree.nodes[3].lazy_fees = Zero::zero();
    assert_err!(
        tree.exit(&5, 1),
        LiquidityTreeError::UnwithdrawnFees.into_dispatch_error::<Runtime>()
    );
}

#[test]
fn exit_fails_on_unclaimed_fees_at_last_node_due_to_lazy_fees() {
    let mut tree = utility::create_test_tree();
    // Clear unclaimed fees except for the last node.
    tree.nodes[1].lazy_fees = Zero::zero();
    // This ensures that the error is caused by propagated lazy fees sitting in the node.
    tree.nodes[3].fees = Zero::zero();
    assert_err!(
        tree.exit(&5, 1),
        LiquidityTreeError::UnwithdrawnFees.into_dispatch_error::<Runtime>()
    );
}

#[test]
fn exit_fails_on_unclaimed_fees_at_last_node_due_to_fees() {
    let mut tree = utility::create_test_tree();
    // Clear unclaimed fees except for the last node.
    tree.nodes[1].lazy_fees = Zero::zero();
    // This ensures that the error is caused by normal fees.
    tree.nodes[3].lazy_fees = Zero::zero();
    assert_err!(
        tree.exit(&5, 1),
        LiquidityTreeError::UnwithdrawnFees.into_dispatch_error::<Runtime>()
    );
}
