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
use alloc::collections::BTreeMap;
use frame_support::{ensure, PalletError};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{
    traits::{AtLeast32BitUnsigned, CheckedSub, Zero},
    DispatchError, DispatchResult, RuntimeDebug,
};
use zeitgeist_primitives::math::checked_ops_res::{
    CheckedAddRes, CheckedDivRes, CheckedMulRes, CheckedPowRes, CheckedSubRes,
};

#[derive(Decode, Encode, Eq, PartialEq, PalletError, RuntimeDebug, TypeInfo)]
pub enum LiquidityTreeError {
    AccountNotFound,
    NodeNotFound,
    UnclaimedFees,
    TreeIsFull,
    InsufficientStake,
    MaxIterationsReached,
}

impl<T> From<LiquidityTreeError> for Error<T> {
    fn from(error: LiquidityTreeError) -> Error<T> {
        Error::<T>::LiquidityTreeError(error)
    }
}

impl LiquidityTreeError {
    fn to_dispatch<T: Config>(self) -> DispatchError {
        Error::<T>::LiquidityTreeError(self).into()
    }
}

#[derive(TypeInfo, MaxEncodedLen, Clone, Encode, Eq, Decode, PartialEq, RuntimeDebug)]
#[scale_info(skip_type_params(T))]
pub struct Node<T: Config> {
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
    pub(crate) fn new(account: T::AccountId, stake: BalanceOf<T>) -> Node<T> {
        Node {
            account: Some(account),
            stake,
            fees: 0u8.into(),
            descendant_stake: 0u8.into(),
            lazy_fees: 0u8.into(),
        }
    }

    pub(crate) fn total_stake(&self) -> Result<BalanceOf<T>, DispatchError> {
        self.stake.checked_add_res(&self.descendant_stake)
    }

    pub(crate) fn is_leaf(&self) -> bool {
        self.descendant_stake == Zero::zero()
    }
}

pub struct LiquidityTree<T: Config> {
    max_depth: usize,
    nodes: Vec<Node<T>>,
    account_to_index: BTreeMap<T::AccountId, usize>,
    abandoned_nodes: Vec<usize>,
}

impl<T> LiquidityTree<T>
where
    T: Config,
{
    pub(crate) fn new(
        max_depth: usize,
        account: T::AccountId,
        stake: BalanceOf<T>,
    ) -> LiquidityTree<T> {
        let root = Node::new(account.clone(), stake);
        let mut account_to_index = BTreeMap::new();
        account_to_index.insert(account, 0usize);
        LiquidityTree { max_depth, nodes: vec![root], account_to_index, abandoned_nodes: vec![] }
    }

    pub(crate) fn max_node_count(&self) -> Result<usize, DispatchError> {
        2usize.checked_pow_res(self.max_depth)
    }
}

impl<T> LiquiditySharesManager<T> for LiquidityTree<T>
where
    T: Config + frame_system::Config,
    T::AccountId: PartialEq<T::AccountId>,
    BalanceOf<T>: AtLeast32BitUnsigned + Copy + Zero,
{
    fn join(&mut self, who: &T::AccountId, stake: BalanceOf<T>) -> DispatchResult {
        // TODO Handle root case?
        let index_maybe = self.account_to_index.get(who);
        let index = if let Some(&index) = index_maybe {
            // Pile onto existing account.
            self.propagate_fees_to_node(index)?;
            let node = self.get_node_mut(index)?;
            node.stake = node.stake.checked_add_res(&stake)?;
            index
        } else {
            // Push onto new account.
            // FIXME Beware! This violates verify first, write last!
            let index = match self.peek_next_free_node_index()? {
                NextNode::Abandoned(index) => {
                    self.propagate_fees_to_node(index)?;
                    let node = self.get_node_mut(index)?;
                    node.account = Some(who.clone());
                    node.stake = stake;
                    node.fees = Zero::zero(); // Not necessary, but better safe than sorry.
                    // Don't change `descendant_stake`; we're still maintaining it for abandoned nodes.
                    node.lazy_fees = Zero::zero();
                    self.abandoned_nodes.pop();
                    index
                }
                NextNode::Leaf => {
                    // Add new leaf. Propagate first so we don't propagate fees to the new leaf.
                    let index = self.nodes.len();
                    if let Some(parent_index) = self.parent_index(index) {
                        self.update_descendant_stake(parent_index, stake, false)?;
                    }
                    self.nodes.push(Node::new(who.clone(), stake));
                    index
                }
                NextNode::None => {
                    return Err::<(), DispatchError>(
                        Into::<Error<T>>::into(LiquidityTreeError::TreeIsFull).into(),
                    );
                }
            };
            self.account_to_index.insert(who.clone(), index);
            index
        };
        if let Some(parent_index) = self.parent_index(index) {
            self.update_descendant_stake(parent_index, stake, false)?;
        }
        Ok(())
    }

    fn exit(&mut self, who: &T::AccountId, stake: BalanceOf<T>) -> DispatchResult {
        // TODO Handle root case?
        let index = self.map_account_to_index(who)?;
        self.propagate_fees_to_node(index)?;
        let node = self.get_node_mut(index)?;
        ensure!(node.fees == Zero::zero(), LiquidityTreeError::UnclaimedFees.to_dispatch::<T>());
        node.stake = node
            .stake
            .checked_sub(&stake)
            .ok_or(LiquidityTreeError::InsufficientStake.to_dispatch::<T>())?;
        if node.stake == Zero::zero() {
            node.account = None;
            self.abandoned_nodes.push(index);
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
        let root = self.get_node_mut(0usize)?;
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
        let root = self.get_node(0usize)?;
        root.total_stake()
    }
}

enum NextNode {
    Abandoned(usize),
    Leaf,
    None,
}

trait LiquidityTreeHelper<T>
where
    T: Config,
{
    fn propagate_fees_to_node(&mut self, index: usize) -> DispatchResult;

    fn propagate_fees(&mut self, index: usize) -> DispatchResult;

    fn children(&self, index: usize) -> Result<[Option<usize>; 2], DispatchError>;

    // None if `index` is `zero`, i.e. the node is root.
    fn parent_index(&self, index: usize) -> Option<usize>;

    fn path_to_node(&self, index: usize) -> Result<Vec<usize>, DispatchError>;

    fn peek_next_free_node_index(&mut self) -> Result<NextNode, DispatchError>;

    fn update_descendant_stake(
        &mut self,
        index: usize,
        delta: BalanceOf<T>,
        neg: bool,
    ) -> DispatchResult;

    fn with_each_child<F>(&mut self, index: usize, mutator: F) -> DispatchResult
    where
        F: FnMut(&mut Node<T>) -> DispatchResult;

    fn node_count(&self) -> usize;

    fn get_node(&self, index: usize) -> Result<&Node<T>, DispatchError>;

    fn get_node_mut(&mut self, index: usize) -> Result<&mut Node<T>, DispatchError>;

    fn map_account_to_index(&self, account: &T::AccountId) -> Result<usize, DispatchError>;
}

impl<T> LiquidityTreeHelper<T> for LiquidityTree<T>
where
    T: Config,
{
    fn propagate_fees_to_node(&mut self, index: usize) -> DispatchResult {
        let path = self.path_to_node(index)?;
        for i in path {
            self.propagate_fees(i)?;
        }
        Ok(())
    }
    fn propagate_fees(&mut self, index: usize) -> DispatchResult {
        let node = self.get_node(index)?;
        // TODO Shouldn't this be descendant_stake?
        if node.total_stake()? == Zero::zero() {
            return Ok(()); // Don't propagate if there are no LPs under this node.
        }

        // We can use the immutable data from node here safely.
        let total_stake = node.total_stake()?;
        let is_leaf = node.is_leaf();
        let descendant_stake = node.descendant_stake;
        let lazy_fees = node.lazy_fees;

        // Only borrow mutably to update the current node.
        let node_mut = self.get_node_mut(index)?;
        node_mut.lazy_fees = Zero::zero();
        if is_leaf {
            node_mut.fees = node_mut.fees.checked_add_res(&lazy_fees)?;
        } else {
            let mut remaining_lazy_fees =
                descendant_stake.checked_div_res(&total_stake)?.checked_mul_res(&lazy_fees)?;
            let fees = lazy_fees.checked_sub_res(&remaining_lazy_fees)?;
            node_mut.fees = node_mut.fees.checked_add_res(&fees)?;

            // Now loop over child nodes to update them.
            self.with_each_child(index, |child_node| {
                // Mutably borrow each child node inside the loop to update it.
                let child_total_stake = child_node.total_stake()?;
                let child_lazy_fees = child_total_stake
                    .checked_div_res(&descendant_stake)?
                    .checked_mul_res(&remaining_lazy_fees)?;
                child_node.lazy_fees = child_node.lazy_fees.checked_add_res(&child_lazy_fees)?;
                remaining_lazy_fees = remaining_lazy_fees.checked_sub_res(&child_lazy_fees)?;
                Ok(())
            })?;
        }

        Ok(())
    }

    fn children(&self, index: usize) -> Result<[Option<usize>; 2], DispatchError> {
        let max_node_count = self.max_node_count()?;
        let calculate_child =
            |child_index: usize| Some(child_index).filter(|&i| i < max_node_count);
        let left_child_index = index.checked_mul_res(&2)?.checked_add_res(&1)?;
        let left_child = calculate_child(left_child_index);
        let right_child_index = left_child_index.checked_add_res(&1)?;
        let right_child = calculate_child(right_child_index);
        Ok([left_child, right_child])
    }

    fn parent_index(&self, index: usize) -> Option<usize> {
        if index == 0 {
            None
        } else {
            // Won't ever fail, always returns `Some(...)`.
            index.checked_sub(1)?.checked_div(2)
        }
    }

    fn path_to_node(&self, mut index: usize) -> Result<Vec<usize>, DispatchError> {
        let mut path = Vec::new();
        while let Some(parent_index) = self.parent_index(index) {
            // TODO max iterations
            path.push(index);
            index = parent_index;
        }
        path.push(0); // Add the root of the tree (`parent_index` returns `None` for root)
        path.reverse(); // The path should be from root to the node
        Ok(path)
    }

    fn peek_next_free_node_index(&mut self) -> Result<NextNode, DispatchError> {
        if let Some(index) = self.abandoned_nodes.last() {
            Ok(NextNode::Abandoned(*index))
        } else if self.nodes.len() < self.max_node_count()? {
            Ok(NextNode::Leaf)
        } else {
            Ok(NextNode::None)
        }
    }

    fn update_descendant_stake(
        &mut self,
        index: usize,
        delta: BalanceOf<T>,
        neg: bool,
    ) -> DispatchResult {
        let mut iterations = 0;
        while let Some(parent_index) = self.parent_index(index) {
            let node = self
                .nodes
                .get_mut(index)
                .ok_or::<Error<T>>(LiquidityTreeError::NodeNotFound.into())?;
            if neg {
                node.descendant_stake = node.descendant_stake.checked_sub_res(&delta)?;
            } else {
                node.descendant_stake = node.descendant_stake.checked_add_res(&delta)?;
            }
            if iterations == self.max_depth {
                return Err(LiquidityTreeError::MaxIterationsReached.to_dispatch::<T>());
                // TODO Error
            }
        }
        Ok(())
    }

    fn with_each_child<F>(&mut self, index: usize, mut mutator: F) -> DispatchResult
    where
        F: FnMut(&mut Node<T>) -> DispatchResult,
    {
        let children_indices = self.children(index)?;
        for child_option in children_indices {
            if let Some(child_index) = child_option {
                let child_node = self
                    .nodes
                    .get_mut(child_index)
                    .ok_or::<Error<T>>(LiquidityTreeError::NodeNotFound.into())?;
                mutator(child_node)?;
            }
        }
        Ok(())
    }

    fn node_count(&self) -> usize {
        self.nodes.len()
    }

    fn get_node(&self, index: usize) -> Result<&Node<T>, DispatchError> {
        self.nodes.get(index).ok_or(LiquidityTreeError::NodeNotFound.to_dispatch::<T>())
    }

    fn get_node_mut(&mut self, index: usize) -> Result<&mut Node<T>, DispatchError> {
        self.nodes.get_mut(index).ok_or(LiquidityTreeError::NodeNotFound.to_dispatch::<T>())
    }

    fn map_account_to_index(&self, who: &T::AccountId) -> Result<usize, DispatchError> {
        self.account_to_index
            .get(who)
            .ok_or(LiquidityTreeError::AccountNotFound.to_dispatch::<T>())
            .copied()
    }
}
