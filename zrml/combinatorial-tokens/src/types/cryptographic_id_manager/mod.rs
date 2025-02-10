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

mod decompressor;
mod hash_tuple;

use super::CollectionIdError;
use crate::traits::CombinatorialIdManager;
use alloc::vec::Vec;
use core::marker::PhantomData;
use hash_tuple::{HashTuple, ToBytes};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use zeitgeist_primitives::{
    traits::CombinatorialTokensFuel,
    types::{Asset, CombinatorialId},
};

#[derive(Clone, Debug, Decode, Encode, Eq, MaxEncodedLen, PartialEq, TypeInfo)]
pub struct Fuel {
    /// The maximum number of iterations to perform in the main loop of `get_collection_id`.
    total: u32,

    /// Perform `self.total` of iterations in the main loop of `get_collection_id`. Useful for
    /// benchmarking purposes and should probably not be used in production.
    consume_all: bool,
}

impl Fuel {
    pub fn new(total: u32, consume_all: bool) -> Self {
        Fuel { total, consume_all }
    }

    pub fn consume_all(&self) -> bool {
        self.consume_all
    }
}

impl CombinatorialTokensFuel for Fuel {
    fn from_total(total: u32) -> Fuel {
        Fuel { total, consume_all: true }
    }

    fn total(&self) -> u32 {
        self.total
    }
}

pub struct CryptographicIdManager<MarketId, Hasher>(PhantomData<(MarketId, Hasher)>);

impl<MarketId, Hasher> CombinatorialIdManager for CryptographicIdManager<MarketId, Hasher>
where
    MarketId: ToBytes + Encode,
    Hasher: HashTuple,
{
    type Asset = Asset<MarketId>;
    type CombinatorialId = CombinatorialId;
    type MarketId = MarketId;
    type Fuel = Fuel;

    fn get_collection_id(
        parent_collection_id: Option<Self::CombinatorialId>,
        market_id: Self::MarketId,
        index_set: Vec<bool>,
        fuel: Self::Fuel,
    ) -> Result<Self::CombinatorialId, CollectionIdError> {
        let input = (market_id, index_set);
        let hash = Hasher::hash_tuple(input);

        decompressor::get_collection_id(hash, parent_collection_id, fuel)
    }

    fn get_position_id(
        collateral: Self::Asset,
        collection_id: Self::CombinatorialId,
    ) -> Self::CombinatorialId {
        let input = (collateral, collection_id);

        Hasher::hash_tuple(input)
    }
}
