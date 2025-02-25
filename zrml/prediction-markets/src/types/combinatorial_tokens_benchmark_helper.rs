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
