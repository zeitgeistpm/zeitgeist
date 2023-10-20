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
fn join_works() {
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
        let pool_shares_amount = _4; // Add 40% to the pool.
        assert_ok!(AssetManager::deposit(BASE_ASSET, &ALICE, pool_shares_amount));
        assert_ok!(PredictionMarkets::buy_complete_set(
            RuntimeOrigin::signed(ALICE),
            market_id,
            pool_shares_amount,
        ));
        let pool_before = Pools::<Runtime>::get(market_id).unwrap();
        let alice_long_before = AssetManager::free_balance(pool_before.assets()[1], &ALICE);
        let pool_outcomes_before: Vec<_> =
            pool_before.assets().iter().map(|a| pool_before.reserve_of(a).unwrap()).collect();
        assert_ok!(NeoSwaps::join(
            RuntimeOrigin::signed(ALICE),
            market_id,
            pool_shares_amount,
            vec![u128::MAX, u128::MAX],
        ));
        let pool_after = Pools::<Runtime>::get(market_id).unwrap();
        let ratio = (liquidity + pool_shares_amount).bdiv(liquidity).unwrap();
        let pool_outcomes_after: Vec<_> =
            pool_after.assets().iter().map(|a| pool_after.reserve_of(a).unwrap()).collect();
        assert_eq!(pool_outcomes_after[0], ratio.bmul(pool_outcomes_before[0]).unwrap());
        assert_eq!(pool_outcomes_after[1], 14_245_783_753);
        let long_diff = pool_outcomes_after[1] - pool_outcomes_before[1];
        assert_eq!(AssetManager::free_balance(pool_after.assets()[0], &ALICE), 0);
        assert_eq!(
            AssetManager::free_balance(pool_after.assets()[1], &ALICE),
            alice_long_before - long_diff
        );
        assert_eq!(
            pool_after.liquidity_parameter,
            ratio.bmul(pool_before.liquidity_parameter).unwrap()
        );
        assert_eq!(
            pool_after.liquidity_shares_manager.shares_of(&ALICE).unwrap(),
            liquidity + pool_shares_amount
        );
        System::assert_last_event(
            Event::JoinExecuted {
                who: ALICE,
                market_id,
                pool_shares_amount,
                amounts_in: vec![pool_shares_amount, long_diff],
                new_liquidity_parameter: pool_after.liquidity_parameter,
            }
            .into(),
        );
    });
}

#[test]
fn join_fails_on_incorrect_vec_len() {
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
            NeoSwaps::join(RuntimeOrigin::signed(ALICE), market_id, _1, vec![0]),
            Error::<Runtime>::IncorrectVecLen
        );
    });
}

#[test]
fn join_fails_on_market_not_found() {
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
            NeoSwaps::join(RuntimeOrigin::signed(ALICE), market_id, _1, vec![u128::MAX, u128::MAX]),
            zrml_market_commons::Error::<Runtime>::MarketDoesNotExist
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
fn join_fails_on_inactive_market(market_status: MarketStatus) {
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
            NeoSwaps::join(RuntimeOrigin::signed(BOB), market_id, _1, vec![u128::MAX, u128::MAX]),
            Error::<Runtime>::MarketNotActive,
        );
    });
}

#[test]
fn join_fails_on_pool_not_found() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = create_market(ALICE, BASE_ASSET, MarketType::Scalar(0..=1), ScoringRule::Lmsr);
        assert_noop!(
            NeoSwaps::join(
                RuntimeOrigin::signed(ALICE),
                market_id,
                _1,
                vec![u128::MAX, u128::MAX],
            ),
            Error::<Runtime>::PoolNotFound,
        );
    });
}

#[test]
fn join_fails_on_insufficient_funds() {
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
            NeoSwaps::join(
                RuntimeOrigin::signed(ALICE),
                market_id,
                _100,
                vec![u128::MAX, u128::MAX]
            ),
            orml_tokens::Error::<Runtime>::BalanceTooLow
        );
    });
}

#[test]
fn join_fails_on_amount_in_above_max() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Scalar(0..=1),
            _20,
            vec![_1_2, _1_2],
            CENT,
        );
        let pool_shares_amount = _10;
        assert_ok!(PredictionMarkets::buy_complete_set(
            RuntimeOrigin::signed(ALICE),
            market_id,
            pool_shares_amount,
        ));
        assert_noop!(
            NeoSwaps::join(
                RuntimeOrigin::signed(ALICE),
                market_id,
                pool_shares_amount,
                vec![pool_shares_amount - 1, pool_shares_amount]
            ),
            Error::<Runtime>::AmountInAboveMax
        );
    });
}

#[test]
fn join_fails_if_not_allowed() {
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
            NeoSwaps::join(
                RuntimeOrigin::signed(BOB),
                market_id,
                pool_shares_amount,
                vec![pool_shares_amount, pool_shares_amount]
            ),
            Error::<Runtime>::NotAllowed
        );
    });
}
