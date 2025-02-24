// Copyright 2023-2025 Forecasting Technologies LTD.
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

use super::*;
use test_case::test_case;

// Example taken from
// https://github.com/gnosis/conditional-tokens-docs/blob/e73aa18ab82446049bca61df31fc88efd3cdc5cc/docs/intro3.md?plain=1#L78-L88
#[test]
fn combo_buy_works() {
    ExtBuilder::default().build().execute_with(|| {
        let liquidity = _10;
        let spot_prices = vec![_1_2, _1_2];
        let swap_fee = CENT;
        let (_, pool_id) = create_markets_and_deploy_combinatorial_pool(
            ALICE,
            BASE_ASSET,
            vec![MarketType::Categorical(2)],
            liquidity,
            spot_prices.clone(),
            swap_fee,
        );
        let pool = Pools::<Runtime>::get(pool_id).unwrap();
        let total_fee_percentage = swap_fee + EXTERNAL_FEES;
        let amount_in_minus_fees = _10;
        let amount_in = amount_in_minus_fees.bdiv(_1 - total_fee_percentage).unwrap(); // This is exactly _10 after deducting fees.
        let expected_fees = amount_in - amount_in_minus_fees;
        let expected_swap_fee_amount = expected_fees / 2;
        let expected_external_fee_amount = expected_fees / 2;
        let pool_outcomes_before: Vec<_> =
            pool.assets().iter().map(|a| pool.reserve_of(a).unwrap()).collect();
        let liquidity_parameter_before = pool.liquidity_parameter;
        let buy = vec![pool.assets()[0]];
        let sell = pool.assets_complement(&buy);
        assert_ok!(AssetManager::deposit(BASE_ASSET, &BOB, amount_in));
        // Deposit some stuff in the pool account to check that the pools `reserves` fields tracks
        // the reserve correctly.
        assert_ok!(AssetManager::deposit(sell[0], &pool.account_id, _100));
        assert_ok!(NeoSwaps::combo_buy(
            RuntimeOrigin::signed(BOB),
            pool_id,
            2,
            buy.clone(),
            sell.clone(),
            amount_in,
            0,
        ));
        let pool = Pools::<Runtime>::get(pool_id).unwrap();
        let expected_swap_amount_out = 58496250072;
        let expected_amount_in_minus_fees = _10 + 1; // Note: This is 1 Pennock off of the correct result.
        let expected_reserves = vec![
            pool_outcomes_before[0] - expected_swap_amount_out,
            pool_outcomes_before[0] + expected_amount_in_minus_fees,
        ];
        assert_pool_state!(
            pool_id,
            expected_reserves,
            vec![_3_4, _1_4],
            liquidity_parameter_before,
            create_b_tree_map!({ ALICE => liquidity }),
            expected_swap_fee_amount,
        );
        let expected_amount_out = expected_swap_amount_out + expected_amount_in_minus_fees;
        assert_balance!(BOB, BASE_ASSET, 0);
        assert_balance!(BOB, buy[0], expected_amount_out);
        assert_balance!(
            pool.account_id,
            BASE_ASSET,
            expected_swap_fee_amount + AssetManager::minimum_balance(pool.collateral)
        );
        assert_balance!(FEE_ACCOUNT, BASE_ASSET, expected_external_fee_amount);
        System::assert_last_event(
            Event::ComboBuyExecuted {
                who: BOB,
                pool_id,
                buy,
                sell,
                amount_in,
                amount_out: expected_amount_out,
                swap_fee_amount: expected_swap_fee_amount,
                external_fee_amount: expected_external_fee_amount,
            }
            .into(),
        );
    });
}

#[test_case(
    vec![MarketType::Categorical(5)],
    333 * _1,
    vec![10 * CENT, 30 * CENT, 25 * CENT, 13 * CENT, 22 * CENT],
    vec![0, 2],
    vec![3],
    vec![1, 4],
    102_040_816_327,
    236_865_613_849,
    100_000_000_001,
    vec![3193134386152, 1841186221785, 1867994157274, 2950568636818, 2289732472863],
    vec![1_099_260_911, 2_799_569_315, 2_748_152_277, 1_300_000_000, 2_053_017_497],
    1_020_408_163
)]
#[test_case(
    vec![MarketType::Categorical(5)],
    _100,
    vec![80 * CENT, 5 * CENT, 5 * CENT, 5 * CENT, 5 * CENT],
    vec![4],
    vec![1, 2, 3],
    vec![0],
    336_734_693_877,
    1_131_842_030_026,
    329_999_999_999,
    vec![404_487_147_360, _100, _100, _100, 198_157_969_973],
    vec![2_976_802_957, 5 * CENT, 5 * CENT, 5 * CENT, 5_523_197_043],
    3_367_346_939
)]
#[test_case(
    vec![MarketType::Categorical(2), MarketType::Categorical(2), MarketType::Scalar(0..=1)],
    1000 * _1,
    vec![1_250_000_000; 8],
    vec![0, 2, 5, 6, 7],
    vec![],
    vec![1, 3, 4],
    5_208_333_333_333,
    6_576_234_413_778,
    // keep_indices vector is empty anyways, so `expected_amount_out_keep` amount has no effect
    0,
    vec![
        8_423_765_586_223,
        1500 * _1 + 1,
        8_423_765_586_223,
        1500 * _1 + 1,
        1500 * _1 + 1,
        8_423_765_586_223,
        8_423_765_586_223,
        8_423_765_586_223,
    ],
    vec![
        1_734_834_957,
        441_941_738,
        1_734_834_957,
        441_941_738,
        441_941_738,
        1_734_834_957,
        1_734_834_957,
        1_734_834_957,
    ],
    52_083_333_333
)]
fn combo_buy_works_multi_market(
    market_types: Vec<MarketType>,
    liquidity: u128,
    spot_prices: Vec<u128>,
    buy_indices: Vec<u16>,
    keep_indices: Vec<u16>,
    sell_indices: Vec<u16>,
    amount_in: u128,
    expected_amount_out_buy: u128,
    expected_amount_out_keep: u128,
    expected_reserves: Vec<u128>,
    expected_spot_prices: Vec<u128>,
    expected_fees: u128, // pool fees, not market fees
) {
    ExtBuilder::default().build().execute_with(|| {
        let asset_count = spot_prices.len() as u16;
        let swap_fee = CENT;
        let (_, pool_id) = create_markets_and_deploy_combinatorial_pool(
            ALICE,
            BASE_ASSET,
            market_types,
            liquidity,
            spot_prices.clone(),
            swap_fee,
        );
        let sentinel = 123_456_789;
        assert_ok!(AssetManager::deposit(BASE_ASSET, &BOB, amount_in + sentinel));

        let pool = Pools::<Runtime>::get(pool_id).unwrap();
        let expected_liquidity = pool.liquidity_parameter;

        let buy: Vec<_> = buy_indices.iter().map(|&i| pool.assets()[i as usize]).collect();
        let keep: Vec<_> = keep_indices.iter().map(|&i| pool.assets()[i as usize]).collect();
        let sell: Vec<_> = sell_indices.iter().map(|&i| pool.assets()[i as usize]).collect();

        assert_ok!(NeoSwaps::combo_buy(
            RuntimeOrigin::signed(BOB),
            pool_id,
            asset_count,
            buy.clone(),
            sell.clone(),
            amount_in,
            0,
        ));

        assert_balance!(BOB, BASE_ASSET, sentinel);
        for &asset in buy.iter() {
            assert_balance!(BOB, asset, expected_amount_out_buy);
        }
        for &asset in keep.iter() {
            assert_balance!(BOB, asset, expected_amount_out_keep);
        }
        for &asset in sell.iter() {
            assert_balance!(BOB, asset, 0);
        }

        assert_pool_state!(
            pool_id,
            expected_reserves,
            expected_spot_prices,
            expected_liquidity,
            create_b_tree_map!({ ALICE => liquidity }),
            expected_fees,
        );
    });
}

#[test]
fn combo_buy_fails_on_incorrect_asset_count() {
    ExtBuilder::default().build().execute_with(|| {
        let (_, pool_id) = create_markets_and_deploy_combinatorial_pool(
            ALICE,
            BASE_ASSET,
            vec![MarketType::Categorical(2), MarketType::Scalar(0..=1)],
            _10,
            vec![_1_4, _1_4, _1_4, _1_4],
            CENT,
        );
        let pool = <Pallet<Runtime> as PoolStorage>::get(pool_id).unwrap();
        let assets = pool.assets();
        assert_noop!(
            NeoSwaps::combo_buy(
                RuntimeOrigin::signed(BOB),
                pool_id,
                3,
                vec![assets[0]],
                vec![assets[1]],
                _1,
                0
            ),
            Error::<Runtime>::IncorrectAssetCount
        );
    });
}

#[test]
fn combo_buy_fails_on_pool_not_found() {
    ExtBuilder::default().build().execute_with(|| {
        let (_, pool_id) = create_markets_and_deploy_combinatorial_pool(
            ALICE,
            BASE_ASSET,
            vec![MarketType::Categorical(2), MarketType::Scalar(0..=1)],
            _10,
            vec![_1_4, _1_4, _1_4, _1_4],
            CENT,
        );
        let pool = <Pallet<Runtime> as PoolStorage>::get(pool_id).unwrap();
        let assets = pool.assets();
        assert_noop!(
            NeoSwaps::combo_buy(
                RuntimeOrigin::signed(BOB),
                1,
                4,
                vec![assets[0]],
                vec![assets[1]],
                _1,
                0
            ),
            Error::<Runtime>::PoolNotFound,
        );
    });
}

#[test_case(MarketStatus::Proposed)]
#[test_case(MarketStatus::Closed)]
#[test_case(MarketStatus::Reported)]
#[test_case(MarketStatus::Disputed)]
#[test_case(MarketStatus::Resolved)]
fn combo_buy_fails_on_inactive_market(market_status: MarketStatus) {
    ExtBuilder::default().build().execute_with(|| {
        let (market_ids, pool_id) = create_markets_and_deploy_combinatorial_pool(
            ALICE,
            BASE_ASSET,
            vec![MarketType::Categorical(2), MarketType::Scalar(0..=1)],
            _10,
            vec![_1_4, _1_4, _1_4, _1_4],
            CENT,
        );
        let pool = <Pallet<Runtime> as PoolStorage>::get(pool_id).unwrap();
        let assets = pool.assets();
        MarketCommons::mutate_market(&market_ids[1], |market| {
            market.status = market_status;
            Ok(())
        })
        .unwrap();
        assert_noop!(
            NeoSwaps::combo_buy(
                RuntimeOrigin::signed(BOB),
                pool_id,
                4,
                vec![assets[0]],
                vec![assets[1]],
                _1,
                0
            ),
            Error::<Runtime>::MarketNotActive,
        );
    });
}

#[test]
fn combo_buy_fails_on_insufficient_funds() {
    ExtBuilder::default().build().execute_with(|| {
        let (_, pool_id) = create_markets_and_deploy_combinatorial_pool(
            ALICE,
            BASE_ASSET,
            vec![MarketType::Categorical(2), MarketType::Scalar(0..=1)],
            _10,
            vec![_1_4, _1_4, _1_4, _1_4],
            CENT,
        );
        let pool = <Pallet<Runtime> as PoolStorage>::get(pool_id).unwrap();
        let assets = pool.assets();
        let amount_in = _10;
        assert_ok!(AssetManager::deposit(BASE_ASSET, &BOB, amount_in - 1));

        #[cfg(feature = "parachain")]
        let expected_error = orml_tokens::Error::<Runtime>::BalanceTooLow;
        #[cfg(not(feature = "parachain"))]
        let expected_error = orml_currencies::Error::<Runtime>::BalanceTooLow;

        assert_noop!(
            NeoSwaps::combo_buy(
                RuntimeOrigin::signed(BOB),
                pool_id,
                4,
                vec![assets[0]],
                vec![assets[1]],
                amount_in,
                0,
            ),
            expected_error
        );
    });
}

#[test]
fn combo_buy_fails_on_amount_out_below_min() {
    ExtBuilder::default().build().execute_with(|| {
        let (_, pool_id) = create_markets_and_deploy_combinatorial_pool(
            ALICE,
            BASE_ASSET,
            vec![MarketType::Categorical(2), MarketType::Scalar(0..=1)],
            100_000_000 * _100, // Massive liquidity to keep slippage low.
            vec![_1_4, _1_4, _1_4, _1_4],
            CENT,
        );
        let pool = <Pallet<Runtime> as PoolStorage>::get(pool_id).unwrap();
        let assets = pool.assets();

        let asset_count = 4;
        let buy = assets[0..1].to_vec();
        let sell = assets[1..4].to_vec();
        // amount_in is _1 / 0.97 (i.e. _1 after deducting 3% fees - 1% trading fees, 1% external
        // fees _for each market_)
        let amount_in = 10_309_278_350;

        assert_ok!(AssetManager::deposit(BASE_ASSET, &BOB, amount_in));
        // Buying for 1 at a price of .25 will return less than 4 outcomes due to slippage.
        assert_noop!(
            NeoSwaps::combo_buy(
                RuntimeOrigin::signed(BOB),
                pool_id,
                asset_count,
                buy.clone(),
                sell.clone(),
                amount_in,
                _4,
            ),
            Error::<Runtime>::AmountOutBelowMin,
        );

        // Post OakSecurity audit: Show that the slippage limit is tight.
        assert_ok!(NeoSwaps::combo_buy(
            RuntimeOrigin::signed(BOB),
            pool_id,
            asset_count,
            buy,
            sell,
            amount_in,
            _4 - 33,
        ));
    });
}

#[test_case(vec![0], vec![0]; "overlap")]
#[test_case(vec![], vec![0, 1]; "empty_buy")]
#[test_case(vec![2, 3], vec![]; "empty_sell")]
#[test_case(vec![0, 2, 3], vec![1, 3, 4]; "overlap2")]
#[test_case(vec![0, 1, 3, 1], vec![2]; "duplicate_buy")]
#[test_case(vec![0, 1, 3], vec![4, 2, 4]; "duplicate_sell")]
fn combo_buy_fails_on_invalid_partition(buy_indices: Vec<u16>, sell_indices: Vec<u16>) {
    ExtBuilder::default().build().execute_with(|| {
        let (_, pool_id) = create_markets_and_deploy_combinatorial_pool(
            ALICE,
            BASE_ASSET,
            vec![MarketType::Categorical(5)],
            _10,
            vec![_1_5, _1_5, _1_5, _1_5, _1_5],
            CENT,
        );
        let pool = <Pallet<Runtime> as PoolStorage>::get(pool_id).unwrap();
        let assets = pool.assets();
        let amount_in = _1;
        assert_ok!(AssetManager::deposit(BASE_ASSET, &BOB, amount_in));

        let buy: Vec<_> = buy_indices.iter().map(|&i| assets[i as usize]).collect();
        let sell: Vec<_> = sell_indices.iter().map(|&i| assets[i as usize]).collect();

        assert_noop!(
            NeoSwaps::combo_buy(RuntimeOrigin::signed(BOB), pool_id, 5, buy, sell, amount_in, 0),
            Error::<Runtime>::InvalidPartition,
        );
    });
}

#[test]
fn combo_buy_fails_on_spot_price_slipping_too_low() {
    ExtBuilder::default().build().execute_with(|| {
        let (_, pool_id) = create_markets_and_deploy_combinatorial_pool(
            ALICE,
            BASE_ASSET,
            vec![MarketType::Categorical(5)],
            _10,
            vec![_1_5, _1_5, _1_5, _1_5, _1_5],
            CENT,
        );
        let amount_in = _100;
        let pool = <Pallet<Runtime> as PoolStorage>::get(pool_id).unwrap();
        let assets = pool.assets();

        assert_ok!(AssetManager::deposit(BASE_ASSET, &BOB, amount_in));

        let buy = assets[0..4].to_vec();
        let sell = vec![assets[4]];

        assert_noop!(
            NeoSwaps::combo_buy(RuntimeOrigin::signed(BOB), pool_id, 5, buy, sell, amount_in, 0),
            Error::<Runtime>::NumericalLimits(NumericalLimitsError::SpotPriceSlippedTooLow),
        );
    });
}

#[test]
fn combo_buy_fails_on_large_buy() {
    ExtBuilder::default().build().execute_with(|| {
        let (_, pool_id) = create_markets_and_deploy_combinatorial_pool(
            ALICE,
            BASE_ASSET,
            vec![MarketType::Categorical(5)],
            _10,
            vec![_1_5, _1_5, _1_5, _1_5, _1_5],
            CENT,
        );
        let amount_in = 100 * _100;
        let pool = <Pallet<Runtime> as PoolStorage>::get(pool_id).unwrap();
        let assets = pool.assets();

        assert_ok!(AssetManager::deposit(BASE_ASSET, &BOB, amount_in));

        let buy = vec![assets[4]];
        let sell = assets[0..2].to_vec();

        assert_noop!(
            NeoSwaps::combo_buy(RuntimeOrigin::signed(BOB), pool_id, 5, buy, sell, amount_in, 0),
            Error::<Runtime>::MathError,
        );
    });
}

#[test]
fn combo_buy_fails_on_invalid_pool_type() {
    ExtBuilder::default().build().execute_with(|| {
        let pool_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Categorical(5),
            _10,
            vec![_1_5, _1_5, _1_5, _1_5, _1_5],
            CENT,
        );

        let amount_in = _1;
        let pool = <Pallet<Runtime> as PoolStorage>::get(pool_id).unwrap();
        let assets = pool.assets();

        assert_ok!(AssetManager::deposit(BASE_ASSET, &BOB, amount_in));

        let buy = vec![assets[4]];
        let sell = assets[0..2].to_vec();

        assert_noop!(
            NeoSwaps::combo_buy(RuntimeOrigin::signed(BOB), pool_id, 5, buy, sell, amount_in, 0),
            Error::<Runtime>::InvalidPoolType,
        );
    });
}
