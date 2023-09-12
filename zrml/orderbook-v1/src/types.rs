use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use zeitgeist_primitives::types::Asset;

pub type OrderId = u128;

#[derive(Clone, Encode, Eq, Decode, MaxEncodedLen, PartialEq, RuntimeDebug, TypeInfo)]
pub enum OrderSide {
    Bid,
    Ask,
}

#[derive(Clone, Encode, Eq, Decode, MaxEncodedLen, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Order<AccountId, Balance, MarketId: MaxEncodedLen> {
    pub market_id: MarketId,
    pub order_id: OrderId,
    pub side: OrderSide,
    pub maker: AccountId,
    pub outcome_asset: Asset<MarketId>,
    pub base_asset: Asset<MarketId>,
    pub amount: Balance,
    pub price: Balance,
}
