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
use crate::types::LiquidityTreeError;
use test_case::test_case;

#[test_case(MarketStatus::Active, vec![39_960_000_000, 4_066_153_704], 33_508_962_010)]
#[test_case(MarketStatus::Resolved, vec![40_000_000_000, 4_070_223_928], 33_486_637_585)]
fn exit_works(
    market_status: MarketStatus,
    amounts_out: Vec<BalanceOf<Runtime>>,
    new_liquidity_parameter: BalanceOf<Runtime>,
) {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
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
        assert_ok!(AssetManager::deposit(BASE_ASSET, &BOB, liquidity));
        assert_ok!(<Runtime as Config>::CompleteSetOperations::buy_complete_set(
            RuntimeOrigin::signed(BOB),
            market_id,
            liquidity
        ));
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
        assert_pool_status!(
            market_id,
            pool_balances,
            spot_prices,
            55_811_062_642,
            create_b_tree_map!({ALICE => _5, BOB => _5})
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
        assert_pool_status!(
            market_id,
            new_pool_balances,
            spot_prices,
            new_liquidity_parameter,
            create_b_tree_map!({ALICE => _1, BOB => _5})
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

// TODO Test that full exit doesn't kill the pool if there's more than one LP.

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
        MarketCommons::mutate_market(&market_id, |market| {
            market.status = MarketStatus::Resolved;
            Ok(())
        })
        .unwrap();
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
            Event::PoolDestroyed { who: ALICE, market_id, amounts_out }.into(),
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
            LiquidityTreeError::InsufficientStake.into_dispatch::<Runtime>(),
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
            .deposit_fees(1)));
        assert_noop!(
            NeoSwaps::exit(RuntimeOrigin::signed(ALICE), market_id, _1, vec![0, 0]),
            LiquidityTreeError::UnclaimedFees.into_dispatch::<Runtime>(),
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
