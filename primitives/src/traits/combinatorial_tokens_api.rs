// Copyright 2024-2025 Forecasting Technologies LTD.
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

use crate::{traits::CombinatorialTokensFuel, types::SplitPositionDispatchInfo};
use alloc::vec::Vec;
use core::fmt::Debug;
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::DispatchError;

/// Trait that can be used to expose the internal functionality of zrml-combinatorial-tokens to
/// other pallets.
pub trait CombinatorialTokensApi {
    type AccountId;
    type Balance;
    type CombinatorialId;
    type MarketId;
    type Fuel: Clone + CombinatorialTokensFuel + Debug + Decode + Encode + Eq + TypeInfo;

    fn split_position(
        who: Self::AccountId,
        parent_collection_id: Option<Self::CombinatorialId>,
        market_id: Self::MarketId,
        partition: Vec<Vec<bool>>,
        amount: Self::Balance,
        force_max_work: Self::Fuel,
    ) -> Result<SplitPositionDispatchInfo<Self::CombinatorialId, Self::MarketId>, DispatchError>;
}
