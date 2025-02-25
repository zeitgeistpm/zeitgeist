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
#[cfg(not(feature = "parachain"))]
use sp_runtime::{DispatchError, TokenError};
use test_case::test_case;

// Example taken from
// https://github.com/gnosis/conditional-tokens-docs/blob/e73aa18ab82446049bca61df31fc88efd3cdc5cc/docs/intro3.md?plain=1#L78-L88
#[test]
fn buy_works() {
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
        let asset_out = pool.assets()[0];
        assert_ok!(AssetManager::deposit(BASE_ASSET, &BOB, amount_in));
        // Deposit some stuff in the pool account to check that the pools `reserves` fields tracks
        // the reserve correctly.
        assert_ok!(AssetManager::deposit(asset_out, &pool.account_id, _100));
        assert_ok!(NeoSwaps::buy(
            RuntimeOrigin::signed(BOB),
            market_id,
            2,
            asset_out,
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
        assert_balance!(BOB, asset_out, expected_amount_out);
        assert_balance!(
            pool.account_id,
            BASE_ASSET,
            expected_swap_fee_amount + AssetManager::minimum_balance(pool.collateral)
        );
        assert_balance!(FEE_ACCOUNT, BASE_ASSET, expected_external_fee_amount);
        System::assert_last_event(
            Event::BuyExecuted {
                who: BOB,
                pool_id: market_id,
                asset_out,
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
fn buy_fails_on_incorrect_asset_count() {
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
            NeoSwaps::buy(
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
fn buy_fails_on_market_not_found() {
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
            NeoSwaps::buy(
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
#[test_case(MarketStatus::Closed)]
#[test_case(MarketStatus::Reported)]
#[test_case(MarketStatus::Disputed)]
#[test_case(MarketStatus::Resolved)]
fn buy_fails_on_inactive_market(market_status: MarketStatus) {
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
            NeoSwaps::buy(
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
fn buy_fails_on_pool_not_found() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id =
            create_market(ALICE, BASE_ASSET, MarketType::Scalar(0..=1), ScoringRule::AmmCdaHybrid);
        assert_noop!(
            NeoSwaps::buy(
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
fn buy_fails_on_asset_not_found(market_type: MarketType) {
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
            NeoSwaps::buy(
                RuntimeOrigin::signed(BOB),
                market_id,
                2,
                Asset::CategoricalOutcome(market_id, 2),
                _1,
                0
            ),
            Error::<Runtime>::AssetNotFound,
        );
    });
}

#[test]
fn buy_fails_if_amount_in_is_greater_than_numerical_threshold() {
    ExtBuilder::default().build().execute_with(|| {
        let asset_count = 4;
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Categorical(asset_count),
            _10,
            vec![_1_4, _1_4, _1_4, _1_4],
            CENT,
        );
        let pool = Pools::<Runtime>::get(market_id).unwrap();
        // Using twice the threshold here to account for the removal of swap fees.
        let amount_in = 2 * pool.calculate_numerical_threshold();
        assert_ok!(AssetManager::deposit(BASE_ASSET, &BOB, amount_in));
        assert_noop!(
            NeoSwaps::buy(
                RuntimeOrigin::signed(BOB),
                market_id,
                asset_count,
                Asset::CategoricalOutcome(market_id, asset_count - 1),
                amount_in,
                0,
            ),
            Error::<Runtime>::NumericalLimits(NumericalLimitsError::MaxAmountExceeded),
        );
    });
}

#[test]
fn buy_fails_if_ln_arg_is_less_than_numerical_limit() {
    ExtBuilder::default().build().execute_with(|| {
        let asset_count = 4;
        let price = CENT;
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Categorical(asset_count),
            _10,
            vec![_1_4, _1_4, _1_2 - price, price],
            CENT,
        );
        let pool = Pools::<Runtime>::get(market_id).unwrap();
        let amount_in = 5 * CENT.bmul(pool.liquidity_parameter).unwrap();
        assert_ok!(AssetManager::deposit(BASE_ASSET, &BOB, amount_in));
        assert_noop!(
            NeoSwaps::buy(
                RuntimeOrigin::signed(BOB),
                market_id,
                asset_count,
                Asset::CategoricalOutcome(market_id, asset_count - 1),
                amount_in,
                0,
            ),
            Error::<Runtime>::NumericalLimits(NumericalLimitsError::MinAmountNotMet),
        );
    });
}

#[test]
fn buy_fails_on_insufficient_funds() {
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
        #[cfg(not(feature = "parachain"))]
        let expected_error = DispatchError::Token(TokenError::FundsUnavailable);
        #[cfg(feature = "parachain")]
        let expected_error = orml_tokens::Error::<Runtime>::BalanceTooLow;
        assert_ok!(AssetManager::deposit(BASE_ASSET, &BOB, amount_in - 1));
        assert_noop!(
            NeoSwaps::buy(
                RuntimeOrigin::signed(BOB),
                market_id,
                2,
                Asset::ScalarOutcome(market_id, ScalarPosition::Long),
                amount_in,
                0,
            ),
            expected_error,
        );
    });
}

#[test]
fn buy_fails_on_amount_out_below_min() {
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
            NeoSwaps::buy(
                RuntimeOrigin::signed(BOB),
                market_id,
                2,
                Asset::ScalarOutcome(market_id, ScalarPosition::Long),
                amount_in,
                _2,
            ),
            Error::<Runtime>::AmountOutBelowMin,
        );
    });
}

#[test]
fn buy_fails_on_invalid_pool_type() {
    ExtBuilder::default().build().execute_with(|| {
        let (_, pool_id) = create_markets_and_deploy_combinatorial_pool(
            ALICE,
            BASE_ASSET,
            vec![MarketType::Scalar(0..=1)],
            _10,
            vec![_1_2, _1_2],
            CENT,
        );

        let pool = <Pallet<Runtime> as PoolStorage>::get(pool_id).unwrap();
        let assets = pool.assets();

        assert_noop!(
            NeoSwaps::buy(RuntimeOrigin::signed(BOB), pool_id, 2, assets[0], _1, 0),
            Error::<Runtime>::InvalidPoolType,
        );
    });
}
