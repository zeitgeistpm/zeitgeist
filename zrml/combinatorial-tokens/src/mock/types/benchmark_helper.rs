use crate::{
    mock::{runtime::Runtime, types::MockPayout},
    BalanceOf, MarketIdOf,
};
use alloc::vec::Vec;
use sp_runtime::DispatchResult;
use zeitgeist_primitives::traits::CombinatorialTokensBenchmarkHelper;

pub struct BenchmarkHelper;

impl CombinatorialTokensBenchmarkHelper for BenchmarkHelper {
    type Balance = BalanceOf<Runtime>;
    type MarketId = MarketIdOf<Runtime>;

    /// A bit of a messy implementation as this sets the return value of the next `payout_vector`
    /// call, regardless of what `_market_id` is.
    fn setup_payout_vector(
        _market_id: Self::MarketId,
        payout: Option<Vec<Self::Balance>>,
    ) -> DispatchResult {
        MockPayout::set_return_value(payout);

        Ok(())
    }
}
