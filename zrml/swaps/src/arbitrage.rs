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
    types::{Asset, Pool},
};

// TODO Research: Why do we need the `Fixed`/`u128` type to begin with? Can't we just use a generic `Balance` for all Balancer math functions?

// TODO Make this a generic parameter of `Arbitrage`
type Fixed = u128;

const TOLERANCE: Fixed = BASE / 1_000; // 0.001

// TODO Rename to `ArbitrageForCpmm`.
pub trait Arbitrage<Balance, MarketId>
where
    MarketId: MaxEncodedLen,
{
    fn calc_total_spot_price(
        &self,
        balances: &BTreeMap<Asset<MarketId>, Balance>,
    ) -> Result<Fixed, &'static str>;

    fn calc_arbitrage_amount_mint_sell(
        &self,
        balances: &BTreeMap<Asset<MarketId>, Balance>,
        max_iterations: usize,
    ) -> Result<(Balance, usize), &'static str>;

    fn calc_arbitrage_amount_buy_burn(
        &self,
        balances: &BTreeMap<Asset<MarketId>, Balance>,
        max_iterations: usize,
    ) -> Result<(Balance, usize), &'static str>;
}

impl<Balance, MarketId> Arbitrage<Balance, MarketId> for Pool<Balance, MarketId>
where
    Balance: AtLeast32BitUnsigned + Copy,
    MarketId: MaxEncodedLen + AtLeast32Bit + Copy,
    Pool<Balance, MarketId>: ArbitrageHelper<Balance, MarketId>,
{
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
        let weight_in = weights
            .get(&self.base_asset)
            .cloned()
            .ok_or("Unexpectedly found no weight for base asset")?;
        let mut result: Fixed = 0;
        for asset in self.assets.iter().filter(|a| **a != self.base_asset) {
            let balance_out: Fixed = balances
                .get(asset)
                .cloned()
                .ok_or("Asset balance missing from BTreeMap")?
                .saturated_into();
            let weight_out = weights
                .get(asset)
                .cloned()
                .ok_or("Unexpectedly found no weight for outcome asset.")?;
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

    fn calc_arbitrage_amount_mint_sell(
        &self,
        balances: &BTreeMap<Asset<MarketId>, Balance>,
        max_iterations: usize,
    ) -> Result<(Balance, usize), &'static str> {
        self.calc_arbitrage_amount_common(balances, |a| *a == self.base_asset, max_iterations)
    }

    fn calc_arbitrage_amount_buy_burn(
        &self,
        balances: &BTreeMap<Asset<MarketId>, Balance>,
        max_iterations: usize,
    ) -> Result<(Balance, usize), &'static str> {
        self.calc_arbitrage_amount_common(balances, |a| *a != self.base_asset, max_iterations)
    }
}

trait ArbitrageHelper<Balance, MarketId>
where
    MarketId: MaxEncodedLen,
{
    /// Common code of `Arbitrage::calc_arbitrage_amount_*`.
    ///
    /// The only difference between the two `calc_arbitrage_amount_*` functions is that they
    /// increase/decrease different assets. The `cond` parameter is `true` on assets that must be
    /// decreased, `false` otherwise.
    ///
    /// # Arguments
    ///
    /// - `balances`: Maps assets to their balance in the pool.
    /// - `cond`: Returns `true` if the asset's balance must be decreased, `false` otherwise.
    /// - `max_iterations`: The maximum number of iterations to use in the bisection method.
    fn calc_arbitrage_amount_common<F>(
        &self,
        balances: &BTreeMap<Asset<MarketId>, Balance>,
        cond: F,
        max_iterations: usize,
    ) -> Result<(Balance, usize), &'static str>
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
        max_iterations: usize,
    ) -> Result<(Balance, usize), &'static str>
    where
        F: Fn(&Asset<MarketId>) -> bool,
    {
        let smallest_balance: Fixed = balances
            .iter()
            .filter_map(|(a, b)| if cond(a) { Some(b) } else { None })
            .min()
            .cloned()
            .ok_or("calc_arbitrage_amount_common: Cannot find any matching assets")?
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
        let (preimage, iterations) = calc_preimage::<Fixed, _>(
            calc_total_spot_price_after_arbitrage,
            BASE,
            0,
            smallest_balance / 2,
            max_iterations,
            TOLERANCE,
        )?;
        Ok((preimage.saturated_into(), iterations))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
