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

use crate::traits::CombinatorialIdManager;
use alloc::vec::Vec;
use core::marker::PhantomData;
use hash_tuple::{HashTuple, ToBytes};
use parity_scale_codec::Encode;
use zeitgeist_primitives::types::{Asset, CombinatorialId};

pub struct CryptographicIdManager<MarketId, Hasher>(PhantomData<(MarketId, Hasher)>);

impl<MarketId, Hasher> CombinatorialIdManager for CryptographicIdManager<MarketId, Hasher>
where
    MarketId: ToBytes + Encode,
    Hasher: HashTuple,
{
    type Asset = Asset<MarketId>;
    type CombinatorialId = CombinatorialId;
    type MarketId = MarketId;

    fn get_collection_id(
        parent_collection_id: Option<Self::CombinatorialId>,
        market_id: Self::MarketId,
        index_set: Vec<bool>,
        force_max_work: bool,
    ) -> Option<Self::CombinatorialId> {
        let input = (market_id, index_set);
        let hash = Hasher::hash_tuple(input);
        decompressor::get_collection_id(hash, parent_collection_id, force_max_work)
    }

    fn get_position_id(
        collateral: Self::Asset,
        collection_id: Self::CombinatorialId,
    ) -> Self::CombinatorialId {
        let input = (collateral, collection_id);
        Hasher::hash_tuple(input)
    }
}
