// Copyright 2025 Forecasting Technologies LTD.
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
//
// This file incorporates work licensed under the GNU Lesser General
// Public License 3.0 but published without copyright notice by Gnosis
// (<https://gnosis.io>, info@gnosis.io) in the
// conditional-tokens-contracts repository
// <https://github.com/gnosis/conditional-tokens-contracts>,
// and has been relicensed under GPL-3.0-or-later in this repository.

use crate::types::CollectionIdError;
use alloc::vec::Vec;

/// Handles calculations of combinatorial IDs.
pub trait CombinatorialIdManager {
    type Asset;
    type MarketId;
    type CombinatorialId;
    type Fuel;

    /// Calculate the collection ID obtained when splitting `parent_collection_id` over the market
    /// given by `market_id` and the `index_set`.
    ///
    /// The `fuel` parameter specifies how much work the function will do and can be used for
    /// benchmarking purposes.
    fn get_collection_id(
        parent_collection_id: Option<Self::CombinatorialId>,
        market_id: Self::MarketId,
        index_set: Vec<bool>,
        fuel: Self::Fuel,
    ) -> Result<Self::CombinatorialId, CollectionIdError>;

    /// Calculate the position ID belonging to the `collection_id` combined with `collateral` as
    /// collateral.
    fn get_position_id(
        collateral: Self::Asset,
        collection_id: Self::CombinatorialId,
    ) -> Self::CombinatorialId;
}
