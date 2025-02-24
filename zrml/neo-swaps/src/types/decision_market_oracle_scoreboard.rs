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

use crate::{BalanceOf, Config};
use frame_system::pallet_prelude::BlockNumberFor;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{traits::Zero, Saturating};
use zeitgeist_primitives::math::fixed::FixedDiv;

/// Records until the end of time.
#[derive(Clone, Debug, Decode, Encode, Eq, MaxEncodedLen, PartialEq, TypeInfo)]
pub struct DecisionMarketOracleScoreboard<T>
where
    T: Config,
{
    /// The block at which the oracle records its first tick.
    start: BlockNumberFor<T>,

    /// The number of ticks the positive outcome requires to have
    victory_margin: u128,

    /// The absolute minimum difference in prices required for the positive outcome to receive a
    /// point.
    price_margin_abs: BalanceOf<T>,

    /// The relative minimum difference in prices required for the positive outcome to receive a
    /// point, specified as fractional (i.e. 0.1 represents 10%).
    price_margin_rel: BalanceOf<T>,

    /// The number of ticks for the positive outcome.
    pass_score: u128,

    /// The number of ticks for the negative outcome.
    reject_score: u128,
}

impl<T> DecisionMarketOracleScoreboard<T>
where
    T: Config,
{
    pub fn new(
        start: BlockNumberFor<T>,
        victory_margin: u128,
        price_margin_abs: BalanceOf<T>,
        price_margin_rel: BalanceOf<T>,
    ) -> DecisionMarketOracleScoreboard<T> {
        DecisionMarketOracleScoreboard {
            start,
            victory_margin,
            price_margin_abs,
            price_margin_rel,
            pass_score: 0,
            reject_score: 0,
        }
    }

    pub fn update(
        &mut self,
        now: BlockNumberFor<T>,
        positive_outcome_price: BalanceOf<T>,
        negative_outcome_price: BalanceOf<T>,
    ) {
        if now < self.start {
            return;
        }

        // Saturation is fine as that just means that the negative outcome is more valuable than the
        // positive outcome.
        let margin_abs = positive_outcome_price.saturating_sub(negative_outcome_price);
        // In case of error, we're using zero here as a defensive default value.
        let margin_rel = margin_abs.bdiv(negative_outcome_price).unwrap_or(Zero::zero());

        if margin_abs >= self.price_margin_abs && margin_rel >= self.price_margin_rel {
            // Saturation is fine as that would mean the oracle has been collecting data for
            // hundreds of years.
            self.pass_score.saturating_inc();
        } else {
            // Saturation is fine as that would mean the oracle has been collecting data for
            // hundreds of years.
            self.reject_score.saturating_inc();
        }
    }

    pub fn evaluate(&self) -> bool {
        // Saturating is fine as that just means that the `reject_score` is higher than `pass_score`
        // anyways.
        let score_margin = self.pass_score.saturating_sub(self.reject_score);

        score_margin >= self.victory_margin
    }

    /// Skips update on this block and awards a point to the negative outcome.
    pub fn skip_update(&mut self, now: BlockNumberFor<T>) {
        if now < self.start {
            return;
        }

        // Saturation is fine as that would mean the oracle has been collecting data for
        // hundreds of years.
        self.reject_score.saturating_inc();
    }
}
