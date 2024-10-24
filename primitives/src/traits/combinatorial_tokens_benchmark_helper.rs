use alloc::vec::Vec;
use sp_runtime::DispatchResult;

pub trait CombinatorialTokensBenchmarkHelper {
    type Balance;
    type MarketId;

    /// Prepares the market with the specified `market_id` to have a particular `payout`.
    fn setup_payout_vector(
        market_id: Self::MarketId,
        payout: Option<Vec<Self::Balance>>,
    ) -> DispatchResult;
}
