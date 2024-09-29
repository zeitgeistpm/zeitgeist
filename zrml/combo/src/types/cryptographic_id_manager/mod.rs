mod alt_bn128;

use crate::traits::IdManager;
use alt_bn128::Hash;
use core::marker::PhantomData;
use ethnum::U256;
use frame_support::{Blake2_256, StorageHasher};
use parity_scale_codec::Encode;
use sp_runtime::DispatchError;
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
        alt_bn128::get_collection_id(hash, parent_collection_id)
    }

    fn get_position_id(collateral: Self::Asset, collection_id: Self::Id) -> Option<Self::Id> {
        let input = (collateral, collection_id);
        Some(hash_tuple(input))
    }
}

fn hash_tuple<T1, T2>(tuple: (T1, T2)) -> Hash
where
    T1: ToBytes,
    T2: ToBytes,
{
    let mut bytes = Vec::new();

    bytes.extend_from_slice(&tuple.0.to_bytes());
    bytes.extend_from_slice(&tuple.1.to_bytes());

    Blake2_256::hash(&bytes)
}

// TODO Move into traits!
trait ToBytes {
    /// `None` indicates failure.
    fn to_bytes(&self) -> Vec<u8>;
}

// TODO Use macros for this
impl ToBytes for u32 {
    fn to_bytes(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
}

impl ToBytes for u128 {
    fn to_bytes(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
}

impl ToBytes for bool {
    fn to_bytes(&self) -> Vec<u8> {
        vec![*self as u8]
    }
}

impl ToBytes for Hash {
    fn to_bytes(&self) -> Vec<u8> {
        self.to_vec()
    }
}

impl<T> ToBytes for Vec<T>
where
    T: ToBytes,
{
    fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::new();

        for b in self.iter() {
            result.extend_from_slice(&b.to_bytes());
        }

        result
    }
}

/// Beware! All changes to this implementation need to be backwards compatible. Failure to follow this
/// restriction will result in assets changing hashes between versions, causing unreachable funds.
///
/// Of course, this is true of any modification of the collection ID manager, but this is the place
/// where it's most likely to happen.
impl<MarketId> ToBytes for Asset<MarketId>
where
    MarketId: Encode,
{
    fn to_bytes(&self) -> Vec<u8> {
        self.encode()
    }
}
