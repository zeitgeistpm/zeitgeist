mod decompressor;
mod hash_tuple;
mod typedefs;

use crate::traits::IdManager;
use core::marker::PhantomData;
use ethnum::U256;
use frame_support::{Blake2_256, StorageHasher};
use hash_tuple::{HashTuple, ToBytes};
use parity_scale_codec::Encode;
use sp_runtime::DispatchError;
use typedefs::Hash;
use zeitgeist_primitives::types::Asset;

pub(crate) struct CryptographicIdManager<MarketId, Hasher>(PhantomData<(MarketId, Hasher)>);

impl<MarketId, Hasher> IdManager for CryptographicIdManager<MarketId, Hasher>
where
    MarketId: ToBytes + Encode,
    Hasher: HashTuple
{
    type Asset = Asset<MarketId>;
    type Id = Hash;
    type MarketId = MarketId;

    fn get_collection_id(
        parent_collection_id: Option<Self::Id>,
        market_id: Self::MarketId,
        index_set: Vec<bool>,
        force_max_work: bool,
    ) -> Option<Self::Id> {
        let input = (market_id, index_set);
        let hash = Hasher::hash_tuple(input);
        decompressor::get_collection_id(hash, parent_collection_id, force_max_work)
    }

    fn get_position_id(collateral: Self::Asset, collection_id: Self::Id) -> Option<Self::Id> {
        let input = (collateral, collection_id);
        Some(Hasher::hash_tuple(input))
    }
}
