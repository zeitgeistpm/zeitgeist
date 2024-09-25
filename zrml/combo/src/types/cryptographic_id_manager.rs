use crate::traits::IdManager;
use core::marker::PhantomData;
use ethnum::U256;
use sp_runtime::DispatchError;

type Id = U256;

pub(crate) struct CryptographicIdManager<Asset, MarketId>(PhantomData<(Asset, MarketId)>);

impl<Asset, MarketId> IdManager for CryptographicIdManager<Asset, MarketId>
where
    MarketId: Into<Id>,
{
    type Asset = u128;
    type Id = Id;
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
