#![no_main]

mod common;

use arbitrary::{Arbitrary, Result as ArbitraryResult, Unstructured};
use libfuzzer_sys::fuzz_target;
use orml_traits::currency::MultiCurrency;
use rand::seq::SliceRandom;
use zeitgeist_primitives::{
    constants::base_multiples::*,
    traits::MarketCommonsPalletApi,
    types::{Asset, MarketType},
};
use zrml_neo_swaps::{
    mock::{ExtBuilder, NeoSwaps, Runtime, RuntimeOrigin},
    AccountIdOf, BalanceOf, Config, MarketIdOf, MAX_SPOT_PRICE, MIN_SPOT_PRICE,
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
    sell: Vec<usize>,
    amount_in: BalanceOf<Runtime>,
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
        let buy_len = u.int_in_range(1..=asset_count_usize - 1)?;
        let buy = indices[0..buy_len].to_vec();
        let sell = indices[buy_len..asset_count_usize].to_vec();

        let amount_in = u.int_in_range(_1..=_100)?;
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
            sell,
            amount_in,
            min_amount_out,
        };

        Ok(params)
    }
}

fuzz_target!(|params: ComboBuyFuzzParams| {
    let mut ext = ExtBuilder::default().build();

    ext.execute_with(|| {
        // We create the required markets and deposit enough funds for the user.
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
            100 * params.amount_in,
        )
        .unwrap();

        // Create a pool to trade on.
        NeoSwaps::deploy_combinatorial_pool(
            RuntimeOrigin::signed(params.account_id),
            params.asset_count,
            params.market_ids,
            10 * params.amount_in,
            params.spot_prices,
            params.swap_fee,
            false,
        )
        .unwrap();

        // Convert indices to assets.
        let assets = NeoSwaps::assets(params.pool_id).unwrap();
        let buy = params.buy.into_iter().map(|i| assets[i]).collect();
        let sell = params.sell.into_iter().map(|i| assets[i]).collect();

        let _ = NeoSwaps::combo_buy(
            RuntimeOrigin::signed(params.account_id),
            params.pool_id,
            params.asset_count,
            buy,
            sell,
            params.amount_in,
            params.min_amount_out,
        );
    });

    let _ = ext.commit_all();
});
