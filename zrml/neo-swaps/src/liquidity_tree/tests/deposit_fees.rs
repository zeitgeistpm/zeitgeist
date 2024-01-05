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
fn deposit_fees_works_root() {
    let mut tree = utility::create_test_tree();
    let mut nodes = tree.nodes.clone().into_inner();
    let account_to_index = tree.account_to_index.clone().into_inner();
    let abandoned_nodes = tree.abandoned_nodes.clone().into_inner();
    let amount = _12;
    nodes[0].lazy_fees += amount;
    tree.deposit_fees(amount).unwrap();
    assert_liquidity_tree_state!(tree, nodes, account_to_index, abandoned_nodes);
}

#[test]
fn deposit_fees_works_no_root() {
    let mut tree = utility::create_test_tree();
    tree.nodes[0].account = None;
    tree.nodes[1].stake = Zero::zero();
    tree.nodes[2].fees = Zero::zero();
    let mut nodes = tree.nodes.clone().into_inner();
    let account_to_index = tree.account_to_index.clone().into_inner();
    let abandoned_nodes = tree.abandoned_nodes.clone().into_inner();
    let amount = _12;
    nodes[0].lazy_fees += amount;
    tree.deposit_fees(amount).unwrap();
    assert_liquidity_tree_state!(tree, nodes, account_to_index, abandoned_nodes);
}
