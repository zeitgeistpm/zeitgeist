use crate::{BalanceOf, Config, MarketIdOf};
use alloc::vec::Vec;
use core::marker::PhantomData;
use sp_runtime::{traits::Zero, DispatchResult};
use zeitgeist_primitives::{
    traits::{CombinatorialTokensBenchmarkHelper, MarketCommonsPalletApi},
    types::{MarketStatus, OutcomeReport},
};

pub struct PredictionMarketsCombinatorialTokensBenchmarkHelper<T>(PhantomData<T>);

impl<T> CombinatorialTokensBenchmarkHelper
    for PredictionMarketsCombinatorialTokensBenchmarkHelper<T>
where
    T: Config,
{
    type Balance = BalanceOf<T>;
    type MarketId = MarketIdOf<T>;

    /// Aggressively modifies the market specified by `market_id` to be resolved. The payout vector
    /// must contain exactly one non-zero entry. Does absolutely no error management.
    fn setup_payout_vector(
        market_id: Self::MarketId,
        payout_vector: Option<Vec<Self::Balance>>,
    ) -> DispatchResult {
        let payout_vector = payout_vector.unwrap();
        let index = payout_vector.iter().position(|&value| !value.is_zero()).unwrap();

        <zrml_market_commons::Pallet<T> as MarketCommonsPalletApi>::mutate_market(
            &market_id,
            |market| {
                market.resolved_outcome =
                    Some(OutcomeReport::Categorical(index.try_into().unwrap()));
                market.status = MarketStatus::Resolved;

                Ok(())
            },
        )
    }
}
