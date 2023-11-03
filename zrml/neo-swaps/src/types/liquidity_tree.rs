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
use frame_support::ensure;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{
    traits::{AtLeast32BitUnsigned, CheckedAdd, CheckedSub, Zero},
    DispatchError, DispatchResult, RuntimeDebug,
};

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
        Node { account, stake, fees: 0, descendant_stake: 0, lazy_fees: 0 }
    }

    pub(crate) fn total_stake(self) -> Result<BalanceOf<T>, DispatchError> {
        self.stake.checked_add_res(self.descendant_stake)?
    }

    pub(crate) fn is_leaf(self) -> bool {
        self.descendant_stake == Zero::zero()
    }
}

pub struct LiquidityTree<T: Config> {
    max_depth: usize,
    nodes: Vec<Node>,
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
        let root = Node::new(account, stake);
        let account_to_index = BTreeMap::new();
        account_to_index.insert(account, 0usize);
        LiquidityTree { max_depth, nodes: vec![root], account_to_index, abandoned_nodes: vec![] }
    }

    pub(crate) fn max_node_count(self) -> DispatchResult<usize, DispatchError> {
        2usize.checked_pow(self.max_depth as u32).ok_or(TODO)
    }
}

impl<T> LiquiditySharesManager<T> for LiquidityTree<T>
where
    T: Config + frame_system::Config,
    T::AccountId: PartialEq<T::AccountId>,
    BalanceOf<T>: AtLeast32BitUnsigned + Copy + Zero,
{
    fn join(&mut self, who: &T::AccountId, stake: BalanceOf<T>) -> DispatchResult {
        let index_maybe = self.account_to_index.get(who);
        let index = if let Some(index) = index_maybe {
            // Pile onto existing account.
            self.propagate_fees_to_node(index)?;
            let node = self.nodes.get_mut(index);
            node.stake = node.stake.checked_add_res(stake)?;
            index
        } else {
            // Push onto new account.
            // FIXME Beware! This violates verify first, write last!
            // TODO: Return an enum here which determines whether to take an abandoned node.
            let index = self.peek_next_free_node_index().ok_or(LiquidityTreeError::TreeIsFull)?;
            if index < self.node_count() {
                // Reassign abandoned node.
                self.propagate_fees_to_node(index)?;
                let node = self.nodes.get_mut(index);
                node.account = who;
                node.stake = stake;
                node.fees = 0; // Not necessary, but better safe than sorry.
                // Don't change `descendant_stake`; we're still maintaining it for abandoned nodes.
                node.lazy_fees = 0;
                self.abandoned_nodes
            } else {
                // Add new leaf. Propagate first so we don't propagate fees to the new leaf.
                self.propagate_fees_to_node(self.parent_index())?;
                self.nodes.append(Node::new(who, stake));
            }
            self.account_to_index.insert(index, node);
            index
        };
        self.update_descendant_stake(self.parent_index(index), stake, false)?;
        Ok(())
    }

    fn exit(&mut self, who: &T::AccountId, stake: BalanceOf<T>) -> DispatchResult {
        let index = self.account_to_index.get(who).ok_or(LiquidityTreeError::AccountNotFound)?;
        self.propagate_fees_to_node(index)?;
        let node = self.nodes.get_mut(index).ok_or(LiquidityTreeError::NodeNotFound)?;
        ensure!(node.fees == Zero::zero(), LiquidityTreeError::UnclaimedFees);
        node.stake = node.stake.checked_sub(shares).ok_or(LiquidityTreeError::InsufficientStake)?;
        if node.stake == Zero::zero() {
            node.account = None;
            self.abandoned_nodes.push(index);
            self.account_to_index.pop(index);
        }
        self.update_descendant_stake(self.parent_index(index), stake, true)?;
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
        let root = self.nodes.get_mut(&0usize);
        root.lazy_fees = root_lazy_fees.checked_add_res(amount)
    }

    fn withdraw_fees(&mut self, who: &T::AccountId) -> Result<BalanceOf<T>, DispatchError> {
        // TODO Abstract this into `account_to_index` function.
        let index = self.account_to_index.get(who).ok_or(LiquidityTreeError::AccountNotFound)?;
        let mut node = self.nodes.get_mut(index).ok_or(LiquidityTreeError::NodeNotFound)?;
        self.propagate_fees_to_node(index)?;
        fees = node.fees;
        node.fees = Zero::zero();
        Ok(fees)
    }

    fn shares_of(&self, who: &T::AccountId) -> Result<BalanceOf<T>, DispatchError> {
        let index = self.account_to_index.get(who).ok_or(LiquidityTreeError::AccountNotFound)?;
        let node = self.nodes.get(index).ok_or(LiquidityTreeError::NodeNotFound)?;
        Ok(node.stake)
    }

    fn total_shares(&self) -> Result<BalanceOf<T>, DispatchError> {
        let root = self.nodes.get(0usize).ok_or(LiquidityTreeError::NodeNotFound)?;
        Ok(root.total_stake())
    }
}

trait LiquidityTreeHelper<T> {
    fn propagate_fees_to_node(self, index: usize) -> DispatchResult;

    fn propagate_fees(self, account: T::AccountId) -> DispatchResult;

    fn children(self, index: usize) -> DispatchResult<[Option<usize>; 2], DispatchError>;

    fn parent_index(&self, index: usize) -> usize;

    fn path_to_node(&self, index: usize) -> Result<Vec<usize>, DispatchError>;

    fn peek_next_free_node_index(&mut self) -> Option<usize>;

    fn take_next_free_node_index(&mut self) -> Option<usize>;

    fn update_descendant_stake(
        self,
        index: usize,
        delta: BalanceOf<T>,
        neg: bool,
    ) -> DispatchResult;

    fn with_each_child<F>(&mut self, index: usize, mut mutator: F) -> DispatchResult
    where
        F: FnMut(&mut Node) -> DispatchResult;

    fn node_count(&self) -> usize;
}

impl<T> LiquidityTreeHelper<T> for LiquidityTree<T>
where
    T: Config,
{
    fn propagate_fees_to_node(&mut self, index: usize) -> DispatchResult {
        let path = self._get_path_to_node(index)?;
        for i in path {
            self.propagate_fees(i)?;
        }
        Ok(())
    }

    fn propagate_fees(&mut self, index: usize) -> DispatchResult {
        // TODO Abstract this into `get_mut_node` since it's used so often.
        let mut node = self.nodes.get_mut(index).ok_or(LiquidityTreeError::NodeNotFound)?;
        if node.total_stake == Zero::zero() {
            return; // Don't propagate if there are no LPs under this node.
        }
        if node.is_leaf() {
            node.fees = node.fees.checked_add_res(node.lazy_fees)?;
        } else {
            let mut remaining = node
                .descendant_stake
                .checked_div_res(node.total_stake)
                .checked_mul_res(node.lazy_fees);
            let fees = node.lazy_fees.checked_sub_res(remaining)?;
            node.fees = node.fees.checked_add_res(fees)?;
            self.with_each_child(index, |child_node| {
                let child_lazy_fees = child_node
                    .total_stake
                    .checked_div_res(&node.descendant_stake)?
                    .checked_mul_res(&remaining_lazy_fees)?;
                child_node.lazy_fees = child_node.lazy_fees.checked_add_res(&child_lazy_fees)?;
                remaining = remaining.checked_sub_res(&child_lazy_fees)?;
                Ok(())
            })?;
        }
        node.lazy_fees = Zero::zero();
        Ok(())
    }

    fn children(
        &self,
        index: usize,
    ) -> DispatchResult<(Option<usize>, Option<usize>), DispatchError> {
        let max_node_count = self.max_node_count()?;
        let calculate_child =
            |child_index: usize| Some(child_index).filter(|&i| i < max_node_count);
        let left_child_index = index.checked_mul_res(2)?.checked_add_res(1)?;
        let left_child = calculate_child(left_child_index);
        let right_child_index = left_child_index.checked_add_res(1)?;
        let right_child = calculate_child(right_child_index);
        Ok((left_child, right_child))
    }

    fn parent_index(&self, index: usize) -> Option<usize> {
        if index == 0 {
            None
        } else {
            // Won't ever fail, always returns `Some(...)`.
            index.checked_sub(&1)?.checked_div(&2)
        }
    }

    fn path_to_node(&self, index: usize) -> Result<Vec<usize>, DispatchError> {
        let mut path = Vec::new();
        while let Some(parent_index) = self.parent_index(node_index) {
            // TODO max iterations
            path.push(node_index);
            node_index = parent_index;
        }
        path.push(0); // Add the root of the tree (`parent_index` returns `None` for root)
        path.reverse(); // The path should be from root to the node
        path
    }

    fn peek_next_free_node_index(&mut self) -> abc {
        if let Some(index) = self.abandoned_nodes.keys().next() {
            Some(index)
        } else if self.nodes.len() < self.max_node_count()? {
            Some(self.nodes.len())
        } else {
            None
        }
    }

    fn update_descendant_stake(
        &mut self,
        index: usize,
        delta: BalanceOf<T>,
        neg: bool,
    ) -> DispatchResult<(), DispatchError> {
        let mut iterations = 0;
        while let Some(parent_index) = self.parent_index(index) {
            let node = self.nodes.get_mut(index).ok_or(LiquidityTreeError::NodeNotFound)?;
            if neg {
                node.descendant_stake.checked_sub_res(delta)?
            } else {
                node.descendant_stake.checked_add_res(delta)?
            }
            if iterations == self.max_depth()? {
                // TODO Error
            }
        }
        Ok(())
    }

    fn with_each_child<F>(&mut self, index: usize, mut mutator: F) -> DispatchResult
    where
        F: FnMut(&mut Node) -> DispatchResult,
    {
        let children_indices = self.children(index)?;
        for child_option in children_indices {
            if let Some(child_index) = child_option {
                let child_node =
                    self.nodes.get_mut(child_index).ok_or(LiquidityTreeError::NodeNotFound)?;
                mutator(child_node)?;
            }
        }
        Ok(())
    }

    fn node_count(&self) -> usize {
        self.nodes.len()
    }
}
