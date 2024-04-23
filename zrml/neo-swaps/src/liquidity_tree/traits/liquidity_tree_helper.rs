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

use crate::{
    liquidity_tree::types::{LiquidityTreeChildIndices, UpdateDescendantStakeOperation},
    BalanceOf, Config,
};
use alloc::vec::Vec;
use sp_runtime::{DispatchError, DispatchResult};

/// A collection of member functions used in the implementation of `LiquiditySharesManager` for
/// `LiquidityTree`.
pub(crate) trait LiquidityTreeHelper<T>
where
    T: Config,
{
    type Node;

    /// Propagate lazy fees from the tree's root to the node at `index`.
    ///
    /// Propagation includes moving the part of the lazy fees of each node on the path to the node
    /// at `index` to the node's fees.
    ///
    /// Assuming correct state this function can only fail if there is no node at `index`.
    fn propagate_fees_to_node(&mut self, index: u32) -> DispatchResult;

    /// Propagate lazy fees from the node at `index` to its children.
    ///
    /// Propagation includes moving the node's share of the lazy fees to the node's fees.
    ///
    /// Assuming correct state this function can only fail if there is no node at `index`.
    fn propagate_fees(&mut self, index: u32) -> DispatchResult;

    /// Return the indices of the children of the node at `index`.
    fn children(&self, index: u32) -> Result<LiquidityTreeChildIndices, DispatchError>;

    /// Return the index of a node's parent; `None` if `index` is `0u32`, i.e. the node is root.
    fn parent_index(&self, index: u32) -> Option<u32>;

    /// Return a path from the tree's root to the node at `index`.
    ///
    /// The return value is a vector of the indices of the nodes of the path, starting with the
    /// root and including `index`. The parameter `opt_iterations` specifies how many iterations the
    /// operation is allowed to take and can be used to terminate if the number of iterations
    /// exceeds the expected amount by setting it to `None`.
    fn path_to_node(
        &self,
        index: u32,
        opt_iterations: Option<usize>,
    ) -> Result<Vec<u32>, DispatchError>;

    /// Pops the most recently abandoned node's index from the stack. Returns `None` if there's no
    /// abandoned node.
    fn take_last_abandoned_node_index(&mut self) -> Option<u32>;

    /// Returns the index of the next free leaf; `None` if the tree is full.
    fn peek_next_free_leaf(&self) -> Option<u32>;

    /// Mutate a node's ancestor's `descendant_stake` field.
    ///
    /// # Parameters
    ///
    /// - `index`: The index of the node.
    /// - `delta`: The (absolute) amount by which to modfiy the descendant stake.
    /// - `op`: The sign of the delta.
    fn update_descendant_stake_of_ancestors(
        &mut self,
        index: u32,
        delta: BalanceOf<T>,
        op: UpdateDescendantStakeOperation,
    ) -> DispatchResult;

    /// Return the number of nodes in the tree. Note that abandoned nodes are counted.
    fn node_count(&self) -> u32;

    /// Get a reference to the node at `index`.
    fn get_node(&self, index: u32) -> Result<&Self::Node, DispatchError>;

    /// Get a mutable reference to the node at `index`.
    fn get_node_mut(&mut self, index: u32) -> Result<&mut Self::Node, DispatchError>;

    /// Get the node which belongs to `account`.
    fn map_account_to_index(&self, account: &T::AccountId) -> Result<u32, DispatchError>;

    /// Mutate the node at `index` using `mutator`.
    fn mutate_node<F>(&mut self, index: u32, mutator: F) -> DispatchResult
    where
        F: FnOnce(&mut Self::Node) -> DispatchResult;

    /// Return the maximum allowed depth of the tree.
    fn max_depth() -> u32;

    /// Return the maximum allowed amount of nodes in the tree.
    fn max_node_count() -> u32;
}
