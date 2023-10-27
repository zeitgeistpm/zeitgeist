// Copyright 2023 Forecasting Technologies LTD.
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
fn sell_works() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
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
        let amount_in = _10;
        let pool_outcomes_before: Vec<_> =
            pool.assets().iter().map(|a| pool.reserve_of(a).unwrap()).collect();
        let pool_liquidity_before = pool.liquidity_parameter;
        AssetManager::deposit(BASE_ASSET, &BOB, amount_in).unwrap();
        assert_ok!(PredictionMarkets::buy_complete_set(
            RuntimeOrigin::signed(BOB),
            market_id,
            amount_in,
        ));
        let asset_in = pool.assets()[1];
        assert_ok!(NeoSwaps::sell(
            RuntimeOrigin::signed(BOB),
            market_id,
            2,
            asset_in,
            amount_in,
            0,
        ));
        let total_fee_percentage = swap_fee + EXTERNAL_FEES;
        let expected_amount_out = 59632253897u128;
        let expected_fees = total_fee_percentage.bmul(expected_amount_out).unwrap();
        let expected_swap_fee_amount = expected_fees / 2;
        let expected_external_fee_amount = expected_fees - expected_swap_fee_amount;
        let expected_amount_out_minus_fees = expected_amount_out - expected_fees;
        assert_eq!(AssetManager::free_balance(BASE_ASSET, &BOB), expected_amount_out_minus_fees);
        assert_eq!(AssetManager::free_balance(asset_in, &BOB), 0);
        let pool = Pools::<Runtime>::get(market_id).unwrap();
        assert_eq!(pool.liquidity_parameter, pool_liquidity_before);
        assert_eq!(pool.liquidity_shares_manager.owner, ALICE);
        assert_eq!(pool.liquidity_shares_manager.total_shares, liquidity);
        assert_eq!(pool.liquidity_shares_manager.fees, expected_swap_fee_amount);
        let pool_outcomes_after: Vec<_> =
            pool.assets().iter().map(|a| pool.reserve_of(a).unwrap()).collect();
        assert_eq!(pool_outcomes_after[0], pool_outcomes_before[0] - expected_amount_out);
        assert_eq!(
            pool_outcomes_after[1],
            pool_outcomes_before[1] + (amount_in - expected_amount_out)
        );
        let expected_pool_account_balance =
            expected_swap_fee_amount + AssetManager::minimum_balance(pool.collateral);
        assert_eq!(
            AssetManager::free_balance(BASE_ASSET, &pool.account_id),
            expected_pool_account_balance
        );
        assert_eq!(
            AssetManager::free_balance(BASE_ASSET, &FEE_ACCOUNT),
            expected_external_fee_amount
        );
        assert_eq!(
            AssetManager::total_issuance(pool.assets()[0]),
            liquidity + amount_in - expected_amount_out
        );
        assert_eq!(
            AssetManager::total_issuance(pool.assets()[1]),
            liquidity + amount_in - expected_amount_out
        );
        let price_sum =
            pool.assets().iter().map(|&a| pool.calculate_spot_price(a).unwrap()).sum::<u128>();
        assert_eq!(price_sum, _1);
        System::assert_last_event(
            Event::SellExecuted {
                who: BOB,
                market_id,
                asset_in,
                amount_in,
                amount_out: expected_amount_out,
                swap_fee_amount: expected_swap_fee_amount,
                external_fee_amount: expected_external_fee_amount,
            }
            .into(),
        );
    });
}

#[test]
fn sell_fails_on_incorrect_asset_count() {
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
            NeoSwaps::sell(
                RuntimeOrigin::signed(BOB),
                market_id,
                1,
                Asset::ScalarOutcome(market_id, ScalarPosition::Long),
                _1,
                0
            ),
            Error::<Runtime>::IncorrectAssetCount
        );
    });
}

#[test]
fn sell_fails_on_market_not_found() {
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
            NeoSwaps::sell(
                RuntimeOrigin::signed(BOB),
                market_id,
                2,
                Asset::ScalarOutcome(market_id, ScalarPosition::Long),
                _1,
                0
            ),
            zrml_market_commons::Error::<Runtime>::MarketDoesNotExist,
        );
    });
}

#[test_case(MarketStatus::Proposed)]
#[test_case(MarketStatus::Suspended)]
#[test_case(MarketStatus::Closed)]
#[test_case(MarketStatus::CollectingSubsidy)]
#[test_case(MarketStatus::InsufficientSubsidy)]
#[test_case(MarketStatus::Reported)]
#[test_case(MarketStatus::Disputed)]
#[test_case(MarketStatus::Resolved)]
fn sell_fails_on_inactive_market(market_status: MarketStatus) {
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
            NeoSwaps::sell(
                RuntimeOrigin::signed(BOB),
                market_id,
                2,
                Asset::ScalarOutcome(market_id, ScalarPosition::Long),
                _1,
                0
            ),
            Error::<Runtime>::MarketNotActive,
        );
    });
}

#[test]
fn sell_fails_on_pool_not_found() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id =
            create_market(ALICE, BASE_ASSET, MarketType::Scalar(0..=1), ScoringRule::Lmsr);
        assert_noop!(
            NeoSwaps::sell(
                RuntimeOrigin::signed(BOB),
                market_id,
                2,
                Asset::ScalarOutcome(market_id, ScalarPosition::Long),
                _1,
                0
            ),
            Error::<Runtime>::PoolNotFound,
        );
    });
}

#[test_case(MarketType::Categorical(2))]
#[test_case(MarketType::Scalar(0..=1))]
fn sell_fails_on_asset_not_found(market_type: MarketType) {
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
            NeoSwaps::sell(
                RuntimeOrigin::signed(BOB),
                market_id,
                2,
                Asset::CategoricalOutcome(market_id, 2),
                _1,
                u128::MAX,
            ),
            Error::<Runtime>::AssetNotFound,
        );
    });
}

#[test]
fn sell_fails_on_numerical_limits() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Scalar(0..=1),
            _10,
            vec![_1_2, _1_2],
            CENT,
        );
        let pool = Pools::<Runtime>::get(market_id).unwrap();
        let asset_in = Asset::ScalarOutcome(market_id, ScalarPosition::Long);
        let amount_in = 100 * pool.liquidity_parameter;
        assert_ok!(AssetManager::deposit(asset_in, &BOB, amount_in));
        assert_noop!(
            NeoSwaps::buy(RuntimeOrigin::signed(BOB), market_id, 2, asset_in, amount_in, 0),
            Error::<Runtime>::NumericalLimits,
        );
    });
}

#[test]
fn sell_fails_on_insufficient_funds() {
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
            NeoSwaps::sell(
                RuntimeOrigin::signed(BOB),
                market_id,
                2,
                asset_in,
                amount_in,
                u128::MAX,
            ),
            orml_tokens::Error::<Runtime>::BalanceTooLow,
        );
    });
}

#[test]
fn sell_fails_on_amount_out_below_min() {
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
            NeoSwaps::sell(RuntimeOrigin::signed(BOB), market_id, 2, asset_in, amount_in, _10),
            Error::<Runtime>::AmountOutBelowMin,
        );
    });
}

#[test]
fn sell_fails_on_spot_price_below_min() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Categorical(2),
            _10,
            vec![_1_2, _1_2],
            CENT,
        );
        let asset_in = Asset::CategoricalOutcome(market_id, 0);
        let amount_in = _80;
        assert_ok!(AssetManager::deposit(asset_in, &BOB, amount_in));
        assert_noop!(
            NeoSwaps::sell(RuntimeOrigin::signed(BOB), market_id, 2, asset_in, amount_in, 0),
            Error::<Runtime>::SpotPriceBelowMin
        );
    });
}
