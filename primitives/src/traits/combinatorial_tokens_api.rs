use alloc::vec::Vec;
use frame_support::pallet_prelude::DispatchResultWithPostInfo;

pub trait CombinatorialTokensApi {
    type AccountId;
    type Balance;
    type CombinatorialId;
    type MarketId;

    fn split_position(
        who: Self::AccountId,
        parent_collection_id: Option<Self::CombinatorialId>,
        market_id: Self::MarketId,
        partition: Vec<Vec<bool>>,
        amount: Self::Balance,
        force_max_work: bool,
    ) -> DispatchResultWithPostInfo;

    fn merge_position(
        who: Self::AccountId,
        parent_collection_id: Option<Self::CombinatorialId>,
        market_id: Self::MarketId,
        partition: Vec<Vec<bool>>,
        amount: Self::Balance,
        force_max_work: bool,
    ) -> DispatchResultWithPostInfo;

    fn redeem_position(
        who: Self::AccountId,
        parent_collection_id: Option<Self::CombinatorialId>,
        market_id: Self::MarketId,
        index_set: Vec<bool>,
        force_max_work: bool,
    ) -> DispatchResultWithPostInfo;
}
