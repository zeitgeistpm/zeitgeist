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

use crate::{BalanceOf, Config};
use frame_support::RuntimeDebugNoBound;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{traits::Zero, DispatchError};
use zeitgeist_primitives::math::checked_ops_res::CheckedAddRes;

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
    pub(crate) account: Option<T::AccountId>,
    /// The stake belonging to the owner.
    pub(crate) stake: BalanceOf<T>,
    /// The fees owed to the owner.
    pub(crate) fees: BalanceOf<T>,
    /// The sum of the stake of all descendants of this node.
    pub(crate) descendant_stake: BalanceOf<T>,
    /// The amount of fees to be lazily propagated down the tree.
    pub(crate) lazy_fees: BalanceOf<T>,
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
