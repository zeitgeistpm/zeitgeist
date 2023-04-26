use frame_support::dispatch::DispatchError;

use crate::types::VoteItem;

pub trait VoteCheckApi {
    type MarketId;

    fn pre_validate(market_id: &Self::MarketId, vote_item: VoteItem) -> Result<(), DispatchError>;
}

pub trait AppealCheckApi {
    type MarketId;

    fn pre_appeal(market_id: &Self::MarketId) -> Result<(), DispatchError>;
}

pub trait DefaultWinnerApi {
    type MarketId;

    fn default_winner(market_id: &Self::MarketId) -> Result<VoteItem, DispatchError>;
}
