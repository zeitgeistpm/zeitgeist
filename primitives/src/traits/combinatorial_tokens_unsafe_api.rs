// Copyright 2024 Forecasting Technologies LTD.
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

use crate::types::Asset;
use alloc::vec::Vec;
use sp_runtime::DispatchResult;

// Very fast and very unsafe API for splitting and merging combinatorial tokens. Calling the exposed
// functions with a bad `assets` argument can break the reserve.
pub trait CombinatorialTokensUnsafeApi {
    type AccountId;
    type Balance;
    type MarketId;

    fn split_position_unsafe(
        who: Self::AccountId,
        collateral: Asset<Self::MarketId>,
        assets: Vec<Asset<Self::MarketId>>,
        amount: Self::Balance,
    ) -> DispatchResult;

    fn merge_position_unsafe(
        who: Self::AccountId,
        collateral: Asset<Self::MarketId>,
        assets: Vec<Asset<Self::MarketId>>,
        amount: Self::Balance,
    ) -> DispatchResult;
}
