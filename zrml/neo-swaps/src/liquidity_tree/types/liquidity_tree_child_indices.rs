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

use sp_runtime::DispatchError;

/// Structure for managing children in a liquidity tree.
pub(crate) struct LiquidityTreeChildIndices {
    /// Left-hand side child; `None` if there's no left-hand side child (the node is either empty or
    /// the parent is a leaf).
    pub(crate) lhs: Option<u32>,
    /// Right-hand side child; `None` if there's no right-hand side child (the node is either empty
    /// of the parent is a leaf).
    pub(crate) rhs: Option<u32>,
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
