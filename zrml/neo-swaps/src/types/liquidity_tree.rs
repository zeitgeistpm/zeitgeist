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
    PalletError,
};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{
    traits::{AtLeast32BitUnsigned, CheckedSub, Zero},
    DispatchError, DispatchResult,
};
use zeitgeist_primitives::math::{
    checked_ops_res::{CheckedAddRes, CheckedMulRes, CheckedSubRes},
    fixed::{FixedDiv, FixedMul},
};

/// Gets the maximum number of nodes allowed in the liquidity tree as a function of its depth.
/// Saturates at `u32::MAX`.
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
        2u32.saturating_pow(D::get())
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
/// # Attributes
///
/// - `nodes`: A vector which holds the nodes of the tree. The nodes are ordered by depth (the root
///   is the first element of `nodes`) and from left to right. For example, the right-most
///   grandchild of the root is at index `6`.
/// - `account_to_index`: Maps an account to the node that belongs to it.
/// - `abandoned_nodes`: A vector that contains the indices of abandoned nodes.
///
/// # Generics
///
/// - `T`: The pallet configuration.
/// - `U`: A getter for the maximum depth of the tree. Using a depth larger than `31` will result in
///   undefined behavior.
#[derive(Decode, Encode, Eq, MaxEncodedLen, RuntimeDebugNoBound, TypeInfo)]
#[scale_info(skip_type_params(T, U))]
pub struct LiquidityTree<T, U>
where
    T: Config,
    U: Get<u32>,
{
    pub(crate) nodes: BoundedVec<Node<T>, LiquidityTreeMaxNodes<U>>,
    pub(crate) account_to_index: BoundedBTreeMap<T::AccountId, u32, LiquidityTreeMaxNodes<U>>,
    pub(crate) abandoned_nodes: BoundedVec<u32, LiquidityTreeMaxNodes<U>>,
}

// Boilerplate implementations because Rust is confused by the generic parameter `U`.
impl<T, U> Clone for LiquidityTree<T, U>
where
    T: Config,
    U: Get<u32>,
{
    fn clone(&self) -> Self {
        LiquidityTree {
            nodes: self.nodes.clone(),
            account_to_index: self.account_to_index.clone(),
            abandoned_nodes: self.abandoned_nodes.clone(),
        }
    }
}

// Boilerplate implementations because Rust is confused by the generic parameter `U`.
impl<T, U> PartialEq for LiquidityTree<T, U>
where
    T: Config,
    U: Get<u32>,
{
    fn eq(&self, other: &Self) -> bool {
        self.nodes == other.nodes
            && self.account_to_index == other.account_to_index
            && self.abandoned_nodes == other.abandoned_nodes
    }
}

impl<T, U> LiquidityTree<T, U>
where
    T: Config,
    U: Get<u32>,
{
    /// Create a new liquidity tree.
    ///
    /// Parameter:
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
            .map_err(|_| LiquidityTreeError::AccountNotFound.into_dispatch::<T>())?;
        let mut account_to_index: BoundedBTreeMap<_, _, _> = Default::default();
        account_to_index
            .try_insert(account, 0u32)
            .map_err(|_| LiquidityTreeError::AccountNotFound.into_dispatch::<T>())?;
        let abandoned_nodes = Default::default();
        Ok(LiquidityTree { nodes, account_to_index, abandoned_nodes })
    }
}

// Type for nodes of a liquidity tree.
//
// # Attributes
//
// - `account`: The account that the node belongs to. `None` signifies an abandoned node.
// - `stake`: The stake belonging to the owner.
// - `fees`: The fees owed to the owner.
// - `descendant_stake`: The sum of the stake of all descendant's of this node.
// - `lazy_fees`: The amount of fees to be lazily propagated down the tree.
//
// # Notes
//
// - `descendant_stake` does not contain the stake of `self`.
// - `lazy_fees`, when propagated, is distributed not only to the descendants of `self`, but also to
//   `self`.
#[derive(Clone, Decode, Encode, Eq, MaxEncodedLen, PartialEq, RuntimeDebugNoBound, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub(crate) struct Node<T: Config> {
    pub account: Option<T::AccountId>,
    pub stake: BalanceOf<T>,
    pub fees: BalanceOf<T>,
    pub descendant_stake: BalanceOf<T>,
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
    /// stake. (Strictly speaking, it's not always a leaf!)
    pub(crate) fn is_leaf(&self) -> bool {
        self.descendant_stake == Zero::zero()
    }
}

impl<T, U> LiquiditySharesManager<T> for LiquidityTree<T, U>
where
    T: Config + frame_system::Config,
    T::AccountId: PartialEq<T::AccountId>,
    BalanceOf<T>: AtLeast32BitUnsigned + Copy + Zero,
    U: Get<u32>,
{
    fn join(&mut self, who: &T::AccountId, stake: BalanceOf<T>) -> DispatchResult {
        let index_maybe = self.account_to_index.get(who);
        let index = if let Some(&index) = index_maybe {
            // Pile onto existing account.
            self.propagate_fees_to_node(index)?;
            let node = self.get_node_mut(index)?;
            node.stake = node.stake.checked_add_res(&stake)?;
            index
        } else {
            // Push onto new node.
            let index = match self.peek_next_free_node_index()? {
                NextNode::Abandoned(index) => {
                    self.propagate_fees_to_node(index)?;
                    let node = self.get_node_mut(index)?;
                    node.account = Some(who.clone());
                    node.stake = stake;
                    node.fees = Zero::zero(); // Not necessary, but better safe than sorry.
                    // Don't change `descendant_stake`; we're still maintaining it for abandoned
                    // nodes.
                    node.lazy_fees = Zero::zero();
                    self.abandoned_nodes.pop();
                    index
                }
                NextNode::Leaf => {
                    // Add new leaf. Propagate first so we don't propagate fees to the new leaf.
                    let index = self.nodes.len() as u32;
                    if let Some(parent_index) = self.parent_index(index) {
                        self.propagate_fees_to_node(parent_index)?;
                    }
                    self.nodes
                        .try_push(Node::new(who.clone(), stake))
                        .map_err(|_| LiquidityTreeError::TreeIsFull.into_dispatch::<T>())?;
                    index
                }
                NextNode::None => {
                    return Err::<(), DispatchError>(
                        LiquidityTreeError::TreeIsFull.into_dispatch::<T>(),
                    );
                }
            };
            self.account_to_index
                .try_insert(who.clone(), index)
                .map_err(|_| LiquidityTreeError::TreeIsFull.into_dispatch::<T>())?;
            index
        };
        if let Some(parent_index) = self.parent_index(index) {
            self.update_descendant_stake(parent_index, stake, false)?;
        }
        Ok(())
    }

    fn exit(&mut self, who: &T::AccountId, stake: BalanceOf<T>) -> DispatchResult {
        let index = self.map_account_to_index(who)?;
        self.propagate_fees_to_node(index)?;
        let node = self.get_node_mut(index)?;
        ensure!(node.fees == Zero::zero(), LiquidityTreeError::UnclaimedFees.into_dispatch::<T>());
        node.stake = node
            .stake
            .checked_sub(&stake)
            .ok_or(LiquidityTreeError::InsufficientStake.into_dispatch::<T>())?;
        if node.stake == Zero::zero() {
            node.account = None;
            self.abandoned_nodes
                .try_push(index)
                .map_err(|_| LiquidityTreeError::AccountNotFound.into_dispatch::<T>())?;
            let _ = self.account_to_index.remove(who);
        }
        if let Some(parent_index) = self.parent_index(index) {
            self.update_descendant_stake(parent_index, stake, true)?;
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

/// Type for specifying the next free node.
enum NextNode {
    Abandoned(u32),
    Leaf,
    None,
}

/// A collection of member functions used in the implementation of `LiquiditySharesManager` for
/// `LiquidityTree`.
trait LiquidityTreeHelper<T>
where
    T: Config,
{
    /// Propagate lazy fees from the tree's root to the node at `index`.
    fn propagate_fees_to_node(&mut self, index: u32) -> DispatchResult;

    /// Propagate lazy fees from the node at `index` to its children.
    fn propagate_fees(&mut self, index: u32) -> DispatchResult;

    /// Return the indices of the children of the node at `index`.
    ///
    /// The first (resp. second) component of the array is the left (resp. right) child. If there is
    /// no node at either of these indices, the result is `None`.
    fn children(&self, index: u32) -> Result<[Option<u32>; 2], DispatchError>;

    /// Return the index of a node's parent; `None` if `index` is `0u32`, i.e. the node is root.
    fn parent_index(&self, index: u32) -> Option<u32>;

    /// Return a path from the tree's root to the node at `index`.
    ///
    /// The return value is a vector of the indices of the nodes of the path, starting with the
    /// root.
    fn path_to_node(&self, index: u32) -> Result<Vec<u32>, DispatchError>;

    /// Returns the next free index of the tree.
    ///
    /// If there are abandoned nodes, this will return the nodes in the reverse order in which they
    /// were abandoned.
    fn peek_next_free_node_index(&mut self) -> Result<NextNode, DispatchError>;

    /// Mutate a node's descendant stake.
    ///
    /// # Parameters
    ///
    /// - `index`: The index of the node to modify.
    /// - `delta`: The (absolute) amount by which to modfiy the descendant stake.
    /// - `neg`: The sign of the delta; `true` is the delta is negative.
    fn update_descendant_stake(
        &mut self,
        index: u32,
        delta: BalanceOf<T>,
        neg: bool,
    ) -> DispatchResult;

    /// Mutate each of child of the node at `index` using `mutator`.
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
    fn max_depth(&self) -> u32;

    /// Return the maximum allowed amount of nodes in the tree.
    fn max_node_count(&self) -> u32;
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
        // Temporary storage to ensure that the borrow checker doesn't get upset.
        let descendant_stake = node.descendant_stake;
        if node.is_leaf() {
            self.mutate_node(index, |node| {
                node.fees = node.fees.checked_add_res(&node.lazy_fees)?;
                Ok(())
            })?;
        } else {
            let mut remaining_lazy_fees =
                node.descendant_stake.bdiv(node.total_stake()?)?.bmul(node.lazy_fees)?;
            let fees = node.lazy_fees.checked_sub_res(&remaining_lazy_fees)?;
            self.mutate_node(index, |node| {
                node.fees = node.fees.checked_add_res(&fees)?;
                Ok(())
            })?;
            self.mutate_each_child(index, |child_node| {
                let child_lazy_fees =
                    child_node.total_stake()?.bdiv(descendant_stake)?.bmul(remaining_lazy_fees)?;
                child_node.lazy_fees = child_node.lazy_fees.checked_add_res(&child_lazy_fees)?;
                remaining_lazy_fees = remaining_lazy_fees.checked_sub_res(&child_lazy_fees)?;
                Ok(())
            })?;
        }
        self.mutate_node(index, |node| {
            node.lazy_fees = Zero::zero();
            Ok(())
        })?;
        Ok(())
    }

    fn children(&self, index: u32) -> Result<[Option<u32>; 2], DispatchError> {
        let max_node_count = self.node_count();
        let calculate_child = |child_index: u32| Some(child_index).filter(|&i| i < max_node_count);
        let left_child_index = index.checked_mul_res(&2)?.checked_add_res(&1)?;
        let left_child = calculate_child(left_child_index);
        let right_child_index = left_child_index.checked_add_res(&1)?;
        let right_child = calculate_child(right_child_index);
        Ok([left_child, right_child])
    }

    fn parent_index(&self, index: u32) -> Option<u32> {
        if index == 0 {
            None
        } else {
            // Won't ever fail, always returns `Some(...)`.
            index.checked_sub(1)?.checked_div(2)
        }
    }

    fn path_to_node(&self, mut index: u32) -> Result<Vec<u32>, DispatchError> {
        let mut path = Vec::new();
        let mut iterations = 0;
        let max_iterations = self.max_depth().checked_add_res(&1)?;
        while let Some(parent_index) = self.parent_index(index) {
            if iterations == max_iterations {
                return Err(LiquidityTreeError::MaxIterationsReached.into_dispatch::<T>());
            }
            path.push(index);
            index = parent_index;
            iterations = iterations.checked_add_res(&1)?;
        }
        path.push(0u32); // The tree's root is not considered in the loop above.
        path.reverse(); // The path should be from root to the node
        Ok(path)
    }

    fn peek_next_free_node_index(&mut self) -> Result<NextNode, DispatchError> {
        if let Some(index) = self.abandoned_nodes.last() {
            Ok(NextNode::Abandoned(*index))
        } else if self.node_count() < self.max_node_count() {
            Ok(NextNode::Leaf)
        } else {
            Ok(NextNode::None)
        }
    }

    fn update_descendant_stake(
        &mut self,
        index: u32,
        delta: BalanceOf<T>,
        neg: bool,
    ) -> DispatchResult {
        for &i in self.path_to_node(index)?.iter() {
            let node = self.get_node_mut(i)?;
            if neg {
                node.descendant_stake = node.descendant_stake.checked_sub_res(&delta)?;
            } else {
                node.descendant_stake = node.descendant_stake.checked_add_res(&delta)?;
            }
        }
        Ok(())
    }

    fn mutate_each_child<F>(&mut self, index: u32, mut mutator: F) -> DispatchResult
    where
        F: FnMut(&mut Node<T>) -> DispatchResult,
    {
        let child_indices = self.children(index)?;
        for child_index in child_indices.into_iter().flatten() {
            self.mutate_node(child_index, |node| mutator(node))?;
        }
        Ok(())
    }

    fn node_count(&self) -> u32 {
        self.nodes.len() as u32
    }

    fn get_node(&self, index: u32) -> Result<&Node<T>, DispatchError> {
        self.nodes.get(index as usize).ok_or(LiquidityTreeError::NodeNotFound.into_dispatch::<T>())
    }

    fn get_node_mut(&mut self, index: u32) -> Result<&mut Node<T>, DispatchError> {
        self.nodes
            .get_mut(index as usize)
            .ok_or(LiquidityTreeError::NodeNotFound.into_dispatch::<T>())
    }

    fn map_account_to_index(&self, who: &T::AccountId) -> Result<u32, DispatchError> {
        self.account_to_index
            .get(who)
            .ok_or(LiquidityTreeError::AccountNotFound.into_dispatch::<T>())
            .copied()
    }

    fn mutate_node<F>(&mut self, index: u32, mutator: F) -> DispatchResult
    where
        F: FnOnce(&mut Node<T>) -> DispatchResult,
    {
        let node = self.get_node_mut(index)?;
        mutator(node)
    }

    /// Return the maximum allowed depth of the tree.
    fn max_depth(&self) -> u32 {
        U::get()
    }

    /// Return the maximum allowed amount of nodes in the tree.
    fn max_node_count(&self) -> u32 {
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
    UnclaimedFees,
    /// The liquidity tree is full and can't accept any new nodes.
    TreeIsFull,
    /// This node doesn't hold enough stake.
    InsufficientStake,
    /// A while loop exceeded the expected number of iterations. This is unexpected behavior.
    MaxIterationsReached,
}

impl<T> From<LiquidityTreeError> for Error<T> {
    fn from(error: LiquidityTreeError) -> Error<T> {
        Error::<T>::LiquidityTreeError(error)
    }
}

impl LiquidityTreeError {
    pub(crate) fn into_dispatch<T: Config>(self) -> DispatchError {
        Error::<T>::LiquidityTreeError(self).into()
    }
}