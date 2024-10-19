use crate::{traits::pool_operations::PoolOperations, AssetOf, Config, Error, MarketIdOf, Pools};
use frame_support::pallet_prelude::Weight;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::DispatchError;
use zeitgeist_primitives::traits::FutarchyOracle;

#[derive(Clone, Debug, Decode, Encode, Eq, MaxEncodedLen, PartialEq, TypeInfo)]
pub struct DecisionMarketOracle<T>
where
    T: Config,
{
    market_id: MarketIdOf<T>,
    positive_outcome: AssetOf<T>,
    negative_outcome: AssetOf<T>,
}

// Utility implementation that uses the question mark operator to implement a fallible version of
// `evaluate`.
impl<T> DecisionMarketOracle<T>
where
    T: Config,
{
    fn try_evaluate(&self) -> Result<(Weight, bool), DispatchError> {
        let pool = Pools::<T>::get(self.market_id)
            .ok_or::<DispatchError>(Error::<T>::PoolNotFound.into())?;

        let positive_value = pool.calculate_spot_price(self.positive_outcome)?;
        let negative_value = pool.calculate_spot_price(self.negative_outcome)?;

        let success = positive_value > negative_value;
        // TODO Benchmark
        Ok((Default::default(), success))
    }
}

impl<T> FutarchyOracle for DecisionMarketOracle<T>
where
    T: Config,
{
    fn evaluate(&self) -> (Weight, bool) {
        // Err on the side of caution if the pool is not found or a calculation fails by not
        // enacting the policy.
        match self.try_evaluate() {
            Ok(result) => result,
            // TODO Benchmark
            Err(_) => (Default::default(), false),
        }
    }
}
