mod decompressor;
mod hash_tuple;

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
