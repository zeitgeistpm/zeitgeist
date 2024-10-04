use sp_runtime::DispatchError;

pub(crate) trait CombinatorialIdManager {
    type Asset;
    type MarketId;
    type CombinatorialId;

    // TODO Replace `Vec<bool>` with a more effective bit mask type.
    fn get_collection_id(
        parent_collection_id: Option<Self::CombinatorialId>,
        market_id: Self::MarketId,
        index_set: Vec<bool>,
        force_max_work: bool,
    ) -> Option<Self::CombinatorialId>;

    fn get_position_id(
        collateral: Self::Asset,
        collection_id: Self::CombinatorialId,
    ) -> Self::CombinatorialId;
}
