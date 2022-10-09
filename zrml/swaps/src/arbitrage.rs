// Copyright 2021-2022 Zeitgeist PM LLC.
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

use crate::{math::calc_spot_price, root::calc_preimage};
use alloc::collections::btree_map::BTreeMap;
use parity_scale_codec::MaxEncodedLen;
use sp_runtime::{
    traits::{AtLeast32Bit, AtLeast32BitUnsigned},
    SaturatedConversion,
};
use zeitgeist_primitives::{
    constants::BASE,
    types::{Asset, Pool, PoolId, PoolStatus, ScoringRule},
};

// TODO Research: Why do we need the `Fixed`/`u128` type to begin with? Can't we just use a generic `Balance` for all Balancer math functions?

// TODO Make this a generic parameter of `Arbitrage`
type Fixed = u128;

const MAX_ITERATIONS: usize = 30;
const TOLERANCE: Fixed = BASE / 1_000; // 0.001

// TODO Some of these trait bounds may be removable (only required in impl)
// TODO Rename to `ArbitrageForCpmm`.
pub trait Arbitrage<Balance, MarketId>
where
    Balance: AtLeast32BitUnsigned + Copy,
    MarketId: MaxEncodedLen + AtLeast32Bit + Copy,
{
    fn calc_total_spot_price(
        &self,
        balances: &BTreeMap<Asset<MarketId>, Balance>,
    ) -> Result<Fixed, &'static str>;

    fn calc_arbitrage_amount_mint_sell(
        &self,
        balances: &BTreeMap<Asset<MarketId>, Balance>,
    ) -> Result<Balance, &'static str>;

    fn calc_arbitrage_amount_buy_burn(
        &self,
        balances: &BTreeMap<Asset<MarketId>, Balance>,
    ) -> Result<Balance, &'static str>;
}

impl<Balance, MarketId> Arbitrage<Balance, MarketId> for Pool<Balance, MarketId>
where
    Balance: AtLeast32BitUnsigned + Copy,
    MarketId: MaxEncodedLen + AtLeast32Bit + Copy,
    Pool<Balance, MarketId>: ArbitrageHelper<Balance, MarketId>,
{
    // TODO Use dependency injection to add a shift?
    fn calc_total_spot_price(
        &self,
        balances: &BTreeMap<Asset<MarketId>, Balance>,
    ) -> Result<Fixed, &'static str> {
        let weights = self.weights.as_ref().ok_or("Unexpectedly found no weights in pool.")?;
        let balance_in = balances
            .get(&self.base_asset)
            .cloned()
            .ok_or("Base asset balance missing")?
            .saturated_into();
        let weight_in =
            weights.get(&self.base_asset).cloned().ok_or("Base asset weight missing")?;
        let mut result: Fixed = 0;
        for asset in self.assets.iter().filter(|a| **a != self.base_asset) {
            // TODO Need better error message here!
            let balance_out: Fixed = balances
                .get(asset)
                .cloned()
                .ok_or("Asset balance missing")
                .unwrap() // TODO: Unwrap
                .saturated_into();
            // TODO Individualize error message!
            let weight_out =
                weights.get(asset).cloned().ok_or("Unexpected found no weight for asset.").unwrap();
            // We're deliberately _not_ using the pool's swap fee!
            result = result.saturating_add(calc_spot_price(
                balance_in,
                weight_in,
                balance_out,
                weight_out,
                0,
            )?);
        }
        Ok(result)
    }

    // Calling with a non-CPMM pool results in undefined behavior.
    fn calc_arbitrage_amount_mint_sell(
        &self,
        balances: &BTreeMap<Asset<MarketId>, Balance>,
    ) -> Result<Balance, &'static str> {
        self.calc_arbitrage_amount_common(balances, |a| *a == self.base_asset)
    }

    fn calc_arbitrage_amount_buy_burn(
        &self,
        balances: &BTreeMap<Asset<MarketId>, Balance>,
    ) -> Result<Balance, &'static str> {
        self.calc_arbitrage_amount_common(balances, |a| *a != self.base_asset)
    }
}

// TODO Some of these trait bounds may be removable (only required in impl)
trait ArbitrageHelper<Balance, MarketId>
where
    Balance: AtLeast32BitUnsigned + Copy,
    MarketId: MaxEncodedLen + AtLeast32Bit + Copy,
{
    fn calc_arbitrage_amount_common<F>(
        &self,
        balances: &BTreeMap<Asset<MarketId>, Balance>,
        cond: F,
    ) -> Result<Balance, &'static str>
    where
        F: Fn(&Asset<MarketId>) -> bool;
}

impl<Balance, MarketId> ArbitrageHelper<Balance, MarketId> for Pool<Balance, MarketId>
where
    Balance: AtLeast32BitUnsigned + Copy,
    MarketId: MaxEncodedLen + AtLeast32Bit + Copy,
{
    fn calc_arbitrage_amount_common<F>(
        &self,
        balances: &BTreeMap<Asset<MarketId>, Balance>,
        cond: F,
    ) -> Result<Balance, &'static str>
    where
        F: Fn(&Asset<MarketId>) -> bool,
    {
        // The `unwrap_or` below should never occur
        let smallest_balance: Fixed = balances
            .iter()
            .filter(|(a, b)| cond(a))
            .map(|(_, b)| b)
            .min()
            .cloned()
            .ok_or("")?
            .saturated_into();
        let calc_total_spot_price_after_arbitrage = |amount: Fixed| -> Result<Fixed, &'static str> {
            let shifted_balances = balances
                .iter()
                .map(|(asset, bal)| {
                    if cond(asset) {
                        (*asset, bal.saturating_sub(amount.saturated_into()))
                    } else {
                        (*asset, bal.saturating_add(amount.saturated_into()))
                    }
                })
                .collect::<BTreeMap<_, _>>();
            self.calc_total_spot_price(&shifted_balances)
        };
        let (result, iterations) = calc_preimage::<Fixed, _>(
            calc_total_spot_price_after_arbitrage,
            BASE,
            0,
            smallest_balance / 2,
            MAX_ITERATIONS,
            TOLERANCE,
        )?;
        // TODO How to handle too many iterations?
        Ok(result.saturated_into())
    }
}
