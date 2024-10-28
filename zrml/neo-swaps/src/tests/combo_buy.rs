// Copyright 2023-2024 Forecasting Technologies LTD.
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
use zeitgeist_primitives::types::Asset::CategoricalOutcome;

// Example taken from
// https://docs.gnosis.io/conditionaltokens/docs/introduction3/#an-example-with-lmsr
#[test]
fn combo_buy_works() {
    ExtBuilder::default().build().execute_with(|| {
        let liquidity = _10;
        let spot_prices = vec![_1_2, _1_2];
        let swap_fee = CENT;
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Categorical(2),
            liquidity,
            spot_prices.clone(),
            swap_fee,
        );
        let pool = Pools::<Runtime>::get(market_id).unwrap();
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
            market_id,
            2,
            buy.clone(),
            sell.clone(),
            amount_in,
            0,
        ));
        let pool = Pools::<Runtime>::get(market_id).unwrap();
        let expected_swap_amount_out = 58496250072;
        let expected_amount_in_minus_fees = _10 + 1; // Note: This is 1 Pennock off of the correct result.
        let expected_reserves = vec![
            pool_outcomes_before[0] - expected_swap_amount_out,
            pool_outcomes_before[0] + expected_amount_in_minus_fees,
        ];
        assert_pool_state!(
            market_id,
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
                pool_id: market_id,
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
    1000 * _1,
    vec![1_250_000_000; 8],
    vec![0, 2, 5, 6, 7],
    vec![],
    vec![1, 3, 4],
    5_102_040_816_326,
    6_576_234_413_776,
    5_000_000_000_000,
    vec![
        8_423_765_586_224,
        1500 * _1,
        8_423_765_586_224,
        1500 * _1,
        1500 * _1,
        8_423_765_586_224,
        8_423_765_586_224,
        8_423_765_586_224,
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
    51_020_408_163
)]
fn combo_buy_works_multi_market(
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
    expected_fees: u128,
) {
    ExtBuilder::default().build().execute_with(|| {
        let asset_count = spot_prices.len() as u16;
        let swap_fee = CENT;
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Categorical(asset_count),
            liquidity,
            spot_prices.clone(),
            swap_fee,
        );
        let sentinel = 123_456_789;
        assert_ok!(AssetManager::deposit(BASE_ASSET, &BOB, amount_in + sentinel));

        let pool = Pools::<Runtime>::get(market_id).unwrap();
        let expected_liquidity = pool.liquidity_parameter;

        let buy: Vec<_> =
            buy_indices.iter().map(|&i| Asset::CategoricalOutcome(market_id, i)).collect();
        let keep: Vec<_> =
            keep_indices.iter().map(|&i| Asset::CategoricalOutcome(market_id, i)).collect();
        let sell: Vec<_> =
            sell_indices.iter().map(|&i| Asset::CategoricalOutcome(market_id, i)).collect();
        assert_ok!(NeoSwaps::combo_buy(
            RuntimeOrigin::signed(BOB),
            market_id,
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
            market_id,
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
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Scalar(0..=1),
            _10,
            vec![_1_2, _1_2],
            CENT,
        );
        assert_noop!(
            NeoSwaps::combo_buy(
                RuntimeOrigin::signed(BOB),
                market_id,
                1,
                vec![Asset::ScalarOutcome(market_id, ScalarPosition::Long)],
                vec![Asset::ScalarOutcome(market_id, ScalarPosition::Short)],
                _1,
                0
            ),
            Error::<Runtime>::IncorrectAssetCount
        );
    });
}

#[test]
fn combo_buy_fails_on_market_not_found() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Scalar(0..=1),
            _10,
            vec![_1_2, _1_2],
            CENT,
        );
        Markets::<Runtime>::remove(market_id);
        assert_noop!(
            NeoSwaps::combo_buy(
                RuntimeOrigin::signed(BOB),
                market_id,
                2,
                vec![Asset::ScalarOutcome(market_id, ScalarPosition::Long)],
                vec![Asset::ScalarOutcome(market_id, ScalarPosition::Short)],
                _1,
                0
            ),
            zrml_market_commons::Error::<Runtime>::MarketDoesNotExist,
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
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Scalar(0..=1),
            _10,
            vec![_1_2, _1_2],
            CENT,
        );
        MarketCommons::mutate_market(&market_id, |market| {
            market.status = market_status;
            Ok(())
        })
        .unwrap();
        assert_noop!(
            NeoSwaps::combo_buy(
                RuntimeOrigin::signed(BOB),
                market_id,
                2,
                vec![Asset::ScalarOutcome(market_id, ScalarPosition::Long)],
                vec![Asset::ScalarOutcome(market_id, ScalarPosition::Short)],
                _1,
                0
            ),
            Error::<Runtime>::MarketNotActive,
        );
    });
}

#[test]
fn combo_buy_fails_on_pool_not_found() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id =
            create_market(ALICE, BASE_ASSET, MarketType::Scalar(0..=1), ScoringRule::AmmCdaHybrid);
        assert_noop!(
            NeoSwaps::combo_buy(
                RuntimeOrigin::signed(BOB),
                market_id,
                2,
                vec![Asset::ScalarOutcome(market_id, ScalarPosition::Long)],
                vec![Asset::ScalarOutcome(market_id, ScalarPosition::Short)],
                _1,
                0
            ),
            Error::<Runtime>::PoolNotFound,
        );
    });
}

#[test]
fn combo_buy_fails_on_insufficient_funds() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Scalar(0..=1),
            _10,
            vec![_1_2, _1_2],
            CENT,
        );
        let amount_in = _10;
        assert_ok!(AssetManager::deposit(BASE_ASSET, &BOB, amount_in - 1));
        assert_noop!(
            NeoSwaps::combo_buy(
                RuntimeOrigin::signed(BOB),
                market_id,
                2,
                vec![Asset::ScalarOutcome(market_id, ScalarPosition::Long)],
                vec![Asset::ScalarOutcome(market_id, ScalarPosition::Short)],
                amount_in,
                0,
            ),
            orml_currencies::Error::<Runtime>::BalanceTooLow,
        );
    });
}

#[test]
fn combo_buy_fails_on_amount_out_below_min() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Scalar(0..=1),
            _10,
            vec![_1_2, _1_2],
            CENT,
        );
        let amount_in = _1;
        assert_ok!(AssetManager::deposit(BASE_ASSET, &BOB, amount_in));
        // Buying 1 at price of .5 will return less than 2 outcomes due to slippage.
        assert_noop!(
            NeoSwaps::combo_buy(
                RuntimeOrigin::signed(BOB),
                market_id,
                2,
                vec![Asset::ScalarOutcome(market_id, ScalarPosition::Long)],
                vec![Asset::ScalarOutcome(market_id, ScalarPosition::Short)],
                amount_in,
                _2,
            ),
            Error::<Runtime>::AmountOutBelowMin,
        );
    });
}

#[test_case(vec![0], vec![0]; "overlap")]
#[test_case(vec![], vec![0, 1]; "empty_buy")]
#[test_case(vec![2, 3], vec![]; "empty_sell")]
#[test_case(vec![0, 2, 3], vec![1, 3, 4]; "overlap2")]
#[test_case(vec![0, 1, 3, 1], vec![2]; "duplicate_buy")]
#[test_case(vec![0, 1, 3], vec![4, 2, 4]; "duplicate_sell")]
#[test_case(vec![999], vec![0, 1, 2, 3, 4]; "out_of_bounds_buy")]
#[test_case(vec![0, 1, 3], vec![999]; "out_of_bounds_sell")]
fn combo_buy_fails_on_invalid_partition(indices_buy: Vec<u16>, indices_sell: Vec<u16>) {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Categorical(5),
            _10,
            vec![_1_5, _1_5, _1_5, _1_5, _1_5],
            CENT,
        );
        let amount_in = _1;
        assert_ok!(AssetManager::deposit(BASE_ASSET, &BOB, amount_in));

        let buy = indices_buy.into_iter().map(|i| CategoricalOutcome(market_id, i)).collect();
        let sell = indices_sell.into_iter().map(|i| CategoricalOutcome(market_id, i)).collect();

        // Buying 1 at price of .5 will return less than 2 outcomes due to slippage.
        assert_noop!(
            NeoSwaps::combo_buy(RuntimeOrigin::signed(BOB), market_id, 5, buy, sell, amount_in, 0),
            Error::<Runtime>::InvalidPartition,
        );
    });
}

#[test]
fn combo_buy_fails_on_spot_price_slipping_too_low() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Categorical(5),
            _10,
            vec![_1_5, _1_5, _1_5, _1_5, _1_5],
            CENT,
        );
        let amount_in = _100;

        assert_ok!(AssetManager::deposit(BASE_ASSET, &BOB, amount_in));

        let buy = [0, 1, 2, 3].into_iter().map(|i| CategoricalOutcome(market_id, i)).collect();
        let sell = [4].into_iter().map(|i| CategoricalOutcome(market_id, i)).collect();

        assert_noop!(
            NeoSwaps::combo_buy(RuntimeOrigin::signed(BOB), market_id, 5, buy, sell, amount_in, 0),
            Error::<Runtime>::NumericalLimits(NumericalLimitsError::SpotPriceSlippedTooLow),
        );
    });
}

#[test]
fn combo_buy_fails_on_spot_price_slipping_too_high() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Categorical(5),
            _10,
            vec![_1_5, _1_5, _1_5, _1_5, _1_5],
            CENT,
        );
        let amount_in = _100;

        assert_ok!(AssetManager::deposit(BASE_ASSET, &BOB, amount_in));

        let buy = [0].into_iter().map(|i| CategoricalOutcome(market_id, i)).collect();
        let sell = [1, 2, 3, 4].into_iter().map(|i| CategoricalOutcome(market_id, i)).collect();

        assert_noop!(
            NeoSwaps::combo_buy(RuntimeOrigin::signed(BOB), market_id, 5, buy, sell, amount_in, 0),
            Error::<Runtime>::NumericalLimits(NumericalLimitsError::SpotPriceSlippedTooHigh),
        );
    });
}

#[test]
fn combo_buy_fails_on_large_buy() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Categorical(5),
            _10,
            vec![_1_5, _1_5, _1_5, _1_5, _1_5],
            CENT,
        );
        let amount_in = 100 * _100;

        assert_ok!(AssetManager::deposit(BASE_ASSET, &BOB, amount_in));

        let buy = [0].into_iter().map(|i| CategoricalOutcome(market_id, i)).collect();
        let sell = [1, 2].into_iter().map(|i| CategoricalOutcome(market_id, i)).collect();

        assert_noop!(
            NeoSwaps::combo_buy(RuntimeOrigin::signed(BOB), market_id, 5, buy, sell, amount_in, 0),
            Error::<Runtime>::MathError,
        );
    });
}
