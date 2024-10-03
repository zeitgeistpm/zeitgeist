use sp_runtime::DispatchError;

pub(crate) trait IdManager {
    type Asset;
    type MarketId;
    type Id;

    // TODO Replace `Vec<bool>` with a more effective bit mask type.
    fn get_collection_id(
        parent_collection_id: Option<Self::Id>,
        market_id: Self::MarketId,
        index_set: Vec<bool>,
        force_max_work: bool,
    ) -> Option<Self::Id>;

    fn get_position_id(
        collateral: Self::Asset,
        collection_id: Self::Id,
    ) -> Option<Self::Id>;
}
