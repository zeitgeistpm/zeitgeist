mod alt_bn128;

use crate::traits::IdManager;
use alt_bn128::Hash;
use core::marker::PhantomData;
use ethnum::U256;
use frame_support::{Blake2_256, StorageHasher};
use sp_runtime::DispatchError;
use zeitgeist_primitives::types::Asset;

pub(crate) struct CryptographicIdManager<MarketId>(PhantomData<MarketId>);

impl<MarketId> IdManager for CryptographicIdManager<MarketId>
where
    MarketId: MaybeToBytes,
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
        let hash = hash_tuple(input)?;
        alt_bn128::get_collection_id(hash, parent_collection_id)
    }

    fn get_position_id(
        collateral: Self::Asset,
        collection_id: Self::Id,
    ) -> Option<Self::Id> {
        let input = (collateral, collection_id);
        hash_tuple(input)
    }
}

fn hash_tuple<T1, T2>(tuple: (T1, T2)) -> Option<Hash>
where
    T1: MaybeToBytes,
    T2: MaybeToBytes,
{
    let mut bytes = Vec::new();

    bytes.extend_from_slice(&tuple.0.maybe_to_bytes()?);
    bytes.extend_from_slice(&tuple.1.maybe_to_bytes()?);

    let result = Blake2_256::hash(&bytes);

    Some(result)
}

// TODO Move into traits!
trait MaybeToBytes {
    /// `None` indicates failure.
    fn maybe_to_bytes(&self) -> Option<Vec<u8>>;
}

// TODO Use macros for this
impl MaybeToBytes for u32 {
    fn maybe_to_bytes(&self) -> Option<Vec<u8>> {
        Some(self.to_be_bytes().to_vec())
    }
}

impl MaybeToBytes for u128 {
    fn maybe_to_bytes(&self) -> Option<Vec<u8>> {
        Some(self.to_be_bytes().to_vec())
    }
}

impl MaybeToBytes for bool {
    fn maybe_to_bytes(&self) -> Option<Vec<u8>> {
        Some(vec![*self as u8])
    }
}

impl MaybeToBytes for Hash {
    fn maybe_to_bytes(&self) -> Option<Vec<u8>> {
        Some(self.to_vec())
    }
}

impl<T> MaybeToBytes for Vec<T>
where
    T: MaybeToBytes,
{
    fn maybe_to_bytes(&self) -> Option<Vec<u8>> {
        let mut result = Vec::new();

        for b in self.iter() {
            result.extend_from_slice(&b.maybe_to_bytes()?);
        }

        Some(result)
    }
}

/// Beware! All changes to this implementation need to be backwards compatible. Failure to follow this
/// restriction will result in assets changing hashes between versions, causing unreachable funds.
///
/// Of course, this is true of any modification of the collection ID manager, but this is the place
/// where it's most likely to happen.
impl<MarketId> MaybeToBytes for Asset<MarketId> {
    fn maybe_to_bytes(&self) -> Option<Vec<u8>> {
        let pair = match self {
            Asset::Ztg => (0u32, 0u8.into()),
            Asset::ForeignAsset(id) => (1u32, *id),
            _ => return None,
        };

        hash_tuple(pair).map(|x| x.to_vec())
    }
}
