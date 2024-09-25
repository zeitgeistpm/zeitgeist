use crate::traits::IdManager;
use core::marker::PhantomData;
use ethnum::U256;
use frame_support::Blake2_256;
use sp_runtime::DispatchError;
use zeitgeist_primitives::Asset;

pub(crate) struct CryptographicIdManager<MarketId>(PhantomData<MarketId>);

impl<MarketId> IdManager for CryptographicIdManager<MarketId>
where
    MarketId: Into<U256>,
{
    type Asset = Asset<MarketId>;
    type Id = U256;
    type MarketId = MarketId;

    fn get_collection_id(
        parent_collection_id: Option<Self::Id>,
        market_id: Self::MarketId,
        index_set: Vec<bool>,
    ) -> Result<Self::Id, DispatchError> {
        Ok(0u8.into())
    }

    fn get_position_id(
        collateral: Self::Asset,
        collection_id: Self::Id,
    ) -> Result<Self::Id, DispatchError> {
        let bytes = hash_pair((collateral, collection_id))?;

        U256::from_be_slice(&bytes)
    }
}

fn hash_pair<T1, T2>(pair: (T1, T2)) -> Option<Vec<u8>>
where
    T1: MaybeToBytes,
    T2: MaybeToBytes,
{
    let result = Vec::new();

    result.extend_from_slice(pair.0.maybe_to_bytes()?);
    result.extend_from_slice(pair.1.maybe_to_bytes()?);

    result
}

// TODO Move into traits!
trait MaybeToBytes {
    /// `None` indicates failure.
    fn maybe_to_bytes(&self) -> Option<Vec<u8>>;
}

impl MaybeToBytes for u128 {
    fn maybe_to_bytes(&self) -> Option<Vec<u8>> {
        Some(self.to_be_bytes())
    }
}

impl MaybeToBytes for U256 {
    fn maybe_to_bytes(&self) -> Option<Vec<u8>> {}
}

/// Beware! All changes to this function need to be backwards compatible. Failure to follow this
/// restriction will result in assets changing hashes between versions, causing unreachable funds.
impl<MarketId> MaybeToBytes for Asset<MarketId> {
    fn maybe_to_bytes(&self) -> Option<Vec<u8>> {
        let pair = match self {
            Ztg => (0, 0),
            ForeignAsset(id) => (1, id),
            _ => return None,
        };

        hash_pair(pair)
    }
}
