// Copyright 2024 Forecasting Technologies LTD.
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

use crate::{traits::PoolOperations, weights::WeightInfoZeitgeist, AssetOf, Config, Error, Pools};
use frame_support::pallet_prelude::Weight;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::DispatchError;
use zeitgeist_primitives::traits::FutarchyOracle;

/// Struct that implements `FutarchyOracle` using price measurements from liquidity pools.
///
/// The oracle evaluates to `true` if and only if the `positive_outcome` is more valuable than the
/// `negative_outcome` in the liquidity pool specified by `pool_id`.
#[derive(Clone, Debug, Decode, Encode, Eq, MaxEncodedLen, PartialEq, TypeInfo)]
pub struct DecisionMarketOracle<T>
where
    T: Config,
{
    pool_id: T::PoolId,
    positive_outcome: AssetOf<T>,
    negative_outcome: AssetOf<T>,
}

impl<T> DecisionMarketOracle<T>
where
    T: Config,
{
    pub fn new(
        pool_id: T::PoolId,
        positive_outcome: AssetOf<T>,
        negative_outcome: AssetOf<T>,
    ) -> Self {
        Self { pool_id, positive_outcome, negative_outcome }
    }

    // Utility implementation that uses the question mark operator to implement a fallible version
    // of `evaluate`.
    fn try_evaluate(&self) -> Result<bool, DispatchError> {
        let pool = Pools::<T>::get(self.pool_id)
            .ok_or::<DispatchError>(Error::<T>::PoolNotFound.into())?;

        let positive_value = pool.calculate_spot_price(self.positive_outcome)?;
        let negative_value = pool.calculate_spot_price(self.negative_outcome)?;

        let success = positive_value > negative_value;

        Ok(success)
    }
}

impl<T> FutarchyOracle for DecisionMarketOracle<T>
where
    T: Config,
{
    fn evaluate(&self) -> (Weight, bool) {
        // Err on the side of caution if the pool is not found or a calculation fails by not
        // enacting the policy.
        let value = self.try_evaluate().unwrap_or(false);

        (T::WeightInfo::decision_market_oracle_evaluate(), value)
    }
}
