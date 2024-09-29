mod decompressor;
mod hash_tuple;
mod typedefs;

use crate::traits::IdManager;
use core::marker::PhantomData;
use ethnum::U256;
use frame_support::{Blake2_256, StorageHasher};
use hash_tuple::{hash_tuple, ToBytes};
use parity_scale_codec::Encode;
use sp_runtime::DispatchError;
use typedefs::Hash;
use zeitgeist_primitives::types::Asset;

pub(crate) struct CryptographicIdManager<MarketId>(PhantomData<MarketId>);

impl<MarketId> IdManager for CryptographicIdManager<MarketId>
where
    MarketId: ToBytes + Encode,
{
    type Asset = Asset<MarketId>;
    type Id = Hash;
    type MarketId = MarketId;

    fn get_collection_id(
        parent_collection_id: Option<Self::Id>,
        market_id: Self::MarketId,
        index_set: Vec<bool>,
    ) -> Option<Self::Id> {
        let input = (market_id, index_set);
        let hash = hash_tuple(input);
        decompressor::get_collection_id(hash, parent_collection_id)
    }

    fn get_position_id(collateral: Self::Asset, collection_id: Self::Id) -> Option<Self::Id> {
        let input = (collateral, collection_id);
        Some(hash_tuple(input))
    }
}
