use sp_runtime::DispatchError;

pub(crate) trait IdManager {
    type Asset;
    type MarketId: Into<Self::Id>;
    type Id;

    // TODO Replace `Vec<bool>` with a more effective bit mask type.
    fn get_collection_id(
        parent_collection_id: Option<Self::Id>,
        market_id: Self::MarketId,
        index_set: Vec<bool>,
    ) -> Result<Self::Id, DispatchError>;

    fn get_position_id(
        collateral: Self::Asset,
        collection_id: Self::Id,
    ) -> Result<Self::Id, DispatchError>;
}
