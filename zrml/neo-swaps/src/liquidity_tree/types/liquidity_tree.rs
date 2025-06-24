// Copyright 2023-2025 Forecasting Technologies LTD.
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
    liquidity_tree::{
        traits::LiquidityTreeHelper,
        types::{
            LiquidityTreeChildIndices, LiquidityTreeError, LiquidityTreeMaxNodes, Node,
            StorageOverflowError, UpdateDescendantStakeOperation,
        },
    },
    traits::LiquiditySharesManager,
    BalanceOf, Config, Error,
};
use alloc::{vec, vec::Vec};
use frame_support::{
    ensure,
    pallet_prelude::RuntimeDebugNoBound,
    storage::{bounded_btree_map::BoundedBTreeMap, bounded_vec::BoundedVec},
    traits::Get,
    CloneNoBound, PartialEqNoBound,
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
pub(crate) struct LiquidityTree<T, U>
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

/// Execution path info for `join` calls.
#[derive(Debug, PartialEq)]
pub(crate) enum BenchmarkInfo {
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

impl<T, U> LiquidityTreeHelper<T> for LiquidityTree<T, U>
where
    T: Config,
    U: Get<u32>,
{
    type Node = Node<T>;

    fn propagate_fees_to_node(&mut self, index: u32) -> DispatchResult {
        let path = self.path_to_node(index, None)?;
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
        if index == 0 {
            None
        } else {
            index.checked_sub(1)?.checked_div(2)
        }
    }

    fn path_to_node(
        &self,
        index: u32,
        opt_iterations: Option<usize>,
    ) -> Result<Vec<u32>, DispatchError> {
        let remaining_iterations =
            opt_iterations.unwrap_or(Self::max_depth().checked_add_res(&1)? as usize);
        let remaining_iterations = remaining_iterations
            .checked_sub(1)
            .ok_or(LiquidityTreeError::MaxIterationsReached.into_dispatch_error::<T>())?;
        if let Some(parent_index) = self.parent_index(index) {
            let mut path = self.path_to_node(parent_index, Some(remaining_iterations))?;
            path.push(index);
            Ok(path)
        } else {
            Ok(vec![0])
        }
    }

    fn take_last_abandoned_node_index(&mut self) -> Option<u32> {
        self.abandoned_nodes.pop()
    }

    fn peek_next_free_leaf(&self) -> Option<u32> {
        let node_count = self.node_count();
        if node_count < Self::max_node_count() {
            Some(node_count)
        } else {
            None
        }
    }

    fn update_descendant_stake_of_ancestors(
        &mut self,
        index: u32,
        delta: BalanceOf<T>,
        op: UpdateDescendantStakeOperation,
    ) -> DispatchResult {
        for &i in self.path_to_node(index, None)?.iter() {
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

    fn node_count(&self) -> u32 {
        self.nodes.len() as u32
    }

    fn get_node(&self, index: u32) -> Result<&Self::Node, DispatchError> {
        self.nodes
            .get(index as usize)
            .ok_or(LiquidityTreeError::NodeNotFound.into_dispatch_error::<T>())
    }

    fn get_node_mut(&mut self, index: u32) -> Result<&mut Self::Node, DispatchError> {
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
        F: FnOnce(&mut Self::Node) -> DispatchResult,
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
