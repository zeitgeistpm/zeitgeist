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

#![cfg(test)]

use crate::{mock::*, utils::*, *};
use core::ops::RangeInclusive;
use frame_support::{assert_noop, assert_ok};
use orml_traits::MultiCurrency;
use sp_runtime::Percent;
use test_case::test_case;
use zeitgeist_primitives::types::{Asset, MarketStatus, MarketType, OutcomeReport, ScoringRule};
use zrml_market_commons::{Error as MError, Markets};

#[test]
fn claim_rewards_emits_event() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        let mut market = market_mock::<Runtime>(MARKET_CREATOR);
        market.status = MarketStatus::Active;
        Markets::<Runtime>::insert(market_id, market);

        let winner_asset = Asset::ParimutuelShare(market_id, 0u16);
        let winner_amount = 20 * <Runtime as Config>::MinBetSize::get();
        assert_ok!(Parimutuel::buy(RuntimeOrigin::signed(ALICE), winner_asset, winner_amount));

        let loser_asset = Asset::ParimutuelShare(market_id, 1u16);
        let loser_amount = 10 * <Runtime as Config>::MinBetSize::get();
        assert_ok!(Parimutuel::buy(RuntimeOrigin::signed(BOB), loser_asset, loser_amount));

        let mut market = Markets::<Runtime>::get(market_id).unwrap();
        market.status = MarketStatus::Resolved;
        market.resolved_outcome = Some(OutcomeReport::Categorical(0u16));
        Markets::<Runtime>::insert(market_id, market);

        assert_ok!(Parimutuel::claim_rewards(RuntimeOrigin::signed(ALICE), market_id));

        let withdrawn_asset_balance = 198000000000;
        let actual_payoff = 297000000000;

        System::assert_last_event(
            Event::RewardsClaimed {
                market_id,
                asset: winner_asset,
                withdrawn_asset_balance,
                base_asset_payoff: actual_payoff,
                sender: ALICE,
            }
            .into(),
        );
    });
}

#[test]
fn claim_rewards_categorical_changes_balances_correctly() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        let mut market = market_mock::<Runtime>(MARKET_CREATOR);
        market.status = MarketStatus::Active;
        Markets::<Runtime>::insert(market_id, market);

        let winner_asset = Asset::ParimutuelShare(market_id, 0u16);
        let winner_amount_0 = 20 * <Runtime as Config>::MinBetSize::get();
        assert_ok!(Parimutuel::buy(RuntimeOrigin::signed(ALICE), winner_asset, winner_amount_0));

        let winner_amount_1 = 30 * <Runtime as Config>::MinBetSize::get();
        assert_ok!(Parimutuel::buy(RuntimeOrigin::signed(CHARLIE), winner_asset, winner_amount_1));

        let loser_asset = Asset::ParimutuelShare(market_id, 1u16);
        let loser_amount = 10 * <Runtime as Config>::MinBetSize::get();
        assert_ok!(Parimutuel::buy(RuntimeOrigin::signed(BOB), loser_asset, loser_amount));

        let mut market = Markets::<Runtime>::get(market_id).unwrap();
        market.status = MarketStatus::Resolved;
        market.resolved_outcome = Some(OutcomeReport::Categorical(0u16));
        Markets::<Runtime>::insert(market_id, market.clone());

        let actual_payoff = 594000000000;
        let winner_amount = winner_amount_0 + winner_amount_1;
        let total_pot_amount = loser_amount + winner_amount;
        let total_fees = Percent::from_percent(1) * total_pot_amount;
        assert_eq!(actual_payoff, total_pot_amount - total_fees);

        // 2/5 from 594000000000 = 237600000000
        let actual_payoff_alice = 237600000000;
        assert_eq!(Percent::from_percent(40) * actual_payoff, actual_payoff_alice);
        // 3/5 from 594000000000 = 356400000000
        let actual_payoff_charlie = 356400000000;
        assert_eq!(Percent::from_percent(60) * actual_payoff, actual_payoff_charlie);
        assert_eq!(actual_payoff_alice + actual_payoff_charlie, actual_payoff);

        let free_winner_asset_alice_before = AssetManager::free_balance(winner_asset, &ALICE);
        let winner_amount_0_minus_fees =
            winner_amount_0 - Percent::from_percent(1) * winner_amount_0;
        assert_eq!(free_winner_asset_alice_before, winner_amount_0_minus_fees);
        let free_base_asset_alice_before = AssetManager::free_balance(market.base_asset, &ALICE);
        let free_base_asset_pot_before =
            AssetManager::free_balance(market.base_asset, &Parimutuel::pot_account(market_id));
        assert_eq!(free_base_asset_pot_before, total_pot_amount - total_fees);

        assert_ok!(Parimutuel::claim_rewards(RuntimeOrigin::signed(ALICE), market_id));

        assert_eq!(
            free_winner_asset_alice_before - AssetManager::free_balance(winner_asset, &ALICE),
            winner_amount_0_minus_fees
        );

        assert_eq!(
            AssetManager::free_balance(market.base_asset, &ALICE) - free_base_asset_alice_before,
            actual_payoff_alice
        );

        assert_eq!(
            AssetManager::free_balance(market.base_asset, &Parimutuel::pot_account(market_id)),
            actual_payoff_charlie
        );

        let free_winner_asset_charlie_before = AssetManager::free_balance(winner_asset, &CHARLIE);
        let winner_amount_1_minus_fees =
            winner_amount_1 - Percent::from_percent(1) * winner_amount_1;
        assert_eq!(free_winner_asset_charlie_before, winner_amount_1_minus_fees);
        let free_base_asset_charlie_before =
            AssetManager::free_balance(market.base_asset, &CHARLIE);

        assert_ok!(Parimutuel::claim_rewards(RuntimeOrigin::signed(CHARLIE), market_id));

        assert_eq!(
            free_winner_asset_charlie_before - AssetManager::free_balance(winner_asset, &CHARLIE),
            winner_amount_1_minus_fees
        );
        assert_eq!(
            AssetManager::free_balance(market.base_asset, &CHARLIE)
                - free_base_asset_charlie_before,
            actual_payoff_charlie
        );
        assert_eq!(
            AssetManager::free_balance(market.base_asset, &Parimutuel::pot_account(market_id)),
            0
        );
    });
}

#[test]
fn claim_rewards_fails_if_market_type_is_scalar() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        let mut market = market_mock::<Runtime>(MARKET_CREATOR);
        let range: RangeInclusive<u128> = 0..=100;
        market.market_type = MarketType::Scalar(range);
        market.resolved_outcome = Some(OutcomeReport::Scalar(50));
        market.status = MarketStatus::Resolved;
        Markets::<Runtime>::insert(market_id, market);

        assert_noop!(
            Parimutuel::claim_rewards(RuntimeOrigin::signed(ALICE), market_id),
            Error::<Runtime>::NotCategorical
        );
    });
}

#[test_case(MarketStatus::Active; "active")]
#[test_case(MarketStatus::Proposed; "proposed")]
#[test_case(MarketStatus::Closed; "closed")]
#[test_case(MarketStatus::Reported; "reported")]
#[test_case(MarketStatus::Disputed; "disputed")]
fn claim_rewards_fails_if_not_resolved(status: MarketStatus) {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        let mut market = market_mock::<Runtime>(MARKET_CREATOR);
        market.status = status;
        Markets::<Runtime>::insert(market_id, market);

        assert_noop!(
            Parimutuel::claim_rewards(RuntimeOrigin::signed(ALICE), market_id),
            Error::<Runtime>::MarketIsNotResolvedYet
        );
    });
}

#[test_case(ScoringRule::Orderbook; "orderbook")]
#[test_case(ScoringRule::Lmsr; "lmsr")]
fn claim_rewards_fails_if_scoring_rule_not_parimutuel(scoring_rule: ScoringRule) {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        let mut market = market_mock::<Runtime>(MARKET_CREATOR);
        market.status = MarketStatus::Resolved;
        market.resolved_outcome = Some(OutcomeReport::Categorical(0u16));
        market.scoring_rule = scoring_rule;
        Markets::<Runtime>::insert(market_id, market);

        assert_noop!(
            Parimutuel::claim_rewards(RuntimeOrigin::signed(ALICE), market_id),
            Error::<Runtime>::InvalidScoringRule
        );
    });
}

#[test]
fn claim_rewards_fails_if_no_resolved_outcome() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        let mut market = market_mock::<Runtime>(MARKET_CREATOR);
        market.status = MarketStatus::Resolved;
        market.resolved_outcome = None;
        Markets::<Runtime>::insert(market_id, market);

        assert_noop!(
            Parimutuel::claim_rewards(RuntimeOrigin::signed(ALICE), market_id),
            Error::<Runtime>::NoResolvedOutcome
        );
    });
}

#[test]
fn claim_rewards_fails_if_market_does_not_exist() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;

        assert_noop!(
            Parimutuel::claim_rewards(RuntimeOrigin::signed(ALICE), market_id),
            MError::<Runtime>::MarketDoesNotExist
        );
    });
}

#[test]
fn claim_rewards_categorical_fails_if_no_winner() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        let mut market = market_mock::<Runtime>(MARKET_CREATOR);
        market.status = MarketStatus::Active;
        Markets::<Runtime>::insert(market_id, market);

        let winner_asset = Asset::ParimutuelShare(market_id, 0u16);
        let winner_amount = 20 * <Runtime as Config>::MinBetSize::get();
        assert_ok!(Parimutuel::buy(RuntimeOrigin::signed(ALICE), winner_asset, winner_amount));

        let loser_asset = Asset::ParimutuelShare(market_id, 1u16);
        let loser_amount = 10 * <Runtime as Config>::MinBetSize::get();
        assert_ok!(Parimutuel::buy(RuntimeOrigin::signed(BOB), loser_asset, loser_amount));

        let mut market = Markets::<Runtime>::get(market_id).unwrap();
        market.status = MarketStatus::Resolved;
        let winner_outcome = OutcomeReport::Categorical(6u16);
        market.resolved_outcome = Some(winner_outcome);
        Markets::<Runtime>::insert(market_id, market);

        assert_noop!(
            Parimutuel::claim_rewards(RuntimeOrigin::signed(ALICE), market_id),
            Error::<Runtime>::NoRewardShareOutstanding
        );
    });
}

#[test]
fn claim_rewards_categorical_fails_if_no_winning_shares() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        let mut market = market_mock::<Runtime>(MARKET_CREATOR);
        market.market_type = MarketType::Categorical(10u16);
        market.status = MarketStatus::Active;
        Markets::<Runtime>::insert(market_id, market);

        let winner_asset = Asset::ParimutuelShare(market_id, 0u16);
        let winner_amount = 20 * <Runtime as Config>::MinBetSize::get();
        assert_ok!(Parimutuel::buy(RuntimeOrigin::signed(ALICE), winner_asset, winner_amount));

        let loser_asset = Asset::ParimutuelShare(market_id, 1u16);
        let loser_amount = 10 * <Runtime as Config>::MinBetSize::get();
        assert_ok!(Parimutuel::buy(RuntimeOrigin::signed(BOB), loser_asset, loser_amount));

        let mut market = Markets::<Runtime>::get(market_id).unwrap();
        market.status = MarketStatus::Resolved;
        let winner_outcome = OutcomeReport::Categorical(0u16);
        market.resolved_outcome = Some(winner_outcome);
        Markets::<Runtime>::insert(market_id, market);

        assert_noop!(
            Parimutuel::claim_rewards(RuntimeOrigin::signed(BOB), market_id),
            Error::<Runtime>::NoWinningShares
        );
    });
}

#[test]
fn claim_refunds_works() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        let mut market = market_mock::<Runtime>(MARKET_CREATOR);
        market.market_type = MarketType::Categorical(10u16);
        market.status = MarketStatus::Active;
        Markets::<Runtime>::insert(market_id, market);

        let alice_asset = Asset::ParimutuelShare(market_id, 0u16);
        let alice_amount = 20 * <Runtime as Config>::MinBetSize::get();
        assert_ok!(Parimutuel::buy(RuntimeOrigin::signed(ALICE), alice_asset, alice_amount));

        let bob_asset = Asset::ParimutuelShare(market_id, 1u16);
        let bob_amount = 10 * <Runtime as Config>::MinBetSize::get();
        assert_ok!(Parimutuel::buy(RuntimeOrigin::signed(BOB), bob_asset, bob_amount));

        let mut market = Markets::<Runtime>::get(market_id).unwrap();
        market.status = MarketStatus::Resolved;
        // no winner, because nobody bought shares of the winning outcome
        let winner_outcome = OutcomeReport::Categorical(2u16);
        market.resolved_outcome = Some(winner_outcome);
        Markets::<Runtime>::insert(market_id, market.clone());

        assert_noop!(
            Parimutuel::claim_rewards(RuntimeOrigin::signed(ALICE), market_id),
            Error::<Runtime>::NoRewardShareOutstanding
        );

        let alice_paid_fees = Percent::from_percent(1) * alice_amount;
        let bob_paid_fees = Percent::from_percent(1) * bob_amount;
        let alice_amount_minus_fees = alice_amount - alice_paid_fees;
        let bob_amount_minus_fees = bob_amount - bob_paid_fees;

        let free_base_asset_alice_before = AssetManager::free_balance(market.base_asset, &ALICE);
        let free_base_asset_bob_before = AssetManager::free_balance(market.base_asset, &BOB);
        let free_base_asset_pot_before =
            AssetManager::free_balance(market.base_asset, &Parimutuel::pot_account(market_id));

        assert_ok!(Parimutuel::claim_refunds(RuntimeOrigin::signed(ALICE), alice_asset));

        assert_eq!(
            AssetManager::free_balance(market.base_asset, &ALICE) - free_base_asset_alice_before,
            alice_amount_minus_fees
        );
        assert_eq!(
            AssetManager::free_balance(market.base_asset, &Parimutuel::pot_account(market_id)),
            free_base_asset_pot_before - alice_amount_minus_fees
        );
        assert_eq!(
            AssetManager::free_balance(market.base_asset, &Parimutuel::pot_account(market_id)),
            bob_amount_minus_fees
        );

        assert_ok!(Parimutuel::claim_refunds(RuntimeOrigin::signed(BOB), bob_asset));
        assert_eq!(
            AssetManager::free_balance(market.base_asset, &BOB) - free_base_asset_bob_before,
            bob_amount_minus_fees
        );
        assert_eq!(
            AssetManager::free_balance(market.base_asset, &Parimutuel::pot_account(market_id)),
            0
        );
    });
}
