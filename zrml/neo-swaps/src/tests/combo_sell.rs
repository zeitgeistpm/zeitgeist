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

#[test]
fn combo_sell_works() {
    ExtBuilder::default().build().execute_with(|| {
        let liquidity = _10;
        let spot_prices = vec![_1_4, _3_4];
        let swap_fee = CENT;
        let (_, pool_id) = create_markets_and_deploy_combinatorial_pool(
            ALICE,
            BASE_ASSET,
            vec![MarketType::Scalar(0..=1)],
            liquidity,
            spot_prices.clone(),
            swap_fee,
        );
        let pool = Pools::<Runtime>::get(pool_id).unwrap();
        let amount_buy = _10;
        let amount_keep = 0;
        let liquidity_parameter_before = pool.liquidity_parameter;

        let buy_asset = pool.assets()[1];
        let sell_asset = pool.assets()[0];
        let buy = vec![buy_asset];
        let keep = vec![];
        let sell = vec![sell_asset];

        for &asset in buy.iter() {
            assert_ok!(AssetManager::deposit(asset, &BOB, amount_buy));
        }

        assert_ok!(NeoSwaps::combo_sell(
            RuntimeOrigin::signed(BOB),
            pool_id,
            2,
            buy.clone(),
            keep.clone(),
            sell.clone(),
            amount_buy,
            amount_keep,
            0,
        ));
        let total_fee_percentage = swap_fee + EXTERNAL_FEES;
        let expected_amount_out = 59632253897;
        let expected_fees = total_fee_percentage.bmul(expected_amount_out).unwrap();
        let expected_swap_fee_amount = expected_fees / 2;
        let expected_external_fee_amount = expected_fees - expected_swap_fee_amount;
        let expected_amount_out_minus_fees = expected_amount_out - expected_fees;
        assert_balance!(BOB, BASE_ASSET, expected_amount_out_minus_fees);
        assert_balance!(BOB, buy_asset, 0);
        assert_pool_state!(
            pool_id,
            vec![40367746103, 61119621067],
            [5_714_285_714, 4_285_714_286],
            liquidity_parameter_before,
            create_b_tree_map!({ ALICE => liquidity }),
            expected_swap_fee_amount,
        );
        assert_balance!(
            pool.account_id,
            BASE_ASSET,
            expected_swap_fee_amount + AssetManager::minimum_balance(pool.collateral)
        );
        assert_balance!(FEE_ACCOUNT, BASE_ASSET, expected_external_fee_amount);
        System::assert_last_event(
            Event::ComboSellExecuted {
                who: BOB,
                pool_id,
                buy,
                keep,
                sell,
                amount_buy,
                amount_keep,
                amount_out: expected_amount_out_minus_fees,
                swap_fee_amount: expected_swap_fee_amount,
                external_fee_amount: expected_external_fee_amount,
            }
            .into(),
        );
    });
}

#[test_case(
    vec![MarketType::Categorical(2), MarketType::Categorical(2), MarketType::Scalar(0..=1)],
    1000 * _1,
    vec![1_250_000_000; 8],
    vec![0, 2, 5],
    vec![6, 7],
    vec![1, 3, 4],
    _500,
    _300,
    2_049_142_184_080,
    vec![
        12_865_476_891_584,
        7_865_476_891_584,
        12_865_476_891_584,
        7_865_476_891_584,
        7_865_476_891_584,
        12_865_476_891_584,
        10_865_476_891_584,
        10_865_476_891_584,
    ],
    vec![
        688_861_105,
        1_948_393_435,
        688_861_105,
        1_948_393_435,
        1_948_393_435,
        688_861_105,
        1_044_118_189,
        1_044_118_189,
    ],
    21_345_231_084
)]
#[test_case(
    vec![MarketType::Categorical(3)],
    _321,
    vec![20 * CENT, 30 * CENT, 50 * CENT],
    vec![0, 2],
    vec![],
    vec![1],
    _500,
    0,
    2_012_922_832_062,
    vec![
        6_155_997_110_140,
        347_302_977_256,
        4_328_468_861_556,
    ],
    vec![
        456_610_616,
        8_401_862_845,
        1_141_526_539,
    ],
    20_540_028_899
)]
fn combo_sell_works_multi_market(
    market_types: Vec<MarketType>,
    liquidity: u128,
    spot_prices: Vec<u128>,
    buy_indices: Vec<u16>,
    keep_indices: Vec<u16>,
    sell_indices: Vec<u16>,
    amount_in_buy: u128,
    amount_in_keep: u128,
    expected_amount_out: u128,
    expected_reserves: Vec<u128>,
    expected_spot_prices: Vec<u128>,
    expected_fees: u128,
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

        let pool = <Pallet<Runtime> as PoolStorage>::get(pool_id).unwrap();
        let expected_liquidity = pool.liquidity_parameter;

        let buy: Vec<_> = buy_indices.iter().map(|&i| pool.assets()[i as usize]).collect();
        let keep: Vec<_> = keep_indices.iter().map(|&i| pool.assets()[i as usize]).collect();
        let sell: Vec<_> = sell_indices.iter().map(|&i| pool.assets()[i as usize]).collect();

        for &asset in buy.iter() {
            assert_ok!(AssetManager::deposit(asset, &BOB, amount_in_buy));
        }
        for &asset in keep.iter() {
            assert_ok!(AssetManager::deposit(asset, &BOB, amount_in_keep));
        }

        assert_ok!(NeoSwaps::combo_sell(
            RuntimeOrigin::signed(BOB),
            pool_id,
            asset_count,
            buy.clone(),
            keep.clone(),
            sell.clone(),
            amount_in_buy,
            amount_in_keep,
            0,
        ));

        assert_balance!(BOB, BASE_ASSET, expected_amount_out);
        for asset in pool.assets() {
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
fn combo_sell_fails_on_incorrect_asset_count() {
    ExtBuilder::default().build().execute_with(|| {
        let (_, pool_id) = create_markets_and_deploy_combinatorial_pool(
            ALICE,
            BASE_ASSET,
            vec![MarketType::Categorical(2), MarketType::Categorical(2), MarketType::Scalar(0..=1)],
            _100,
            vec![1_250_000_000; 8],
            CENT,
        );
        let pool = <Pallet<Runtime> as PoolStorage>::get(pool_id).unwrap();

        assert_noop!(
            NeoSwaps::combo_sell(
                RuntimeOrigin::signed(BOB),
                pool_id,
                7,
                vec![pool.assets()[0]],
                vec![],
                vec![pool.assets()[1]],
                _1,
                0,
                0
            ),
            Error::<Runtime>::IncorrectAssetCount
        );
    });
}

#[test]
fn combo_sell_fails_on_pool_not_found() {
    ExtBuilder::default().build().execute_with(|| {
        let (_, pool_id) = create_markets_and_deploy_combinatorial_pool(
            ALICE,
            BASE_ASSET,
            vec![MarketType::Categorical(2), MarketType::Categorical(2), MarketType::Scalar(0..=1)],
            _100,
            vec![1_250_000_000; 8],
            CENT,
        );
        let pool = <Pallet<Runtime> as PoolStorage>::get(pool_id).unwrap();

        assert_noop!(
            NeoSwaps::combo_sell(
                RuntimeOrigin::signed(BOB),
                1,
                2,
                vec![pool.assets()[0]],
                vec![],
                vec![pool.assets()[1]],
                _1,
                0,
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
fn combo_sell_fails_on_inactive_market(market_status: MarketStatus) {
    ExtBuilder::default().build().execute_with(|| {
        let (market_ids, pool_id) = create_markets_and_deploy_combinatorial_pool(
            ALICE,
            BASE_ASSET,
            vec![MarketType::Categorical(2), MarketType::Categorical(2), MarketType::Scalar(0..=1)],
            _100,
            vec![1_250_000_000; 8],
            CENT,
        );
        let pool = <Pallet<Runtime> as PoolStorage>::get(pool_id).unwrap();
        MarketCommons::mutate_market(&market_ids[1], |market| {
            market.status = market_status;
            Ok(())
        })
        .unwrap();
        assert_noop!(
            NeoSwaps::combo_sell(
                RuntimeOrigin::signed(BOB),
                pool_id,
                8,
                vec![pool.assets()[0]],
                vec![],
                vec![pool.assets()[1]],
                _1,
                0,
                0
            ),
            Error::<Runtime>::MarketNotActive,
        );
    });
}

#[test]
fn combo_sell_fails_on_insufficient_funds_with_keep() {
    ExtBuilder::default().build().execute_with(|| {
        let (_, pool_id) = create_markets_and_deploy_combinatorial_pool(
            ALICE,
            BASE_ASSET,
            vec![MarketType::Categorical(2), MarketType::Categorical(2), MarketType::Scalar(0..=1)],
            _100,
            vec![1_250_000_000; 8],
            CENT,
        );
        let pool = <Pallet<Runtime> as PoolStorage>::get(pool_id).unwrap();
        let buy_amount = _10;
        let keep_amount = _9;

        assert_ok!(AssetManager::deposit(pool.assets()[0], &BOB, buy_amount));
        assert_ok!(AssetManager::deposit(pool.assets()[1], &BOB, buy_amount));
        assert_ok!(AssetManager::deposit(pool.assets()[2], &BOB, buy_amount));
        assert_ok!(AssetManager::deposit(pool.assets()[3], &BOB, buy_amount));
        assert_ok!(AssetManager::deposit(pool.assets()[4], &BOB, buy_amount));
        assert_ok!(AssetManager::deposit(pool.assets()[5], &BOB, buy_amount));
        assert_ok!(AssetManager::deposit(pool.assets()[6], &BOB, keep_amount));
        assert_ok!(AssetManager::deposit(pool.assets()[7], &BOB, keep_amount - 1));

        assert_noop!(
            NeoSwaps::combo_sell(
                RuntimeOrigin::signed(BOB),
                pool_id,
                8,
                vec![pool.assets()[0], pool.assets()[2], pool.assets()[4]],
                vec![pool.assets()[6], pool.assets()[7]],
                vec![pool.assets()[1], pool.assets()[3], pool.assets()[5]],
                buy_amount,
                keep_amount,
                0,
            ),
            orml_tokens::Error::<Runtime>::BalanceTooLow,
        );
    });
}

#[test]
fn combo_sell_fails_on_insufficient_funds_sans_keep() {
    ExtBuilder::default().build().execute_with(|| {
        let (_, pool_id) = create_markets_and_deploy_combinatorial_pool(
            ALICE,
            BASE_ASSET,
            vec![MarketType::Categorical(2), MarketType::Categorical(2), MarketType::Scalar(0..=1)],
            _100,
            vec![1_250_000_000; 8],
            CENT,
        );
        let pool = <Pallet<Runtime> as PoolStorage>::get(pool_id).unwrap();
        let amount_in = _10;

        assert_ok!(AssetManager::deposit(pool.assets()[0], &BOB, amount_in));
        assert_ok!(AssetManager::deposit(pool.assets()[1], &BOB, amount_in));
        assert_ok!(AssetManager::deposit(pool.assets()[2], &BOB, amount_in - 1));
        assert_ok!(AssetManager::deposit(pool.assets()[3], &BOB, amount_in));
        assert_ok!(AssetManager::deposit(pool.assets()[4], &BOB, amount_in));
        assert_ok!(AssetManager::deposit(pool.assets()[5], &BOB, amount_in));
        assert_ok!(AssetManager::deposit(pool.assets()[6], &BOB, amount_in));
        assert_ok!(AssetManager::deposit(pool.assets()[7], &BOB, amount_in));

        assert_noop!(
            NeoSwaps::combo_sell(
                RuntimeOrigin::signed(BOB),
                pool_id,
                8,
                vec![pool.assets()[0], pool.assets()[2], pool.assets()[4], pool.assets()[6]],
                vec![],
                vec![pool.assets()[1], pool.assets()[3], pool.assets()[5], pool.assets()[7]],
                amount_in,
                0,
                0,
            ),
            orml_tokens::Error::<Runtime>::BalanceTooLow,
        );
    });
}

#[test]
fn combo_sell_fails_on_amount_out_below_min() {
    ExtBuilder::default().build().execute_with(|| {
        let (_, pool_id) = create_markets_and_deploy_combinatorial_pool(
            ALICE,
            BASE_ASSET,
            vec![MarketType::Categorical(2), MarketType::Categorical(2), MarketType::Scalar(0..=1)],
            _100,
            vec![1_250_000_000; 8],
            CENT,
        );

        let pool = <Pallet<Runtime> as PoolStorage>::get(pool_id).unwrap();
        let amount_in = _10;
        assert_ok!(AssetManager::deposit(pool.assets()[0], &BOB, amount_in));
        assert_ok!(AssetManager::deposit(pool.assets()[1], &BOB, amount_in));
        assert_ok!(AssetManager::deposit(pool.assets()[2], &BOB, amount_in));
        assert_ok!(AssetManager::deposit(pool.assets()[3], &BOB, amount_in));
        assert_ok!(AssetManager::deposit(pool.assets()[4], &BOB, amount_in));
        assert_ok!(AssetManager::deposit(pool.assets()[5], &BOB, amount_in));
        assert_ok!(AssetManager::deposit(pool.assets()[6], &BOB, amount_in));
        assert_ok!(AssetManager::deposit(pool.assets()[7], &BOB, amount_in));

        assert_noop!(
            NeoSwaps::combo_sell(
                RuntimeOrigin::signed(BOB),
                pool_id,
                8,
                vec![pool.assets()[0], pool.assets()[2], pool.assets()[4], pool.assets()[6]],
                vec![],
                vec![pool.assets()[1], pool.assets()[3], pool.assets()[5], pool.assets()[7]],
                amount_in,
                0,
                _10
            ),
            Error::<Runtime>::AmountOutBelowMin,
        );
    });
}

#[test_case(vec![], vec![], vec![2]; "empty_buy")]
#[test_case(vec![0], vec![], vec![]; "empty_sell")]
#[test_case(vec![0, 1], vec![2, 1], vec![3, 4]; "buy_keep_overlap")]
#[test_case(vec![0, 1], vec![2, 4], vec![3, 1]; "buy_sell_overlap")]
#[test_case(vec![0, 1], vec![2, 4], vec![4, 3]; "keep_sell_overlap")]
#[test_case(vec![0, 6, 1, 6], vec![2, 4], vec![5, 3]; "duplicate_buy")]
#[test_case(vec![0, 1], vec![2, 2, 4], vec![5, 3]; "duplicate_keep")]
#[test_case(vec![0, 1], vec![2, 4], vec![5, 3, 6, 6, 6]; "duplicate_sell")]
fn combo_sell_fails_on_invalid_partition(
    buy_indices: Vec<u16>,
    keep_indices: Vec<u16>,
    sell_indices: Vec<u16>,
) {
    ExtBuilder::default().build().execute_with(|| {
        let (_, pool_id) = create_markets_and_deploy_combinatorial_pool(
            ALICE,
            BASE_ASSET,
            vec![MarketType::Categorical(2), MarketType::Categorical(2), MarketType::Scalar(0..=1)],
            _100,
            vec![1_250_000_000; 8],
            CENT,
        );

        let pool = <Pallet<Runtime> as PoolStorage>::get(pool_id).unwrap();
        let amount_in = _10;
        assert_ok!(AssetManager::deposit(pool.assets()[0], &BOB, amount_in));
        assert_ok!(AssetManager::deposit(pool.assets()[1], &BOB, amount_in));
        assert_ok!(AssetManager::deposit(pool.assets()[2], &BOB, amount_in));
        assert_ok!(AssetManager::deposit(pool.assets()[3], &BOB, amount_in));
        assert_ok!(AssetManager::deposit(pool.assets()[4], &BOB, amount_in));
        assert_ok!(AssetManager::deposit(pool.assets()[5], &BOB, amount_in));
        assert_ok!(AssetManager::deposit(pool.assets()[6], &BOB, amount_in));
        assert_ok!(AssetManager::deposit(pool.assets()[7], &BOB, amount_in));

        let buy: Vec<_> = buy_indices.iter().map(|&i| pool.assets()[i as usize]).collect();
        let keep: Vec<_> = keep_indices.iter().map(|&i| pool.assets()[i as usize]).collect();
        let sell: Vec<_> = sell_indices.iter().map(|&i| pool.assets()[i as usize]).collect();

        // Buying 1 at price of .5 will return less than 2 outcomes due to slippage.
        assert_noop!(
            NeoSwaps::combo_sell(
                RuntimeOrigin::signed(BOB),
                pool_id,
                8,
                buy,
                keep,
                sell,
                _2,
                0, // Keep this zero to avoid a different error due to invalid `amount_keep` param.
                0
            ),
            Error::<Runtime>::InvalidPartition,
        );
    });
}

#[test]
fn combo_sell_fails_on_spot_price_slipping_too_low() {
    ExtBuilder::default().build().execute_with(|| {
        let (_, pool_id) = create_markets_and_deploy_combinatorial_pool(
            ALICE,
            BASE_ASSET,
            vec![MarketType::Categorical(5)],
            _100,
            vec![20 * CENT; 5],
            CENT,
        );
        let pool = <Pallet<Runtime> as PoolStorage>::get(pool_id).unwrap();
        let amount_buy = 1_000 * _1;

        let buy = pool.assets()[0..4].to_vec();
        let sell = pool.assets()[4..5].to_vec();

        for &asset in buy.iter() {
            assert_ok!(AssetManager::deposit(asset, &BOB, amount_buy));
        }

        assert_noop!(
            NeoSwaps::combo_sell(
                RuntimeOrigin::signed(BOB),
                pool_id,
                5,
                buy,
                vec![],
                sell,
                amount_buy,
                0,
                0
            ),
            Error::<Runtime>::NumericalLimits(NumericalLimitsError::SpotPriceSlippedTooLow),
        );
    });
}

#[test]
fn combo_sell_fails_on_large_amount() {
    ExtBuilder::default().build().execute_with(|| {
        let (_, pool_id) = create_markets_and_deploy_combinatorial_pool(
            ALICE,
            BASE_ASSET,
            vec![MarketType::Categorical(5)],
            _100,
            vec![20 * CENT; 5],
            CENT,
        );
        let pool = <Pallet<Runtime> as PoolStorage>::get(pool_id).unwrap();
        let amount_buy = 100 * _100;

        let sell = pool.assets()[0..4].to_vec();
        let buy = pool.assets()[4..5].to_vec();

        for &asset in buy.iter() {
            assert_ok!(AssetManager::deposit(asset, &BOB, amount_buy));
        }

        assert_noop!(
            NeoSwaps::combo_sell(
                RuntimeOrigin::signed(BOB),
                pool_id,
                5,
                buy,
                vec![],
                sell,
                amount_buy,
                0,
                0
            ),
            Error::<Runtime>::MathError,
        );
    });
}

#[test_case(vec![], 1)]
#[test_case(vec![2], _2)]
fn combo_sell_fails_on_invalid_amount_keep(keep_indices: Vec<u16>, amount_in_keep: u128) {
    ExtBuilder::default().build().execute_with(|| {
        let (_, pool_id) = create_markets_and_deploy_combinatorial_pool(
            ALICE,
            BASE_ASSET,
            vec![MarketType::Categorical(5)],
            _100,
            vec![20 * CENT; 5],
            CENT,
        );
        let pool = <Pallet<Runtime> as PoolStorage>::get(pool_id).unwrap();

        let buy = vec![pool.assets()[1]];
        let keep: Vec<_> = keep_indices.iter().map(|&i| pool.assets()[i as usize]).collect();
        let sell = vec![pool.assets()[0]];

        assert_noop!(
            NeoSwaps::combo_sell(
                RuntimeOrigin::signed(BOB),
                pool_id,
                5,
                buy,
                keep,
                sell,
                _2,
                amount_in_keep,
                0,
            ),
            Error::<Runtime>::InvalidAmountKeep
        );
    });
}

#[test]
fn combo_sell_fails_on_invalid_pool_type() {
    ExtBuilder::default().build().execute_with(|| {
        let pool_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Categorical(5),
            _10,
            vec![_1_5, _1_5, _1_5, _1_5, _1_5],
            CENT,
        );
        let pool = <Pallet<Runtime> as PoolStorage>::get(pool_id).unwrap();
        let amount_buy = _1;

        let sell = pool.assets()[0..4].to_vec();
        let buy = pool.assets()[4..5].to_vec();

        for &asset in buy.iter() {
            assert_ok!(AssetManager::deposit(asset, &BOB, amount_buy));
        }

        assert_noop!(
            NeoSwaps::combo_sell(
                RuntimeOrigin::signed(BOB),
                pool_id,
                5,
                buy,
                vec![],
                sell,
                amount_buy,
                0,
                0
            ),
            Error::<Runtime>::InvalidPoolType,
        );
    });
}
