use alloc::vec::Vec;
use sp_runtime::DispatchResult;

pub trait DeployPoolApi {
    type AccountId;
    type Balance;
    type MarketId;

    fn deploy_pool(
        who: Self::AccountId,
        market_id: Self::MarketId,
        amount: Self::Balance,
        swap_prices: Vec<Self::Balance>,
        swap_fee: Self::Balance,
    ) -> DispatchResult;
}
