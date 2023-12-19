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

use core::marker::PhantomData;
use sp_runtime::traits::Get;

/// Gets the maximum number of nodes allowed in the liquidity tree as a function of its depth.
/// Saturates at `u32::MAX`, but will warn about this in DEBUG.
///
/// # Generics
///
/// - `D`: A getter for the depth of the tree.
pub(crate) struct LiquidityTreeMaxNodes<D>(PhantomData<D>);

impl<D> Get<u32> for LiquidityTreeMaxNodes<D>
where
    D: Get<u32>,
{
    fn get() -> u32 {
        debug_assert!(D::get() < 31, "LiquidityTreeMaxNodes::get(): Integer overflow");
        2u32.saturating_pow(D::get() + 1).saturating_sub(1)
    }
}
