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

#![no_main]

mod common;

use arbitrary::{Arbitrary, Result as ArbitraryResult, Unstructured};
use libfuzzer_sys::fuzz_target;
use orml_traits::currency::MultiCurrency;
use rand::seq::SliceRandom;
use sp_runtime::traits::Zero;
use zeitgeist_primitives::{
    constants::base_multiples::*,
    traits::{CombinatorialTokensFuel, MarketCommonsPalletApi},
    types::{Asset, MarketType},
};
use zrml_neo_swaps::{
    mock::{ExtBuilder, NeoSwaps, Runtime, RuntimeOrigin},
    AccountIdOf, BalanceOf, Config, FuelOf, MarketIdOf, MAX_SPOT_PRICE, MIN_SPOT_PRICE,
    MIN_SWAP_FEE,
};

#[derive(Debug)]
struct ComboBuyFuzzParams {
    account_id: AccountIdOf<Runtime>,
    pool_id: <Runtime as Config>::PoolId,
    market_ids: Vec<MarketIdOf<Runtime>>,
    spot_prices: Vec<BalanceOf<Runtime>>,
    swap_fee: BalanceOf<Runtime>,
    category_counts: Vec<u16>,
    asset_count: u16,
    buy: Vec<usize>,
    keep: Vec<usize>,
    sell: Vec<usize>,
    amount_buy: BalanceOf<Runtime>,
    amount_keep: BalanceOf<Runtime>,
    min_amount_out: BalanceOf<Runtime>,
}

impl<'a> Arbitrary<'a> for ComboBuyFuzzParams {
    fn arbitrary(u: &mut Unstructured<'a>) -> ArbitraryResult<Self> {
        let account_id = u128::arbitrary(u)?;
        let pool_id = 0;
        let market_ids = vec![0, 1, 2];

        let min_category_count = 2;
        let max_category_count = 16;
        let mut category_counts = vec![];
        for _ in market_ids.iter() {
            // We're just assuming three markets here!
            let category_count = u.int_in_range(min_category_count..=max_category_count)? as u16;
            category_counts.push(category_count);
        }

        let asset_count = category_counts.iter().product();
        let asset_count_usize = asset_count as usize;

        // Create arbitrary spot price vector by creating a vector of `MinSpotPrice` and then adding
        // value to them in increments until a total spot price of one is reached. It's possible
        // that this results in invalid spot prices, for example if `total_assets` is too large.
        let mut spot_prices = vec![MIN_SPOT_PRICE; asset_count_usize];
        let increment = MIN_SPOT_PRICE;
        while spot_prices.iter().sum::<u128>() < _1 {
            let index = u.int_in_range(0..=asset_count_usize - 1)?;
            if spot_prices[index] < MAX_SPOT_PRICE {
                spot_prices[index] += increment;
            }
        }

        let swap_fee = u.int_in_range(MIN_SWAP_FEE..=<Runtime as Config>::MaxSwapFee::get())?;

        // Shuffle 0..asset_count_usize and then obtain `buy` and `sell` from the result.
        let mut indices: Vec<usize> = (0..asset_count_usize).collect();
        for i in (1..indices.len()).rev() {
            let j = u.int_in_range(0..=i)?;
            indices.swap(i, j);
        }

        // This isn't perfectly random, but biased towards producing larger `buy` sets.
        let buy_len = u.int_in_range(1..=asset_count_usize - 1)?;
        let keep_len = u.int_in_range(0..=asset_count_usize - 1 - buy_len)?;
        let buy = indices[0..buy_len].to_vec();
        let keep = indices[buy_len..buy_len + keep_len].to_vec();
        let sell = indices[buy_len + keep_len..asset_count_usize].to_vec();

        let amount_buy = u.int_in_range(_1..=_100)?;
        let amount_keep =
            if keep.is_empty() { Zero::zero() } else { u.int_in_range(_1..=amount_buy)? };

        let min_amount_out = Arbitrary::arbitrary(u)?;

        let params = ComboBuyFuzzParams {
            account_id,
            pool_id,
            market_ids,
            spot_prices,
            swap_fee,
            category_counts,
            asset_count,
            buy,
            keep,
            sell,
            amount_buy,
            amount_keep,
            min_amount_out,
        };

        Ok(params)
    }
}

fuzz_target!(|params: ComboBuyFuzzParams| {
    let mut ext = ExtBuilder::default().build();

    ext.execute_with(|| {
        // We create the required markets and deposit collateral in the user's account.
        let collateral = Asset::Ztg;
        for (market_id, &category_count) in params.category_counts.iter().enumerate() {
            let market = common::market::<Runtime>(
                market_id as u128,
                collateral,
                MarketType::Categorical(category_count),
            );
            <<Runtime as Config>::MarketCommons as MarketCommonsPalletApi>::push_market(market)
                .unwrap();
        }
        <<Runtime as Config>::MultiCurrency>::deposit(
            collateral,
            &params.account_id,
            100 * params.amount_buy,
        )
        .unwrap();

        // Create a pool to trade on.
        NeoSwaps::deploy_combinatorial_pool(
            RuntimeOrigin::signed(params.account_id),
            params.asset_count,
            params.market_ids,
            10 * params.amount_buy,
            params.spot_prices,
            params.swap_fee,
            FuelOf::<Runtime>::from_total(16),
        )
        .unwrap();

        // Convert indices to assets an deposit funds for the user.
        let assets = NeoSwaps::assets(params.pool_id).unwrap();
        for &asset in assets.iter() {
            <<Runtime as Config>::MultiCurrency>::deposit(
                asset,
                &params.account_id,
                params.amount_buy,
            )
            .unwrap();
        }

        let buy = params.buy.into_iter().map(|i| assets[i]).collect();
        let keep = params.keep.into_iter().map(|i| assets[i]).collect();
        let sell = params.sell.into_iter().map(|i| assets[i]).collect();

        let _ = NeoSwaps::combo_sell(
            RuntimeOrigin::signed(params.account_id),
            params.pool_id,
            params.asset_count,
            buy,
            keep,
            sell,
            params.amount_buy,
            params.amount_keep,
            params.min_amount_out,
        );
    });

    let _ = ext.commit_all();
});
