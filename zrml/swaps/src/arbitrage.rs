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

//! Traits and implementation for automatic arbitrage of CPMM pools.

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

type Fixed = u128;

const TOLERANCE: Fixed = BASE / 1_000; // 0.001

/// This trait implements approximations for on-chain arbitrage of CPMM pools using bisection.
///
/// All calculations depend on on-chain, which are passed using the `balances` parameter.
pub(crate) trait ArbitrageForCpmm<Balance, MarketId>
where
    MarketId: MaxEncodedLen,
{
    /// Calculate the total spot price (sum of all spot prices of outcome tokens).
    ///
    /// Arguments:
    ///
    /// * `balances`: Maps assets to their current balance.
    fn calc_total_spot_price(
        &self,
        balances: &BTreeMap<Asset<MarketId>, Balance>,
    ) -> Result<Fixed, &'static str>;

    /// Approximate the amount to mint/sell to move the total spot price close to `1`.
    ///
    /// Arguments:
    ///
    /// * `balances`: Maps assets to their current balance.
    /// * `max_iterations`: Maximum number of iterations allowed in the bisection method.
    fn calc_arbitrage_amount_mint_sell(
        &self,
        balances: &BTreeMap<Asset<MarketId>, Balance>,
        max_iterations: usize,
    ) -> Result<(Balance, usize), &'static str>;

    /// Approximate the amount to buy/burn to move the total spot price close to `1`.
    ///
    /// Arguments:
    ///
    /// * `balances`: Maps assets to their current balance.
    /// * `max_iterations`: Maximum number of iterations allowed in the bisection method.
    fn calc_arbitrage_amount_buy_burn(
        &self,
        balances: &BTreeMap<Asset<MarketId>, Balance>,
        max_iterations: usize,
    ) -> Result<(Balance, usize), &'static str>;
}

impl<Balance, MarketId> ArbitrageForCpmm<Balance, MarketId> for Pool<Balance, MarketId>
where
    Balance: AtLeast32BitUnsigned + Copy,
    MarketId: MaxEncodedLen + AtLeast32Bit + Copy,
    Pool<Balance, MarketId>: ArbitrageForCpmmHelper<Balance, MarketId>,
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

trait ArbitrageForCpmmHelper<Balance, MarketId>
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

impl<Balance, MarketId> ArbitrageForCpmmHelper<Balance, MarketId> for Pool<Balance, MarketId>
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
        // We use `smallest_balance / 2` so we never reduce a balance to zero.
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
    use test_case::test_case;
    use zeitgeist_primitives::{
        constants::CENT,
        types::{Asset, PoolStatus, ScoringRule},
    };

    type MarketId = u128;

    const _1: u128 = BASE;
    const _2: u128 = 2 * BASE;
    const _3: u128 = 3 * BASE;
    const _4: u128 = 4 * BASE;
    const _5: u128 = 5 * BASE;
    const _6: u128 = 6 * BASE;
    const _7: u128 = 7 * BASE;
    const _8: u128 = 8 * BASE;
    const _9: u128 = 9 * BASE;
    const _10: u128 = 10 * BASE;
    const _100: u128 = 100 * BASE;
    const _125: u128 = 125 * BASE;
    const _150: u128 = 150 * BASE;
    const _1_4: u128 = BASE / 4;
    const _1_10: u128 = BASE / 10;
    const _1_1000: u128 = BASE / 1_000;

    // Macro for comparing fixed point u128.
    #[allow(unused_macros)]
    macro_rules! assert_approx {
        ($left:expr, $right:expr, $precision:expr $(,)?) => {
            match (&$left, &$right, &$precision) {
                (left_val, right_val, precision_val) => {
                    let diff = if *left_val > *right_val {
                        *left_val - *right_val
                    } else {
                        *right_val - *left_val
                    };
                    if diff > $precision {
                        panic!("{} is not {}-close to {}", *left_val, *precision_val, *right_val);
                    }
                }
            }
        };
    }

    // Some of these tests are taken from our Python Balancer playground, some as snapshots from
    // the Zeitgeist chain.
    #[test_case(vec![_3, _1, _1, _1], vec![_1, _1, _1, _1], _1 - 1)]
    #[test_case(vec![_6, _3, _3], vec![_100, _100, _100], _1)]
    #[test_case(vec![_6, _3, _3], vec![_100, _100, _150], 8_333_333_333)]
    #[test_case(vec![_7, _3, _4], vec![_100, _100, _150], 8_095_238_096)]
    #[test_case(vec![_9, _3, _6], vec![_100, _125, _150], 7_111_111_111)]
    #[test_case(vec![_9, _3, _6], vec![_100, _125, _100], 9_333_333_334)]
    #[test_case(vec![_6, _3, _3], vec![_125, _100, _100], 12_500_000_000)]
    #[test_case(vec![_6, _3, _3], vec![_125, _100, _150], 10_416_666_667)]
    #[test_case(vec![_7, _3, _4], vec![_125, _100, _150], 10_119_047_619)]
    #[test_case(vec![_9, _3, _6], vec![_125, _125, _150], 8_888_888_889)]
    #[test_case(vec![_9, _3, _6], vec![_125, _125, _100], 11_666_666_666)]
    #[test_case(vec![_6, _3, _3], vec![_150, _100, _100], 15_000_000_000)]
    #[test_case(vec![_6, _3, _3], vec![_150, _100, _150], 12_500_000_000)]
    #[test_case(vec![_7, _3, _4], vec![_150, _100, _150], 12_142_857_143)]
    #[test_case(vec![_9, _3, _6], vec![_150, _125, _150], 10_666_666_667)]
    #[test_case(vec![_9, _3, _6], vec![_150, _125, _100], 14_000_000_000)]
    #[test_case(
        vec![_4, _1, _1, _1, _1],
        vec![
            5_371_011_843_167,
            1_697_583_448_000,
            5_399_900_980_000,
            7_370_000_000_000,
            7_367_296_940_400
        ],
        14_040_918_578
    )]
    fn calc_total_spot_price_calculates_correct_values(
        weights: Vec<u128>,
        balances: Vec<u128>,
        expected: u128,
    ) {
        let pool = construct_pool(None, weights.clone());
        let balances = collect_balances_into_map(pool.assets.clone(), balances);
        assert_eq!(pool.calc_total_spot_price(&balances).unwrap(), expected);
        // Test that swap fees make no difference!
        let pool = construct_pool(Some(_1_10), weights);
        assert_eq!(pool.calc_total_spot_price(&balances).unwrap(), expected);
    }

    #[test]
    fn calc_total_spot_price_errors_if_asset_balance_is_missing() {
        let pool = construct_pool(None, vec![_3, _1, _1, _1]);
        let balances = collect_balances_into_map(pool.assets[..2].into(), vec![_1; 3]);
        assert_eq!(
            pool.calc_total_spot_price(&balances),
            Err("Asset balance missing from BTreeMap"),
        );
    }

    // The first case shouldn't be validated, as it is an example where the algorithm overshoots the
    // target:
    // #[test_case(vec![_3, _1, _1, _1], vec![_1, _1, _1, _1], 0)]
    #[test_case(vec![_6, _3, _3], vec![_100, _100, _100], 0)]
    #[test_case(vec![_6, _3, _3], vec![_100, _100, _150], 97_877_502_440)]
    #[test_case(vec![_7, _3, _4], vec![_100, _100, _150], 115_013_122_558)]
    #[test_case(vec![_9, _3, _6], vec![_100, _125, _150], 202_188_491_820)]
    #[test_case(vec![_9, _3, _6], vec![_100, _125, _100], 35_530_090_331)]
    #[test_case(vec![_9, _3, _6], vec![_125, _125, _150], 77_810_287_474)]
    //                                                     10_000_000_000 = 1 * BASE
    //                                                  1_000_000_000_000 = 100 * BASE
    fn calc_arbitrage_amount_buy_burn_calculates_correct_results(
        weights: Vec<u128>,
        balances: Vec<u128>,
        expected: u128,
    ) {
        let pool = construct_pool(None, weights);
        let balances = collect_balances_into_map(pool.assets.clone(), balances);
        let (amount, _) = pool.calc_arbitrage_amount_buy_burn(&balances, 30).unwrap();
        assert_eq!(amount, expected);
    }

    #[test_case(vec![_3, _1, _1, _1], vec![_1, _1, _1, _1])]
    #[test_case(vec![_6, _3, _3], vec![_100, _100, _100])]
    #[test_case(vec![_6, _3, _3], vec![_100, _100, _150])]
    #[test_case(vec![_7, _3, _4], vec![_100, _100, _150])]
    #[test_case(vec![_9, _3, _6], vec![_100, _125, _150])]
    #[test_case(vec![_9, _3, _6], vec![_100, _125, _100])]
    #[test_case(vec![_9, _3, _6], vec![_125, _125, _150])]
    fn calc_arbitrage_amount_buy_burn_reconfigures_pool_correctly(
        weights: Vec<u128>,
        balances: Vec<u128>,
    ) {
        let pool = construct_pool(None, weights);
        let mut balances = collect_balances_into_map(pool.assets.clone(), balances);
        let (amount, _) = pool.calc_arbitrage_amount_buy_burn(&balances, 30).unwrap();
        *balances.get_mut(&pool.assets[0]).unwrap() += amount;
        for asset in &pool.assets[1..] {
            *balances.get_mut(asset).unwrap() -= amount;
        }
        // It's an iffy question what to use as `precision` parameter here. The precision depends
        // on the derivative of the total spot price function `f` in ZIP-1 on the interval `[0,
        // amount]`.
        assert_approx!(pool.calc_total_spot_price(&balances).unwrap(), _1, CENT);
    }

    #[test]
    fn calc_arbitrage_amount_buy_burn_errors_if_asset_balance_is_missing() {
        let pool = construct_pool(None, vec![_3, _1, _1, _1]);
        let balances = collect_balances_into_map(pool.assets[..2].into(), vec![_1; 3]);
        assert_eq!(
            pool.calc_arbitrage_amount_buy_burn(&balances, usize::MAX),
            Err("Asset balance missing from BTreeMap"),
        );
    }

    #[test_case(vec![_6, _3, _3], vec![_125, _100, _100], 124_998_092_650)]
    #[test_case(vec![_6, _3, _3], vec![_125, _100, _150], 24_518_966_674)]
    #[test_case(vec![_7, _3, _4], vec![_125, _100, _150], 7_200_241_088)]
    #[test_case(vec![_9, _3, _6], vec![_125, _125, _100], 88_872_909_544)]
    #[test_case(vec![_6, _3, _3], vec![_150, _100, _100], 250_001_907_347)]
    #[test_case(vec![_6, _3, _3], vec![_150, _100, _150], 147_359_848_021)]
    #[test_case(vec![_7, _3, _4], vec![_150, _100, _150], 129_919_052_122)]
    #[test_case(vec![_9, _3, _6], vec![_150, _125, _150], 46_697_616_576)]
    #[test_case(vec![_9, _3, _6], vec![_150, _125, _100], 213_369_369_505)]
    fn calc_arbitrage_amount_mint_sell_calculates_correct_results(
        weights: Vec<u128>,
        balances: Vec<u128>,
        expected: u128,
    ) {
        let pool = construct_pool(None, weights);
        let balances = collect_balances_into_map(pool.assets.clone(), balances);
        let (amount, _) = pool.calc_arbitrage_amount_mint_sell(&balances, 30).unwrap();
        assert_eq!(amount, expected);
    }

    #[test_case(vec![_6, _3, _3], vec![_125, _100, _100])]
    #[test_case(vec![_6, _3, _3], vec![_125, _100, _150])]
    #[test_case(vec![_7, _3, _4], vec![_125, _100, _150])]
    #[test_case(vec![_9, _3, _6], vec![_125, _125, _100])]
    #[test_case(vec![_6, _3, _3], vec![_150, _100, _100])]
    #[test_case(vec![_6, _3, _3], vec![_150, _100, _150])]
    #[test_case(vec![_7, _3, _4], vec![_150, _100, _150])]
    #[test_case(vec![_9, _3, _6], vec![_150, _125, _150])]
    #[test_case(vec![_9, _3, _6], vec![_150, _125, _100])]
    fn calc_arbitrage_amount_mint_sell_reconfigures_pool_correctly(
        weights: Vec<u128>,
        balances: Vec<u128>,
    ) {
        let pool = construct_pool(None, weights);
        let mut balances = collect_balances_into_map(pool.assets.clone(), balances);
        let (amount, _) = pool.calc_arbitrage_amount_mint_sell(&balances, 30).unwrap();
        *balances.get_mut(&pool.assets[0]).unwrap() -= amount;
        for asset in &pool.assets[1..] {
            *balances.get_mut(asset).unwrap() += amount;
        }
        // It's an iffy question what to use as `precision` parameter here. The precision depends
        // on the derivative of the total spot price function `f` in ZIP-1 on the interval `[0,
        // amount]`.
        assert_approx!(pool.calc_total_spot_price(&balances).unwrap(), _1, CENT);
    }

    #[test]
    fn calc_arbitrage_amount_mint_sell_errors_if_asset_balance_is_missing() {
        let pool = construct_pool(None, vec![_3, _1, _1, _1]);
        let balances = collect_balances_into_map(pool.assets[..2].into(), vec![_1; 3]);
        assert_eq!(
            pool.calc_arbitrage_amount_mint_sell(&balances, usize::MAX),
            Err("Asset balance missing from BTreeMap"),
        );
    }

    fn construct_pool<Balance>(
        swap_fee: Option<Balance>,
        weights: Vec<u128>,
    ) -> Pool<Balance, MarketId> {
        let fake_market_id = 0;
        let assets = (0..weights.len())
            .map(|i| Asset::CategoricalOutcome(fake_market_id, i as u16))
            .collect::<Vec<_>>();
        let total_weight = weights.iter().sum();
        let weights =
            assets.clone().into_iter().zip(weights.into_iter()).collect::<BTreeMap<_, _>>();
        Pool {
            assets: assets.clone(),
            base_asset: assets[0],
            market_id: 0u8.into(),
            pool_status: PoolStatus::Active, // Doesn't play any role.
            scoring_rule: ScoringRule::CPMM,
            swap_fee,
            total_subsidy: None,
            total_weight: Some(total_weight),
            weights: Some(weights),
        }
    }

    fn collect_balances_into_map<Balance>(
        assets: Vec<Asset<MarketId>>,
        balances: Vec<Balance>,
    ) -> BTreeMap<Asset<MarketId>, Balance> {
        assets.into_iter().zip(balances.into_iter()).collect::<BTreeMap<_, _>>()
    }
}
