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

use crate::{
    assert_liquidity_tree_state, create_b_tree_map,
    liquidity_tree::{
        traits::liquidity_tree_helper::LiquidityTreeHelper,
        types::{LiquidityTreeError, Node},
    },
    mock::Runtime,
    traits::liquidity_shares_manager::LiquiditySharesManager,
    LiquidityTreeOf,
};
use alloc::collections::BTreeMap;
use frame_support::assert_err;
use sp_runtime::traits::Zero;
use zeitgeist_primitives::constants::base_multiples::*;

mod deposit_fees;
mod exit;
mod join;
mod shares_of;
mod total_shares;
mod withdraw_fees;

/// Most tests use the same pattern:
///
/// - Create a test tree. In some cases the test tree needs to be modified as part of the test
///   setup.
/// - Clone the contents of the tests tree and modify them to obtain the expected state of the tree
///   after executing the test.
/// - Run the test.
/// - Verify state.
mod utility {
    use super::*;

    /// Create the following liquidity tree:
    ///
    ///                                    (3, _1, _2, _23, 0)
    ///                                      /               \
    ///                      (None, 0, 0, _20, _4)           (9, _3, _5, 0, 0)
    ///                          /         \                          /         \
    ///        (5, _3, _1, _12, _3)    (7, _1, _1, _4, _3)  (None, 0, 0, 0, 0)  (None, 0, 0, 0, 0)
    ///              /       \                   /       
    /// (6, _12, _1, 0, _3)  (None, 0, 0, 0, 0)  (8, _4, _1, 0, 0)
    ///
    /// This tree is used in most tests, but will sometime have to be modified.
    pub(super) fn create_test_tree() -> LiquidityTreeOf<Runtime> {
        LiquidityTreeOf::<Runtime> {
            nodes: vec![
                // Root
                Node::<Runtime> {
                    account: Some(3),
                    stake: _1,
                    fees: _2,
                    descendant_stake: _23,
                    lazy_fees: Zero::zero(),
                },
                // Depth 1
                Node::<Runtime> {
                    account: None,
                    stake: Zero::zero(),
                    fees: Zero::zero(),
                    descendant_stake: _20,
                    lazy_fees: _4,
                },
                Node::<Runtime> {
                    account: Some(9),
                    stake: _3,
                    fees: _5,
                    descendant_stake: Zero::zero(),
                    lazy_fees: Zero::zero(),
                },
                // Depth 2
                Node::<Runtime> {
                    account: Some(5),
                    stake: _3,
                    fees: _1,
                    descendant_stake: _12,
                    lazy_fees: _3,
                },
                Node::<Runtime> {
                    account: Some(7),
                    stake: _1,
                    fees: _1,
                    descendant_stake: _4,
                    lazy_fees: _3,
                },
                Node::<Runtime> {
                    account: None,
                    stake: Zero::zero(),
                    fees: Zero::zero(),
                    descendant_stake: Zero::zero(),
                    lazy_fees: Zero::zero(),
                },
                Node::<Runtime> {
                    account: None,
                    stake: Zero::zero(),
                    fees: Zero::zero(),
                    descendant_stake: Zero::zero(),
                    lazy_fees: Zero::zero(),
                },
                // Depth 3
                Node::<Runtime> {
                    account: Some(6),
                    stake: _12,
                    fees: _1,
                    descendant_stake: Zero::zero(),
                    lazy_fees: _3,
                },
                Node::<Runtime> {
                    account: None,
                    stake: Zero::zero(),
                    fees: Zero::zero(),
                    descendant_stake: Zero::zero(),
                    lazy_fees: Zero::zero(),
                },
                Node::<Runtime> {
                    account: Some(8),
                    stake: _4,
                    fees: _1,
                    descendant_stake: Zero::zero(),
                    lazy_fees: Zero::zero(),
                },
            ]
            .try_into()
            .unwrap(),
            account_to_index: create_b_tree_map!({3 => 0, 9 => 2, 5 => 3, 7 => 4, 6 => 7, 8 => 9})
                .try_into()
                .unwrap(),
            abandoned_nodes: vec![1, 5, 6, 8].try_into().unwrap(),
        }
    }

    /// Create a full tree. All nodes have the same stake of 1.
    pub(super) fn create_full_tree() -> LiquidityTreeOf<Runtime> {
        let max_depth = LiquidityTreeOf::<Runtime>::max_depth();
        let node_count = LiquidityTreeOf::<Runtime>::max_node_count();
        let nodes = (0..node_count)
            .map(|a| Node::<Runtime>::new(a as u128, 1))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        let account_to_index =
            (0..node_count).map(|a| (a as u128, a)).collect::<BTreeMap<_, _>>().try_into().unwrap();
        let mut tree = LiquidityTreeOf::<Runtime> {
            nodes,
            account_to_index,
            abandoned_nodes: vec![].try_into().unwrap(),
        };
        // Nodes have the wrong descendant stake at this point, so let's fix that.
        for (index, node) in tree.nodes.iter_mut().enumerate() {
            let exp = max_depth + 1 - (index + 1).checked_ilog2().unwrap();
            node.descendant_stake = 2u128.pow(exp) - 2;
        }
        tree
    }
}
