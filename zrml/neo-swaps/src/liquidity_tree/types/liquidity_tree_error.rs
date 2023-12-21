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

use crate::{Config, Error};
use frame_support::{PalletError, RuntimeDebugNoBound};
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::DispatchError;

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
