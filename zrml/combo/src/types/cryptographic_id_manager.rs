use crate::{alt_bn128, traits::IdManager};
use core::marker::PhantomData;
use ethnum::U256;
use frame_support::{Blake2_256, StorageHasher};
use sp_runtime::DispatchError;
use zeitgeist_primitives::types::Asset;

type Hash = [u8; 32];

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
        // TODO: This could just return an `Option` as we don't really expect this to fail with any
        // informative results.
    ) -> Result<Self::Id, DispatchError> {
        let hash = hash_pair((market_id, index_set)).ok_or(DispatchError::Other("TODO"))?;
        alt_bn128::get_collection_id(hash, parent_collection_id).ok_or(DispatchError::Other("TODO"))
    }

    fn get_position_id(
        collateral: Self::Asset,
        collection_id: Self::Id,
        // TODO: This could just return an `Option` as we don't really expect this to fail with any
        // informative results.
    ) -> Result<Self::Id, DispatchError> {
        hash_pair((collateral, collection_id)).ok_or(DispatchError::Other("TODO"))
    }
}

// TODO Replace pair with parameters.
fn hash_pair<T1, T2>(pair: (T1, T2)) -> Option<[u8; 32]>
// TODO Let this return Hash.
where
    T1: MaybeToBytes,
    T2: MaybeToBytes,
{
    let mut bytes = Vec::new();

    bytes.extend_from_slice(&pair.0.maybe_to_bytes()?);
    bytes.extend_from_slice(&pair.1.maybe_to_bytes()?);

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

impl MaybeToBytes for U256 {
    fn maybe_to_bytes(&self) -> Option<Vec<u8>> {
        Some(self.to_be_bytes().to_vec())
    }
}

impl MaybeToBytes for bool {
    fn maybe_to_bytes(&self) -> Option<Vec<u8>> {
        Some(vec![*self as u8])
    }
}

impl MaybeToBytes for [u8; 32] {
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

/// Beware! All changes to this function need to be backwards compatible. Failure to follow this
/// restriction will result in assets changing hashes between versions, causing unreachable funds.
impl<MarketId> MaybeToBytes for Asset<MarketId> {
    fn maybe_to_bytes(&self) -> Option<Vec<u8>> {
        let pair = match self {
            Asset::Ztg => (0u32, 0u8.into()),
            Asset::ForeignAsset(id) => (1u32, *id),
            _ => return None,
        };

        hash_pair(pair).map(|x| x.to_vec())
    }
}
