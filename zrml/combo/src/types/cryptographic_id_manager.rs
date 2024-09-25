use crate::traits::IdManager;
use ethnum::U256;
use sp_runtime::DispatchError;

pub(crate) struct CryptographicIdManager;

impl IdManager for CryptographicIdManager {
    type Asset = u128;
    type Id = U256;
    type MarketId = u128;

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
        Ok(0u8.into())
    }
}
