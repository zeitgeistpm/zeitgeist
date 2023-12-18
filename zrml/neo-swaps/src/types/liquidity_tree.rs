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

use crate::{traits::LiquiditySharesManager, BalanceOf, Config, Error};
use alloc::{vec, vec::Vec};
use core::marker::PhantomData;
use frame_support::{
    ensure,
    pallet_prelude::RuntimeDebugNoBound,
    storage::{bounded_btree_map::BoundedBTreeMap, bounded_vec::BoundedVec},
    traits::Get,
    CloneNoBound, PalletError, PartialEqNoBound,
};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{
    traits::{AtLeast32BitUnsigned, CheckedSub, Zero},
    DispatchError, DispatchResult,
};
use zeitgeist_primitives::math::{
    checked_ops_res::{CheckedAddRes, CheckedMulRes, CheckedSubRes},
    fixed::FixedMulDiv,
};

/// Gets the maximum number of nodes allowed in the liquidity tree as a function of its depth.
/// Saturates at `u32::MAX`, but will warn about this in DEBUG.
///
/// # Generics
///
/// - `D`: A getter for the depth of the tree.
pub struct LiquidityTreeMaxNodes<D>(PhantomData<D>);

impl<D> Get<u32> for LiquidityTreeMaxNodes<D>
where
    D: Get<u32>,
{
    fn get() -> u32 {
        debug_assert!(D::get() < 31, "LiquidityTreeMaxNodes::get(): Integer overflow");
        2u32.saturating_pow(D::get() + 1).saturating_sub(1)
    }
}

/// A segment tree used to track balances of liquidity shares which allows `O(log(n))` distribution
/// of fees.
///
/// Each liquidity provider owns exactly one node of the tree which records their stake and fees.
/// When a liquidity provider leaves the tree, the node is not removed from the tree, but marked as
/// _abandoned_ instead. Abandoned nodes are reassigned when new LPs enter the tree. Nodes are added
/// to the leaves of the tree only if there are no abandoned nodes to reassign.
///
/// Fees are lazily propagated down the tree. This allows fees to be deposited to the tree in `O(1)`
/// (fees deposited at the root and later propagated down). If a particular node requires to know
/// what its fees are, propagating fees to this node takes `O(depth)` operations (or, equivalently,
/// `O(log_2(node_count))`).
///
/// # Generics
///
/// - `T`: The pallet configuration.
/// - `U`: A getter for the maximum depth of the tree. Using a depth larger than `31` will result in
///   undefined behavior.
#[derive(
    CloneNoBound, Decode, Encode, Eq, MaxEncodedLen, PartialEqNoBound, RuntimeDebugNoBound, TypeInfo,
)]
#[scale_info(skip_type_params(T, U))]
pub struct LiquidityTree<T, U>
where
    T: Config,
    U: Get<u32>,
{
    /// A vector which holds the nodes of the tree. The nodes are ordered by depth (the root is the
    /// first element of `nodes`) and from left to right. For example, the right-most grandchild of
    /// the root is at index `6`.
    pub(crate) nodes: BoundedVec<Node<T>, LiquidityTreeMaxNodes<U>>,
    /// Maps an account to the node that belongs to it.
    pub(crate) account_to_index: BoundedBTreeMap<T::AccountId, u32, LiquidityTreeMaxNodes<U>>,
    /// A vector that contains the indices of abandoned nodes. Sorted in the order in which the
    /// nodes were abandoned, with the last element of the vector being the most recently abandoned
    /// node.
    pub(crate) abandoned_nodes: BoundedVec<u32, LiquidityTreeMaxNodes<U>>,
}

impl<T, U> LiquidityTree<T, U>
where
    T: Config,
    U: Get<u32>,
{
    /// Create a new liquidity tree.
    ///
    /// # Parameters
    ///
    /// - `account`: The account to which the tree's root belongs.
    /// - `stake`: The stake of the tree's root.
    pub(crate) fn new(
        account: T::AccountId,
        stake: BalanceOf<T>,
    ) -> Result<LiquidityTree<T, U>, DispatchError> {
        let root = Node::new(account.clone(), stake);
        let nodes = vec![root]
            .try_into()
            .map_err(|_| StorageOverflowError::Nodes.into_dispatch_error::<T>())?;
        let mut account_to_index = BoundedBTreeMap::<_, _, _>::new();
        account_to_index
            .try_insert(account, 0u32)
            .map_err(|_| StorageOverflowError::AccountToIndex.into_dispatch_error::<T>())?;
        let abandoned_nodes = vec![]
            .try_into()
            .map_err(|_| StorageOverflowError::AbandonedNodes.into_dispatch_error::<T>())?;
        Ok(LiquidityTree { nodes, account_to_index, abandoned_nodes })
    }
}

/// Type for nodes of a liquidity tree.
///
/// # Notes
///
/// - `descendant_stake` does not contain the stake of `self`.
/// - `lazy_fees`, when propagated, is distributed not only to the descendants of `self`, but also to
///   `self`.
#[derive(Clone, Decode, Encode, Eq, MaxEncodedLen, PartialEq, RuntimeDebugNoBound, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub(crate) struct Node<T: Config> {
    /// The account that the node belongs to. `None` signifies an abandoned node.
    pub account: Option<T::AccountId>,
    /// The stake belonging to the owner.
    pub stake: BalanceOf<T>,
    /// The fees owed to the owner.
    pub fees: BalanceOf<T>,
    /// The sum of the stake of all descendants of this node.
    pub descendant_stake: BalanceOf<T>,
    /// The amount of fees to be lazily propagated down the tree.
    pub lazy_fees: BalanceOf<T>,
}

impl<T> Node<T>
where
    T: Config,
{
    /// Create a new node with `stake` belonging to `account`.
    pub(crate) fn new(account: T::AccountId, stake: BalanceOf<T>) -> Node<T> {
        Node {
            account: Some(account),
            stake,
            fees: 0u8.into(),
            descendant_stake: 0u8.into(),
            lazy_fees: 0u8.into(),
        }
    }

    /// Return the total stake of the node (the node's stake plus the sum of descendant's stakes).
    pub(crate) fn total_stake(&self) -> Result<BalanceOf<T>, DispatchError> {
        self.stake.checked_add_res(&self.descendant_stake)
    }

    /// Return `true` is the node is a leaf in the sense that none of its descendants hold any
    /// stake. (Strictly speaking, it's not always a leaf, as there might be abandoned nodes!)
    pub(crate) fn is_weak_leaf(&self) -> bool {
        self.descendant_stake == Zero::zero()
    }
}

/// Execution path info for `join` calls.
#[derive(Debug, PartialEq)]
pub enum BenchmarkInfo {
    /// The LP already owns a node in the tree.
    InPlace,
    /// The LP is reassigned an abandoned node.
    Reassigned,
    /// The LP is assigned a leaf of the tree.
    Leaf,
}

impl<T, U> LiquiditySharesManager<T> for LiquidityTree<T, U>
where
    T: Config + frame_system::Config,
    T::AccountId: PartialEq<T::AccountId>,
    BalanceOf<T>: AtLeast32BitUnsigned + Copy + Zero,
    U: Get<u32>,
{
    type JoinBenchmarkInfo = BenchmarkInfo;

    fn join(
        &mut self,
        who: &T::AccountId,
        stake: BalanceOf<T>,
    ) -> Result<Self::JoinBenchmarkInfo, DispatchError> {
        let opt_index = self.account_to_index.get(who);
        let (index, benchmark_info) = if let Some(&index) = opt_index {
            // Pile onto existing account.
            self.propagate_fees_to_node(index)?;
            let node = self.get_node_mut(index)?;
            node.stake = node.stake.checked_add_res(&stake)?;
            (index, BenchmarkInfo::InPlace)
        } else {
            // Push onto new node.
            let (index, benchmark_info) = if let Some(index) = self.take_last_abandoned_node_index()
            {
                self.propagate_fees_to_node(index)?;
                let node = self.get_node_mut(index)?;
                node.account = Some(who.clone());
                node.stake = stake;
                node.fees = Zero::zero(); // Not necessary, but better safe than sorry.
                // Don't change `descendant_stake`; we're still maintaining it for abandoned
                // nodes.
                node.lazy_fees = Zero::zero();
                (index, BenchmarkInfo::Reassigned)
            } else if let Some(index) = self.peek_next_free_leaf() {
                // Add new leaf. Propagate first so we don't propagate fees to the new leaf.
                if let Some(parent_index) = self.parent_index(index) {
                    self.propagate_fees_to_node(parent_index)?;
                }
                self.nodes
                    .try_push(Node::new(who.clone(), stake))
                    .map_err(|_| StorageOverflowError::Nodes.into_dispatch_error::<T>())?;
                (index, BenchmarkInfo::Leaf)
            } else {
                return Err(LiquidityTreeError::TreeIsFull.into_dispatch_error::<T>());
            };
            self.account_to_index
                .try_insert(who.clone(), index)
                .map_err(|_| StorageOverflowError::AccountToIndex.into_dispatch_error::<T>())?;
            (index, benchmark_info)
        };
        if let Some(parent_index) = self.parent_index(index) {
            self.update_descendant_stake_of_ancestors(
                parent_index,
                stake,
                UpdateDescendantStakeOperation::Add,
            )?;
        }
        Ok(benchmark_info)
    }

    fn exit(&mut self, who: &T::AccountId, stake: BalanceOf<T>) -> DispatchResult {
        let index = self.map_account_to_index(who)?;
        self.propagate_fees_to_node(index)?;
        let node = self.get_node_mut(index)?;
        ensure!(
            node.fees == Zero::zero(),
            LiquidityTreeError::UnwithdrawnFees.into_dispatch_error::<T>()
        );
        node.stake = node
            .stake
            .checked_sub(&stake)
            .ok_or(LiquidityTreeError::InsufficientStake.into_dispatch_error::<T>())?;
        if node.stake == Zero::zero() {
            node.account = None;
            self.abandoned_nodes
                .try_push(index)
                .map_err(|_| StorageOverflowError::AbandonedNodes.into_dispatch_error::<T>())?;
            let _ = self.account_to_index.remove(who);
        }
        if let Some(parent_index) = self.parent_index(index) {
            self.update_descendant_stake_of_ancestors(
                parent_index,
                stake,
                UpdateDescendantStakeOperation::Sub,
            )?;
        }
        Ok(())
    }

    fn split(
        &mut self,
        _sender: &T::AccountId,
        _receiver: &T::AccountId,
        _amount: BalanceOf<T>,
    ) -> DispatchResult {
        Err(Error::<T>::NotImplemented.into())
    }

    fn deposit_fees(&mut self, amount: BalanceOf<T>) -> DispatchResult {
        let root = self.get_node_mut(0u32)?;
        root.lazy_fees = root.lazy_fees.checked_add_res(&amount)?;
        Ok(())
    }

    fn withdraw_fees(&mut self, who: &T::AccountId) -> Result<BalanceOf<T>, DispatchError> {
        let index = self.map_account_to_index(who)?;
        self.propagate_fees_to_node(index)?;
        let node = self.get_node_mut(index)?;
        let fees = node.fees;
        node.fees = Zero::zero();
        Ok(fees)
    }

    fn shares_of(&self, who: &T::AccountId) -> Result<BalanceOf<T>, DispatchError> {
        let index = self.map_account_to_index(who)?;
        let node = self.get_node(index)?;
        Ok(node.stake)
    }

    fn total_shares(&self) -> Result<BalanceOf<T>, DispatchError> {
        let root = self.get_node(0u32)?;
        root.total_stake()
    }
}

/// Structure for managing children in a liquidity tree.
pub(crate) struct LiquidityTreeChildIndices {
    /// Left-hand side child; `None` if there's no left-hand side child.
    lhs: Option<u32>,
    /// Right-hand side child; `None` if there's no right-hand side child.
    rhs: Option<u32>,
}

impl LiquidityTreeChildIndices {
    /// Applies a `mutator` function to each child if it exists.
    pub fn apply<F>(&self, mut mutator: F) -> Result<(), DispatchError>
    where
        F: FnMut(u32) -> Result<(), DispatchError>,
    {
        if let Some(lhs) = self.lhs {
            mutator(lhs)?;
        }
        if let Some(rhs) = self.rhs {
            mutator(rhs)?;
        }
        Ok(())
    }
}

// Implement `From` for destructuring
impl From<LiquidityTreeChildIndices> for (Option<u32>, Option<u32>) {
    fn from(child_indices: LiquidityTreeChildIndices) -> (Option<u32>, Option<u32>) {
        (child_indices.lhs, child_indices.rhs)
    }
}

/// Type for specifying a sign for `update_descendant_stake_of_ancestors`.
pub(crate) enum UpdateDescendantStakeOperation {
    Add,
    Sub,
}

/// A collection of member functions used in the implementation of `LiquiditySharesManager` for
/// `LiquidityTree`.
pub(crate) trait LiquidityTreeHelper<T>
where
    T: Config,
{
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
    /// root and including `index`.
    fn path_to_node(&self, index: u32) -> Result<Vec<u32>, DispatchError>;

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

    /// Mutate each child of the node at `index` using `mutator`.
    fn mutate_each_child<F>(&mut self, index: u32, mutator: F) -> DispatchResult
    where
        F: FnMut(&mut Node<T>) -> DispatchResult;

    /// Return the number of nodes in the tree. Note that abandoned nodes are counted.
    fn node_count(&self) -> u32;

    /// Get a reference to the node at `index`.
    fn get_node(&self, index: u32) -> Result<&Node<T>, DispatchError>;

    /// Get a mutable reference to the node at `index`.
    fn get_node_mut(&mut self, index: u32) -> Result<&mut Node<T>, DispatchError>;

    /// Get the node which belongs to `account`.
    fn map_account_to_index(&self, account: &T::AccountId) -> Result<u32, DispatchError>;

    /// Mutate the node at `index` using `mutator`.
    fn mutate_node<F>(&mut self, index: u32, mutator: F) -> DispatchResult
    where
        F: FnOnce(&mut Node<T>) -> DispatchResult;

    /// Return the maximum allowed depth of the tree.
    fn max_depth() -> u32;

    /// Return the maximum allowed amount of nodes in the tree.
    fn max_node_count() -> u32;
}

impl<T, U> LiquidityTreeHelper<T> for LiquidityTree<T, U>
where
    T: Config,
    U: Get<u32>,
{
    fn propagate_fees_to_node(&mut self, index: u32) -> DispatchResult {
        let path = self.path_to_node(index)?;
        for i in path {
            self.propagate_fees(i)?;
        }
        Ok(())
    }

    fn propagate_fees(&mut self, index: u32) -> DispatchResult {
        let node = self.get_node(index)?;
        if node.total_stake()? == Zero::zero() {
            return Ok(()); // Don't propagate if there are no LPs under this node.
        }
        if node.is_weak_leaf() {
            self.mutate_node(index, |node| {
                node.fees = node.fees.checked_add_res(&node.lazy_fees)?;
                Ok(())
            })?;
        } else {
            // Temporary storage to ensure that the borrow checker doesn't get upset.
            let node_descendant_stake = node.descendant_stake;
            // The lazy fees that will be propagated down the tree.
            let mut remaining_lazy_fees =
                node.descendant_stake.bmul_bdiv(node.lazy_fees, node.total_stake()?)?;
            // The fees that stay at this node.
            let fees = node.lazy_fees.checked_sub_res(&remaining_lazy_fees)?;
            self.mutate_node(index, |node| {
                node.fees = node.fees.checked_add_res(&fees)?;
                Ok(())
            })?;
            let (opt_lhs_index, opt_rhs_index) = self.children(index)?.into();
            if let Some(lhs_index) = opt_lhs_index {
                self.mutate_node(lhs_index, |lhs_node| {
                    // The descendant's share of the stake:
                    let child_lazy_fees = lhs_node
                        .total_stake()?
                        .bmul_bdiv(remaining_lazy_fees, node_descendant_stake)?;
                    lhs_node.lazy_fees = lhs_node.lazy_fees.checked_add_res(&child_lazy_fees)?;
                    remaining_lazy_fees = remaining_lazy_fees.checked_sub_res(&child_lazy_fees)?;
                    Ok(())
                })?;
            }
            if let Some(rhs_index) = opt_rhs_index {
                self.mutate_node(rhs_index, |rhs_node| {
                    rhs_node.lazy_fees =
                        rhs_node.lazy_fees.checked_add_res(&remaining_lazy_fees)?;
                    Ok(())
                })?;
            }
        }
        self.mutate_node(index, |node| {
            node.lazy_fees = Zero::zero();
            Ok(())
        })?;
        Ok(())
    }

    fn children(&self, index: u32) -> Result<LiquidityTreeChildIndices, DispatchError> {
        let calculate_child =
            |child_index: u32| Some(child_index).filter(|&i| i < self.node_count());
        let left_child_index = index.checked_mul_res(&2)?.checked_add_res(&1)?;
        let lhs = calculate_child(left_child_index);
        let right_child_index = left_child_index.checked_add_res(&1)?;
        let rhs = calculate_child(right_child_index);
        Ok(LiquidityTreeChildIndices { lhs, rhs })
    }

    fn parent_index(&self, index: u32) -> Option<u32> {
        if index == 0 { None } else { index.checked_sub(1)?.checked_div(2) }
    }

    fn path_to_node(&self, mut index: u32) -> Result<Vec<u32>, DispatchError> {
        let mut path = Vec::new();
        let mut iterations = 0;
        let max_iterations = Self::max_depth().checked_add_res(&1)?;
        while let Some(parent_index) = self.parent_index(index) {
            if iterations == max_iterations {
                return Err(LiquidityTreeError::MaxIterationsReached.into_dispatch_error::<T>());
            }
            path.push(index);
            index = parent_index;
            iterations = iterations.checked_add_res(&1)?;
        }
        path.push(0u32); // The tree's root is not considered in the loop above.
        path.reverse(); // The path should be from root to the node
        Ok(path)
    }

    fn take_last_abandoned_node_index(&mut self) -> Option<u32> {
        self.abandoned_nodes.pop()
    }

    fn peek_next_free_leaf(&self) -> Option<u32> {
        let node_count = self.node_count();
        if node_count < Self::max_node_count() { Some(node_count) } else { None }
    }

    fn update_descendant_stake_of_ancestors(
        &mut self,
        index: u32,
        delta: BalanceOf<T>,
        op: UpdateDescendantStakeOperation,
    ) -> DispatchResult {
        for &i in self.path_to_node(index)?.iter() {
            let node = self.get_node_mut(i)?;
            match op {
                UpdateDescendantStakeOperation::Add => {
                    node.descendant_stake = node.descendant_stake.checked_add_res(&delta)?
                }
                UpdateDescendantStakeOperation::Sub => {
                    node.descendant_stake = node.descendant_stake.checked_sub_res(&delta)?
                }
            }
        }
        Ok(())
    }

    fn mutate_each_child<F>(&mut self, index: u32, mut mutator: F) -> DispatchResult
    where
        F: FnMut(&mut Node<T>) -> DispatchResult,
    {
        let child_indices = self.children(index)?;
        child_indices.apply(|index| {
            self.mutate_node(index, |node| mutator(node))?;
            Ok(())
        })?;
        Ok(())
    }

    fn node_count(&self) -> u32 {
        self.nodes.len() as u32
    }

    fn get_node(&self, index: u32) -> Result<&Node<T>, DispatchError> {
        self.nodes
            .get(index as usize)
            .ok_or(LiquidityTreeError::NodeNotFound.into_dispatch_error::<T>())
    }

    fn get_node_mut(&mut self, index: u32) -> Result<&mut Node<T>, DispatchError> {
        self.nodes
            .get_mut(index as usize)
            .ok_or(LiquidityTreeError::NodeNotFound.into_dispatch_error::<T>())
    }

    fn map_account_to_index(&self, who: &T::AccountId) -> Result<u32, DispatchError> {
        self.account_to_index
            .get(who)
            .ok_or(LiquidityTreeError::AccountNotFound.into_dispatch_error::<T>())
            .copied()
    }

    fn mutate_node<F>(&mut self, index: u32, mutator: F) -> DispatchResult
    where
        F: FnOnce(&mut Node<T>) -> DispatchResult,
    {
        let node = self.get_node_mut(index)?;
        mutator(node)
    }

    fn max_depth() -> u32 {
        U::get()
    }

    fn max_node_count() -> u32 {
        LiquidityTreeMaxNodes::<U>::get()
    }
}

#[derive(Decode, Encode, Eq, PartialEq, PalletError, RuntimeDebugNoBound, TypeInfo)]
pub enum LiquidityTreeError {
    /// There is no node which belongs to this account.
    AccountNotFound,
    /// There is no node with this index.
    NodeNotFound,
    /// Operation can't be executed while there are unclaimed fees.
    UnwithdrawnFees,
    /// The liquidity tree is full and can't accept any new nodes.
    TreeIsFull,
    /// This node doesn't hold enough stake.
    InsufficientStake,
    /// A while loop exceeded the expected number of iterations. This is unexpected behavior.
    MaxIterationsReached,
    /// Unexpected storage overflow.
    StorageOverflow(StorageOverflowError),
}

#[derive(Decode, Encode, Eq, PartialEq, PalletError, RuntimeDebugNoBound, TypeInfo)]
pub enum StorageOverflowError {
    /// Encountered a storage overflow when trying to push onto the `nodes` vector.
    Nodes,
    /// Encountered a storage overflow when trying to push onto the `account_to_index` map.
    AccountToIndex,
    /// Encountered a storage overflow when trying to push onto the `abandoned_nodes` vector.
    AbandonedNodes,
}

impl From<StorageOverflowError> for LiquidityTreeError {
    fn from(error: StorageOverflowError) -> LiquidityTreeError {
        LiquidityTreeError::StorageOverflow(error)
    }
}

impl StorageOverflowError {
    pub(crate) fn into_dispatch_error<T>(self) -> DispatchError
    where
        T: Config,
    {
        let liquidity_tree_error: LiquidityTreeError = self.into();
        liquidity_tree_error.into_dispatch_error::<T>()
    }
}

impl<T> From<LiquidityTreeError> for Error<T> {
    fn from(error: LiquidityTreeError) -> Error<T> {
        Error::<T>::LiquidityTreeError(error)
    }
}

impl LiquidityTreeError {
    pub(crate) fn into_dispatch_error<T>(self) -> DispatchError
    where
        T: Config,
    {
        Error::<T>::LiquidityTreeError(self).into()
    }
}

/// Most tests use the same pattern:
///
/// - Create a test tree. In some cases the test tree needs to be modified as part of the test
///   setup.
/// - Clone the contents of the tests tree and modify them to obtain the expected state of the tree
///   after executing the test.
/// - Run the test.
/// - Verify state.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        assert_liquidity_tree_state, consts::*, create_b_tree_map, mock::Runtime, AccountIdOf,
        LiquidityTreeOf,
    };
    use alloc::collections::BTreeMap;
    use frame_support::assert_err;
    use test_case::test_case;

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

    #[test]
    fn shares_of_works() {
        let tree = utility::create_test_tree();
        assert_eq!(tree.shares_of(&3).unwrap(), _1);
        assert_eq!(tree.shares_of(&9).unwrap(), _3);
        assert_eq!(tree.shares_of(&5).unwrap(), _3);
        assert_eq!(tree.shares_of(&7).unwrap(), _1);
        assert_eq!(tree.shares_of(&6).unwrap(), _12);
        assert_eq!(tree.shares_of(&8).unwrap(), _4);
    }

    #[test]
    fn total_shares() {
        let tree = utility::create_test_tree();
        assert_eq!(tree.total_shares().unwrap(), _24);
    }

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
                account_to_index:
                    create_b_tree_map!({3 => 0, 9 => 2, 5 => 3, 7 => 4, 6 => 7, 8 => 9})
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
            let account_to_index = (0..node_count)
                .map(|a| (a as u128, a))
                .collect::<BTreeMap<_, _>>()
                .try_into()
                .unwrap();
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
}
