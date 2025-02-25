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
use zeitgeist_primitives::{
    constants::base_multiples::*,
    traits::{CombinatorialTokensFuel, MarketCommonsPalletApi},
    types::{Asset, MarketType},
};
use zrml_neo_swaps::{
    mock::{ExtBuilder, NeoSwaps, Runtime, RuntimeOrigin},
    AccountIdOf, BalanceOf, Config, FuelOf, MarketIdOf, COMBO_MAX_SPOT_PRICE, COMBO_MIN_SPOT_PRICE,
    MIN_SWAP_FEE,
};

#[derive(Debug)]
struct DeployCombinatorialPoolFuzzParams {
    account_id: AccountIdOf<Runtime>,
    asset_count: u16,
    market_ids: Vec<MarketIdOf<Runtime>>,
    category_counts: Vec<u16>,
    amount: BalanceOf<Runtime>,
    spot_prices: Vec<BalanceOf<Runtime>>,
    swap_fee: BalanceOf<Runtime>,
    fuel: FuelOf<Runtime>,
}

impl<'a> Arbitrary<'a> for DeployCombinatorialPoolFuzzParams {
    fn arbitrary(u: &mut Unstructured<'a>) -> ArbitraryResult<Self> {
        let account_id = u128::arbitrary(u)?;

        let min_market_ids = 1;
        let max_market_ids = 16;
        let market_ids_len: usize = u.int_in_range(min_market_ids..=max_market_ids)?;
        let market_ids =
            (0..market_ids_len).map(|x| (x as u32).into()).collect::<Vec<MarketIdOf<Runtime>>>();

        let min_category_count = 2;
        let max_category_count = 16;
        let mut category_counts = vec![];
        for _ in market_ids.iter() {
            let category_count = u.int_in_range(min_category_count..=max_category_count)? as u16;
            category_counts.push(category_count);
        }

        let amount = Arbitrary::arbitrary(u)?;

        let asset_count: u16 = category_counts.iter().product();
        let asset_count_usize = asset_count as usize;

        // Create arbitrary spot price vector by creating a vector of `MinSpotPrice` and then adding
        // value to them in increments until a total spot price of one is reached. It's possible
        // that this results in invalid spot prices, for example if `total_assets` is too large.
        let mut spot_prices = vec![COMBO_MIN_SPOT_PRICE; asset_count_usize];
        let increment = COMBO_MIN_SPOT_PRICE;
        while spot_prices.iter().sum::<u128>() < _1 {
            let index = u.int_in_range(0..=asset_count_usize - 1)?;
            if spot_prices[index] < COMBO_MAX_SPOT_PRICE {
                spot_prices[index] += increment;
            }
        }

        let swap_fee = u.int_in_range(MIN_SWAP_FEE..=<Runtime as Config>::MaxSwapFee::get())?;

        let fuel = FuelOf::<Runtime>::from_total(u.int_in_range(1..=100)?);

        let params = DeployCombinatorialPoolFuzzParams {
            account_id,
            asset_count: asset_count as u16,
            market_ids,
            category_counts,
            amount,
            spot_prices,
            swap_fee,
            fuel,
        };

        Ok(params)
    }
}

fuzz_target!(|params: DeployCombinatorialPoolFuzzParams| {
    let mut ext = ExtBuilder::default().build();

    ext.execute_with(|| {
        // We create the required markets and deposit enough funds for the user.
        let collateral = Asset::Ztg;
        for (&market_id, &category_count) in
            params.market_ids.iter().zip(params.category_counts.iter())
        {
            let market = common::market::<Runtime>(
                market_id,
                collateral,
                MarketType::Categorical(category_count),
            );
            <<Runtime as Config>::MarketCommons as MarketCommonsPalletApi>::push_market(market)
                .unwrap();
        }
        <<Runtime as Config>::MultiCurrency>::deposit(
            collateral,
            &params.account_id,
            params.amount,
        )
        .unwrap();

        let _ = NeoSwaps::deploy_combinatorial_pool(
            RuntimeOrigin::signed(params.account_id),
            params.asset_count,
            params.market_ids,
            params.amount,
            params.spot_prices,
            params.swap_fee,
            params.fuel,
        );
    });

    let _ = ext.commit_all();
});
