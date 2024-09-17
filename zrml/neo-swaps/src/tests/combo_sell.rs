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

#[test]
fn combo_sell_works() {
    ExtBuilder::default().build().execute_with(|| {
        let liquidity = _10;
        let spot_prices = vec![_1_4, _3_4];
        let swap_fee = CENT;
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Scalar(0..=1),
            liquidity,
            spot_prices.clone(),
            swap_fee,
        );
        let pool = Pools::<Runtime>::get(market_id).unwrap();
        let amount_buy = _10;
        let amount_keep = 0;
        let liquidity_parameter_before = pool.liquidity_parameter;
        deposit_complete_set(market_id, BOB, amount_buy);
        let buy = vec![pool.assets()[1]];
        let keep = vec![];
        let sell = vec![pool.assets()[0]];
        assert_ok!(NeoSwaps::combo_sell(
            RuntimeOrigin::signed(BOB),
            market_id,
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
        assert_balance!(BOB, buy[0], 0);
        assert_pool_state!(
            market_id,
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
        assert_eq!(
            AssetManager::total_issuance(pool.assets()[0]),
            liquidity + amount_buy - expected_amount_out
        );
        assert_eq!(
            AssetManager::total_issuance(pool.assets()[1]),
            liquidity + amount_buy - expected_amount_out
        );
        System::assert_last_event(
            Event::ComboSellExecuted {
                who: BOB,
                market_id,
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

#[test]
fn combo_sell_fails_on_incorrect_asset_count() {
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
            NeoSwaps::combo_sell(
                RuntimeOrigin::signed(BOB),
                market_id,
                1,
                vec![Asset::ScalarOutcome(market_id, ScalarPosition::Long)],
                vec![],
                vec![Asset::ScalarOutcome(market_id, ScalarPosition::Short)],
                _1,
                0,
                0
            ),
            Error::<Runtime>::IncorrectAssetCount
        );
    });
}

#[test]
fn combo_sell_fails_on_market_not_found() {
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
            NeoSwaps::combo_sell(
                RuntimeOrigin::signed(BOB),
                market_id,
                2,
                vec![Asset::ScalarOutcome(market_id, ScalarPosition::Long)],
                vec![],
                vec![Asset::ScalarOutcome(market_id, ScalarPosition::Short)],
                _1,
                0,
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
fn combo_sell_fails_on_inactive_market(market_status: MarketStatus) {
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
            NeoSwaps::combo_sell(
                RuntimeOrigin::signed(BOB),
                market_id,
                2,
                vec![Asset::ScalarOutcome(market_id, ScalarPosition::Long)],
                vec![],
                vec![Asset::ScalarOutcome(market_id, ScalarPosition::Short)],
                _1,
                0,
                0
            ),
            Error::<Runtime>::MarketNotActive,
        );
    });
}

#[test]
fn combo_sell_fails_on_pool_not_found() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id =
            create_market(ALICE, BASE_ASSET, MarketType::Scalar(0..=1), ScoringRule::AmmCdaHybrid);
        assert_noop!(
            NeoSwaps::combo_sell(
                RuntimeOrigin::signed(BOB),
                market_id,
                2,
                vec![Asset::ScalarOutcome(market_id, ScalarPosition::Long)],
                vec![],
                vec![Asset::ScalarOutcome(market_id, ScalarPosition::Short)],
                _1,
                0,
                0
            ),
            Error::<Runtime>::PoolNotFound,
        );
    });
}

// TODO Needs to be expanded.
#[test_case(MarketType::Categorical(2))]
#[test_case(MarketType::Scalar(0..=1))]
fn combo_sell_fails_on_asset_not_found(market_type: MarketType) {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            market_type,
            _10,
            vec![_1_2, _1_2],
            CENT,
        );
        assert_noop!(
            NeoSwaps::combo_sell(
                RuntimeOrigin::signed(BOB),
                market_id,
                2,
                vec![Asset::CategoricalOutcome(market_id, 3)],
                vec![Asset::CategoricalOutcome(market_id, 5)],
                vec![Asset::CategoricalOutcome(market_id, 4)],
                _1,
                0,
                u128::MAX,
            ),
            Error::<Runtime>::AssetNotFound,
        );
    });
}

#[test]
fn combo_sell_fails_on_insufficient_funds() {
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
        let asset_in = Asset::ScalarOutcome(market_id, ScalarPosition::Long);
        assert_ok!(AssetManager::deposit(asset_in, &BOB, amount_in - 1));
        assert_noop!(
            NeoSwaps::combo_sell(
                RuntimeOrigin::signed(BOB),
                market_id,
                2,
                vec![Asset::ScalarOutcome(market_id, ScalarPosition::Long)],
                vec![],
                vec![Asset::ScalarOutcome(market_id, ScalarPosition::Short)],
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
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Scalar(0..=1),
            _100,
            vec![_1_2, _1_2],
            CENT,
        );
        let amount_in = _20;
        let asset_in = Asset::ScalarOutcome(market_id, ScalarPosition::Long);
        assert_ok!(AssetManager::deposit(asset_in, &BOB, amount_in));
        // Selling 20 at price of .5 will return less than 10 dollars due to slippage.
        assert_noop!(
            NeoSwaps::combo_sell(
                RuntimeOrigin::signed(BOB),
                market_id,
                2,
                vec![Asset::ScalarOutcome(market_id, ScalarPosition::Long)],
                vec![],
                vec![Asset::ScalarOutcome(market_id, ScalarPosition::Short)],
                amount_in,
                0,
                _10
            ),
            Error::<Runtime>::AmountOutBelowMin,
        );
    });
}
