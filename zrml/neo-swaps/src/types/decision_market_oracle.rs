// Copyright 2024-2025 Forecasting Technologies LTD.
//
// This file is part of Zeitgeist.
//
// Zeitgeist is free software: you can redistribute it and/or modify it
// under the terms of the GNU General Public License as published by the
// Free Software Foundation, either version 3 of the License, or (at
// your option) any later version.
//
// Zeitgeist is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Zeitgeist. If not, see <https://www.gnu.org/licenses/>.

use crate::{
    traits::PoolOperations, types::DecisionMarketOracleScoreboard, weights::WeightInfoZeitgeist,
    AssetOf, BalanceOf, Config, Error, Pools,
};
use frame_support::pallet_prelude::Weight;
use frame_system::pallet_prelude::BlockNumberFor;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::DispatchError;
use zeitgeist_primitives::traits::FutarchyOracle;

/// Struct that implements `FutarchyOracle` using price measurements from liquidity pools.
///
/// The oracle evaluates to `true` if and only if the `positive_outcome` is more valuable than the
/// `negative_outcome` in the liquidity pool specified by `pool_id` over
/// a period of time for a certain absolute and relative threshold determined by a
/// [`DecisionMarketOracleScoreboard`].
#[derive(Clone, Debug, Decode, Encode, Eq, MaxEncodedLen, PartialEq, TypeInfo)]
pub struct DecisionMarketOracle<T>
where
    T: Config,
{
    pool_id: T::PoolId,
    positive_outcome: AssetOf<T>,
    negative_outcome: AssetOf<T>,
    scoreboard: DecisionMarketOracleScoreboard<T>,
}

impl<T> DecisionMarketOracle<T>
where
    T: Config,
{
    pub fn new(
        pool_id: T::PoolId,
        positive_outcome: AssetOf<T>,
        negative_outcome: AssetOf<T>,
        scoreboard: DecisionMarketOracleScoreboard<T>,
    ) -> Self {
        Self { pool_id, positive_outcome, negative_outcome, scoreboard }
    }

    fn try_get_prices(&self) -> Result<(BalanceOf<T>, BalanceOf<T>), DispatchError> {
        let pool = Pools::<T>::get(self.pool_id)
            .ok_or::<DispatchError>(Error::<T>::PoolNotFound.into())?;

        let positive_value = pool.calculate_spot_price(self.positive_outcome)?;
        let negative_value = pool.calculate_spot_price(self.negative_outcome)?;

        Ok((positive_value, negative_value))
    }
}

impl<T> FutarchyOracle for DecisionMarketOracle<T>
where
    T: Config,
{
    type BlockNumber = BlockNumberFor<T>;

    fn evaluate(&self) -> (Weight, bool) {
        (T::WeightInfo::decision_market_oracle_evaluate(), self.scoreboard.evaluate())
    }

    fn update(&mut self, now: Self::BlockNumber) -> Weight {
        if let Ok((positive_outcome_price, negative_outcome_price)) = self.try_get_prices() {
            self.scoreboard.update(now, positive_outcome_price, negative_outcome_price);
        } else {
            // Err on the side of caution if the pool is not found or a calculation fails by not
            // enacting the policy.
            self.scoreboard.skip_update(now);
        }

        T::WeightInfo::decision_market_oracle_update()
    }
}
