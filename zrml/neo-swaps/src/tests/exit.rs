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

#[test]
fn exit_works() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        let liquidity = _10;
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
        let pool_shares_amount = _4; // Remove 40% to the pool.
        let pool_before = Pools::<Runtime>::get(market_id).unwrap();
        let alice_outcomes_before = [
            AssetManager::free_balance(pool_before.assets()[0], &ALICE),
            AssetManager::free_balance(pool_before.assets()[1], &ALICE),
        ];
        let pool_outcomes_before: Vec<_> =
            pool_before.assets().iter().map(|a| pool_before.reserve_of(a).unwrap()).collect();
        assert_ok!(NeoSwaps::exit(
            RuntimeOrigin::signed(ALICE),
            market_id,
            pool_shares_amount,
            vec![0, 0],
        ));
        let pool_after = Pools::<Runtime>::get(market_id).unwrap();
        let ratio = bdiv(pool_shares_amount, liquidity).unwrap();
        let pool_outcomes_after: Vec<_> =
            pool_after.assets().iter().map(|a| pool_after.reserve_of(a).unwrap()).collect();
        let expected_pool_diff = vec![
            bmul(ratio, pool_outcomes_before[0]).unwrap(),
            bmul(ratio, pool_outcomes_before[1]).unwrap(),
        ];
        let alice_outcomes_after = [
            AssetManager::free_balance(pool_after.assets()[0], &ALICE),
            AssetManager::free_balance(pool_after.assets()[1], &ALICE),
        ];
        assert_eq!(pool_outcomes_after[0], pool_outcomes_before[0] - expected_pool_diff[0]);
        assert_eq!(pool_outcomes_after[1], pool_outcomes_before[1] - expected_pool_diff[1]);
        assert_eq!(alice_outcomes_after[0], alice_outcomes_before[0] + expected_pool_diff[0]);
        assert_eq!(alice_outcomes_after[1], alice_outcomes_before[1] + expected_pool_diff[1]);
        assert_eq!(
            pool_after.liquidity_parameter,
            bmul(_1 - ratio, pool_before.liquidity_parameter).unwrap()
        );
        assert_eq!(
            pool_after.liquidity_shares_manager.shares_of(&ALICE).unwrap(),
            liquidity - pool_shares_amount
        );
        System::assert_last_event(
            Event::ExitExecuted {
                who: ALICE,
                market_id,
                pool_shares_amount,
                amounts_out: expected_pool_diff,
                new_liquidity_parameter: pool_after.liquidity_parameter,
            }
            .into(),
        );
    });
}

#[test]
fn exit_destroys_pool() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        let liquidity = _10;
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Scalar(0..=1),
            liquidity,
            vec![_1_6, _5_6 + 1],
            CENT,
        );
        let pool = Pools::<Runtime>::get(market_id).unwrap();
        let amounts_out = vec![
            pool.reserve_of(&pool.assets()[0]).unwrap(),
            pool.reserve_of(&pool.assets()[1]).unwrap(),
        ];
        let alice_before = [
            AssetManager::free_balance(pool.assets()[0], &ALICE),
            AssetManager::free_balance(pool.assets()[1], &ALICE),
        ];
        assert_ok!(NeoSwaps::exit(RuntimeOrigin::signed(ALICE), market_id, liquidity, vec![0, 0]));
        assert!(!Pools::<Runtime>::contains_key(market_id));
        assert_eq!(AssetManager::free_balance(pool.collateral, &pool.account_id), 0);
        assert_eq!(AssetManager::free_balance(pool.assets()[0], &pool.account_id), 0);
        assert_eq!(AssetManager::free_balance(pool.assets()[1], &pool.account_id), 0);
        assert_eq!(
            AssetManager::free_balance(pool.assets()[0], &ALICE),
            alice_before[0] + amounts_out[0]
        );
        assert_eq!(
            AssetManager::free_balance(pool.assets()[1], &ALICE),
            alice_before[1] + amounts_out[1]
        );
        System::assert_last_event(
            Event::PoolDestroyed {
                who: ALICE,
                market_id,
                pool_shares_amount: liquidity,
                amounts_out,
            }
            .into(),
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
            Error::<Runtime>::InsufficientPoolShares,
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
fn exit_fails_if_not_allowed() {
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
        assert_ok!(AssetManager::deposit(BASE_ASSET, &BOB, pool_shares_amount));
        assert_ok!(PredictionMarkets::buy_complete_set(
            RuntimeOrigin::signed(BOB),
            market_id,
            pool_shares_amount,
        ));
        assert_noop!(
            NeoSwaps::exit(
                RuntimeOrigin::signed(BOB),
                market_id,
                pool_shares_amount,
                vec![pool_shares_amount, pool_shares_amount]
            ),
            Error::<Runtime>::NotAllowed
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
        let pool_shares_amount = _20;
        assert_ok!(AssetManager::deposit(BASE_ASSET, &BOB, pool_shares_amount));
        assert_ok!(PredictionMarkets::buy_complete_set(
            RuntimeOrigin::signed(BOB),
            market_id,
            pool_shares_amount,
        ));
        assert_ok!(Pools::<Runtime>::try_mutate(market_id, |pool| pool
            .as_mut()
            .unwrap()
            .liquidity_shares_manager
            .deposit_fees(1)));
        assert_noop!(
            NeoSwaps::exit(
                RuntimeOrigin::signed(BOB),
                market_id,
                pool_shares_amount,
                vec![pool_shares_amount, pool_shares_amount]
            ),
            Error::<Runtime>::OutstandingFees
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
