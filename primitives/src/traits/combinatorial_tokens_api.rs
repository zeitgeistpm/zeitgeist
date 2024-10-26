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
use frame_support::pallet_prelude::DispatchResultWithPostInfo;
use sp_runtime::DispatchError;

pub trait CombinatorialTokensApi {
    type AccountId;
    type Balance;
    type CombinatorialId;
    type MarketId;

    fn combinatorial_position(
        parent_collection_id: Option<Self::CombinatorialId>,
        market_id: Self::MarketId,
        partition: Vec<Vec<bool>>,
        force_max_work: bool,
    ) -> Result<Asset<Self::MarketId>, DispatchError>;

    fn split_position(
        who: Self::AccountId,
        parent_collection_id: Option<Self::CombinatorialId>,
        market_id: Self::MarketId,
        partition: Vec<Vec<bool>>,
        amount: Self::Balance,
        force_max_work: bool,
    ) -> DispatchResultWithPostInfo;

    fn merge_position(
        who: Self::AccountId,
        parent_collection_id: Option<Self::CombinatorialId>,
        market_id: Self::MarketId,
        partition: Vec<Vec<bool>>,
        amount: Self::Balance,
        force_max_work: bool,
    ) -> DispatchResultWithPostInfo;

    fn redeem_position(
        who: Self::AccountId,
        parent_collection_id: Option<Self::CombinatorialId>,
        market_id: Self::MarketId,
        index_set: Vec<bool>,
        force_max_work: bool,
    ) -> DispatchResultWithPostInfo;
}
