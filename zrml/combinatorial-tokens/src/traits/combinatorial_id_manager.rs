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

pub trait CombinatorialIdManager {
    type Asset;
    type MarketId;
    type CombinatorialId;

    // TODO Replace `Vec<bool>` with a more effective bit mask type.
    fn get_collection_id(
        parent_collection_id: Option<Self::CombinatorialId>,
        market_id: Self::MarketId,
        index_set: Vec<bool>,
        force_max_work: bool,
    ) -> Option<Self::CombinatorialId>;

    fn get_position_id(
        collateral: Self::Asset,
        collection_id: Self::CombinatorialId,
    ) -> Self::CombinatorialId;
}
