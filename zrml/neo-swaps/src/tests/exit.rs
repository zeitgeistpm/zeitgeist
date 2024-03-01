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
use crate::liquidity_tree::types::LiquidityTreeError;
use test_case::test_case;

#[test_case(MarketStatus::Active, vec![39_960_000_000, 4_066_153_704], 33_508_962_010)]
#[test_case(MarketStatus::Resolved, vec![40_000_000_000, 4_070_223_928], 33_486_637_585)]
fn exit_works(
    market_status: MarketStatus,
    amounts_out: Vec<BalanceOf<Runtime>>,
    new_liquidity_parameter: BalanceOf<Runtime>,
) {
    ExtBuilder::default().build().execute_with(|| {
        let liquidity = _5;
        let spot_prices = vec![_1_6, _5_6 + 1];
        let swap_fee = CENT;
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Scalar(0..=1),
            liquidity,
            spot_prices.clone(),
            swap_fee,
        );
        // Add a second LP to create a more generic situation, bringing the total of shares to _10.
        deposit_complete_set(market_id, BOB, liquidity);
        assert_ok!(NeoSwaps::join(
            RuntimeOrigin::signed(BOB),
            market_id,
            liquidity,
            vec![u128::MAX, u128::MAX],
        ));
        MarketCommons::mutate_market(&market_id, |market| {
            market.status = market_status;
            Ok(())
        })
        .unwrap();
        let pool = Pools::<Runtime>::get(market_id).unwrap();
        let outcomes = pool.assets();
        let alice_balances = [0, 44_912_220_089];
        assert_balances!(ALICE, outcomes, alice_balances);
        let pool_balances = vec![100_000_000_000, 10_175_559_822];
        assert_pool_state!(
            market_id,
            pool_balances,
            spot_prices,
            55_811_062_642,
            create_b_tree_map!({ ALICE => _5, BOB => _5 }),
            0,
        );
        let pool_shares_amount = _4; // Remove 40% to the pool.
        assert_ok!(NeoSwaps::exit(
            RuntimeOrigin::signed(ALICE),
            market_id,
            pool_shares_amount,
            vec![0, 0],
        ));
        let new_pool_balances =
            pool_balances.iter().zip(amounts_out.iter()).map(|(b, a)| b - a).collect::<Vec<_>>();
        let new_alice_balances =
            alice_balances.iter().zip(amounts_out.iter()).map(|(b, a)| b + a).collect::<Vec<_>>();
        assert_balances!(ALICE, outcomes, new_alice_balances);
        assert_pool_state!(
            market_id,
            new_pool_balances,
            spot_prices,
            new_liquidity_parameter,
            create_b_tree_map!({ ALICE => _1, BOB => _5 }),
            0,
        );
        System::assert_last_event(
            Event::ExitExecuted {
                who: ALICE,
                market_id,
                pool_shares_amount,
                amounts_out,
                new_liquidity_parameter,
            }
            .into(),
        );
    });
}

#[test_case(MarketStatus::Active, vec![39_960_000_000, 4_066_153_705])]
#[test_case(MarketStatus::Resolved, vec![40_000_000_000, 4_070_223_929])]
fn last_exit_destroys_pool(market_status: MarketStatus, amounts_out: Vec<BalanceOf<Runtime>>) {
    ExtBuilder::default().build().execute_with(|| {
        let liquidity = _4;
        let spot_prices = vec![_1_6, _5_6 + 1];
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Scalar(0..=1),
            liquidity,
            spot_prices.clone(),
            CENT,
        );
        MarketCommons::mutate_market(&market_id, |market| {
            market.status = market_status;
            Ok(())
        })
        .unwrap();
        let pool = Pools::<Runtime>::get(market_id).unwrap();
        let pool_account = pool.account_id;
        let outcomes = pool.assets();
        let alice_balances = [0, 35_929_776_071];
        assert_balances!(ALICE, outcomes, alice_balances);
        let pool_balances = vec![40_000_000_000, 4_070_223_929];
        assert_pool_state!(
            market_id,
            pool_balances,
            spot_prices,
            22_324_425_057,
            create_b_tree_map!({ ALICE => _4 }),
            0,
        );
        assert_ok!(NeoSwaps::exit(RuntimeOrigin::signed(ALICE), market_id, liquidity, vec![0, 0]));
        let new_alice_balances =
            alice_balances.iter().zip(amounts_out.iter()).map(|(b, a)| b + a).collect::<Vec<_>>();
        assert_balances!(ALICE, outcomes, new_alice_balances);
        // Pool doesn't exist anymore and exit fees are cleared.
        assert!(!Pools::<Runtime>::contains_key(market_id));
        assert_balances!(pool_account, outcomes, [0, 0]);
        System::assert_last_event(
            Event::PoolDestroyed { who: ALICE, market_id, amounts_out }.into(),
        );
    });
}

#[test]
fn removing_second_to_last_lp_does_not_destroy_pool_and_removes_node_from_liquidity_tree() {
    ExtBuilder::default().build().execute_with(|| {
        let liquidity = _5;
        let spot_prices = vec![_1_6, _5_6 + 1];
        let swap_fee = CENT;
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Scalar(0..=1),
            liquidity,
            spot_prices.clone(),
            swap_fee,
        );
        // Add a second LP, bringing the total of shares to _10.
        deposit_complete_set(market_id, BOB, liquidity);
        assert_ok!(NeoSwaps::join(
            RuntimeOrigin::signed(BOB),
            market_id,
            liquidity,
            vec![u128::MAX, u128::MAX],
        ));
        assert_pool_state!(
            market_id,
            [100_000_000_000, 10_175_559_822],
            spot_prices,
            55_811_062_642,
            create_b_tree_map!({ ALICE => _5, BOB => _5 }),
            0,
        );
        assert_ok!(NeoSwaps::exit(RuntimeOrigin::signed(BOB), market_id, liquidity, vec![0, 0]));
        assert_pool_state!(
            market_id,
            [50_050_000_000, 5_092_867_691],
            spot_prices,
            27_933_436_852,
            create_b_tree_map!({ ALICE => _5 }),
            0,
        );
    });
}

#[test]
fn exit_fails_on_incorrect_vec_len() {
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
            NeoSwaps::exit(RuntimeOrigin::signed(ALICE), market_id, _1, vec![0]),
            Error::<Runtime>::IncorrectVecLen
        );
    });
}

#[test]
fn exit_fails_on_market_not_found() {
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
            NeoSwaps::exit(RuntimeOrigin::signed(ALICE), market_id, _1, vec![0, 0]),
            zrml_market_commons::Error::<Runtime>::MarketDoesNotExist
        );
    });
}

#[test]
fn exit_fails_on_pool_not_found() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id =
            create_market(ALICE, BASE_ASSET, MarketType::Scalar(0..=1), ScoringRule::Lmsr);
        assert_noop!(
            NeoSwaps::exit(RuntimeOrigin::signed(ALICE), market_id, _1, vec![0, 0]),
            Error::<Runtime>::PoolNotFound,
        );
    });
}

#[test]
fn exit_fails_on_insufficient_funds() {
    ExtBuilder::default().build().execute_with(|| {
        let liquidity = _10;
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Scalar(0..=1),
            liquidity,
            vec![_1_2, _1_2],
            CENT,
        );
        assert_noop!(
            NeoSwaps::exit(
                RuntimeOrigin::signed(ALICE),
                market_id,
                liquidity + 1, // One more than Alice has.
                vec![0, 0]
            ),
            LiquidityTreeError::InsufficientStake.into_dispatch_error::<Runtime>(),
        );
    });
}

#[test]
fn exit_fails_on_amount_out_below_min() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Scalar(0..=1),
            _20,
            vec![_1_2, _1_2],
            CENT,
        );
        let pool_shares_amount = _5;
        assert_ok!(PredictionMarkets::buy_complete_set(
            RuntimeOrigin::signed(ALICE),
            market_id,
            pool_shares_amount,
        ));
        assert_noop!(
            NeoSwaps::exit(
                RuntimeOrigin::signed(ALICE),
                market_id,
                pool_shares_amount,
                vec![pool_shares_amount + 1, pool_shares_amount]
            ),
            Error::<Runtime>::AmountOutBelowMin
        );
    });
}

#[test]
fn exit_fails_on_outstanding_fees() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Scalar(0..=1),
            _20,
            vec![_1_2, _1_2],
            CENT,
        );
        assert_ok!(Pools::<Runtime>::try_mutate(market_id, |pool| pool
            .as_mut()
            .unwrap()
            .liquidity_shares_manager
            .deposit_fees(_10)));
        assert_noop!(
            NeoSwaps::exit(RuntimeOrigin::signed(ALICE), market_id, _1, vec![0, 0]),
            LiquidityTreeError::UnwithdrawnFees.into_dispatch_error::<Runtime>(),
        );
    });
}

#[test]
fn exit_pool_fails_on_liquidity_too_low() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Scalar(0..=1),
            _10,
            vec![_1_2, _1_2],
            CENT,
        );
        // Will result in liquidity of about 0.7213475204444817.
        assert_noop!(
            NeoSwaps::exit(RuntimeOrigin::signed(ALICE), market_id, _10 - _1_2, vec![0, 0]),
            Error::<Runtime>::LiquidityTooLow
        );
    });
}

#[test]
fn exit_pool_fails_on_relative_liquidity_threshold_violated() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Scalar(0..=1),
            _100,
            vec![_1_2, _1_2],
            CENT,
        );
        // Bob contributes only 1.390...% of liquidity. Any removal (no matter how small the amount)
        // should fail.
        let amount = 13_910_041_100;
        deposit_complete_set(market_id, BOB, amount);
        assert_ok!(NeoSwaps::join(
            RuntimeOrigin::signed(BOB),
            market_id,
            amount,
            vec![u128::MAX, u128::MAX],
        ));
        assert_noop!(
            NeoSwaps::exit(RuntimeOrigin::signed(BOB), market_id, CENT, vec![0, 0]),
            Error::<Runtime>::MinRelativeLiquidityThresholdViolated
        );
    });
}
