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
fn buy_emits_event() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        let mut market = market_mock::<Runtime>();
        market.status = MarketStatus::Active;
        Markets::<Runtime>::insert(market_id, market);

        let asset = Asset::ParimutuelShare(market_id, 0u16);
        let amount = <Runtime as Config>::MinBetSize::get();
        assert_ok!(Parimutuel::buy(RuntimeOrigin::signed(ALICE), asset, amount));

        let amount_minus_fees = 9900000000;
        let fees = 100000000;
        assert_eq!(amount, amount_minus_fees + fees);

        System::assert_last_event(
            Event::OutcomeBought { market_id, buyer: ALICE, asset, amount_minus_fees, fees }.into(),
        );
    });
}

#[test]
fn buy_balances_change_correctly() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        let mut market = market_mock::<Runtime>();
        market.status = MarketStatus::Active;
        market.creator = MARKET_CREATOR;
        Markets::<Runtime>::insert(market_id, market.clone());

        let base_asset = market.base_asset;

        let free_alice_before = AssetManager::free_balance(base_asset, &ALICE);
        let free_creator_before = AssetManager::free_balance(base_asset, &market.creator);
        let free_pot_before =
            AssetManager::free_balance(base_asset, &Parimutuel::pot_account(market_id));

        let asset = Asset::ParimutuelShare(market_id, 0u16);
        let amount = <Runtime as Config>::MinBetSize::get();
        assert_ok!(Parimutuel::buy(RuntimeOrigin::signed(ALICE), asset, amount));

        let amount_minus_fees = 9900000000;
        let fees = 100000000;
        assert_eq!(amount, amount_minus_fees + fees);

        assert_eq!(AssetManager::free_balance(base_asset, &ALICE), free_alice_before - amount);
        assert_eq!(
            AssetManager::free_balance(base_asset, &Parimutuel::pot_account(market_id))
                - free_pot_before,
            amount_minus_fees
        );
        assert_eq!(AssetManager::free_balance(asset, &ALICE), amount_minus_fees);
        assert_eq!(
            AssetManager::free_balance(base_asset, &market.creator) - free_creator_before,
            fees
        );
    });
}

#[test]
fn buy_fails_if_asset_not_parimutuel_share() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        let mut market = market_mock::<Runtime>();
        market.status = MarketStatus::Active;
        Markets::<Runtime>::insert(market_id, market.clone());

        let asset = Asset::CategoricalOutcome(market_id, 0u16);
        let amount = <Runtime as Config>::MinBetSize::get();
        assert_noop!(
            Parimutuel::buy(RuntimeOrigin::signed(ALICE), asset, amount),
            Error::<Runtime>::NotParimutuelOutcome
        );
    });
}

#[test_case(ScoringRule::CPMM, MarketStatus::Active, Error::<Runtime>::InvalidScoringRule)]
#[test_case(ScoringRule::Parimutuel, MarketStatus::Proposed, Error::<Runtime>::MarketIsNotActive)]
fn buy_fails(scoring_rule: ScoringRule, status: MarketStatus, error: Error<Runtime>) {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        let mut market = market_mock::<Runtime>();
        market.status = status;
        market.scoring_rule = scoring_rule;

        Markets::<Runtime>::insert(market_id, market.clone());

        let asset = Asset::ParimutuelShare(market_id, 0u16);
        let amount = <Runtime as Config>::MinBetSize::get();
        assert_noop!(Parimutuel::buy(RuntimeOrigin::signed(ALICE), asset, amount), error);
    });
}

#[test]
fn buy_fails_if_insufficient_balance() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        let mut market = market_mock::<Runtime>();
        market.status = MarketStatus::Active;
        Markets::<Runtime>::insert(market_id, market.clone());

        let free_alice = AssetManager::free_balance(market.base_asset, &ALICE);
        AssetManager::slash(market.base_asset, &ALICE, free_alice);

        let asset = Asset::ParimutuelShare(market_id, 0u16);
        let amount = <Runtime as Config>::MinBetSize::get();
        assert_noop!(
            Parimutuel::buy(RuntimeOrigin::signed(ALICE), asset, amount),
            Error::<Runtime>::InsufficientBalance
        );
    });
}

#[test]
fn buy_fails_if_below_minimum_bet_size() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        let mut market = market_mock::<Runtime>();
        market.status = MarketStatus::Active;
        Markets::<Runtime>::insert(market_id, market.clone());

        let asset = Asset::ParimutuelShare(market_id, 0u16);
        let amount = <Runtime as Config>::MinBetSize::get() - 1;
        assert_noop!(
            Parimutuel::buy(RuntimeOrigin::signed(ALICE), asset, amount),
            Error::<Runtime>::AmountTooSmall
        );
    });
}

#[test]
fn buy_fails_if_market_does_not_exist() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;

        let asset = Asset::ParimutuelShare(market_id, 0u16);
        let amount = <Runtime as Config>::MinBetSize::get();
        assert_noop!(
            Parimutuel::buy(RuntimeOrigin::signed(ALICE), asset, amount),
            MError::<Runtime>::MarketDoesNotExist
        );
    });
}

#[test]
fn claim_rewards_emits_event() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        let mut market = market_mock::<Runtime>();
        market.status = MarketStatus::Active;
        Markets::<Runtime>::insert(market_id, market);

        let winner_asset = Asset::ParimutuelShare(market_id, 0u16);
        let winner_amount =
            <Runtime as Config>::MinBetSize::get() + <Runtime as Config>::MinBetSize::get();
        assert_ok!(Parimutuel::buy(RuntimeOrigin::signed(ALICE), winner_asset, winner_amount));

        let loser_asset = Asset::ParimutuelShare(market_id, 1u16);
        let loser_amount = <Runtime as Config>::MinBetSize::get();
        assert_ok!(Parimutuel::buy(RuntimeOrigin::signed(BOB), loser_asset, loser_amount));

        let mut market = Markets::<Runtime>::get(market_id).unwrap();
        market.status = MarketStatus::Resolved;
        market.resolved_outcome = Some(OutcomeReport::Categorical(0u16));
        Markets::<Runtime>::insert(market_id, market);

        assert_ok!(Parimutuel::claim_rewards(RuntimeOrigin::signed(ALICE), market_id));

        let slashable_balance = 19800000000;
        let actual_payoff = 29700000000;

        System::assert_last_event(
            Event::RewardsClaimed {
                market_id,
                asset: winner_asset,
                balance: slashable_balance,
                actual_payoff,
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
        let mut market = market_mock::<Runtime>();
        market.status = MarketStatus::Active;
        Markets::<Runtime>::insert(market_id, market);

        let winner_asset = Asset::ParimutuelShare(market_id, 0u16);
        let winner_amount =
            <Runtime as Config>::MinBetSize::get() + <Runtime as Config>::MinBetSize::get();
        assert_ok!(Parimutuel::buy(RuntimeOrigin::signed(ALICE), winner_asset, winner_amount));

        let loser_asset = Asset::ParimutuelShare(market_id, 1u16);
        let loser_amount = <Runtime as Config>::MinBetSize::get();
        assert_ok!(Parimutuel::buy(RuntimeOrigin::signed(BOB), loser_asset, loser_amount));

        let mut market = Markets::<Runtime>::get(market_id).unwrap();
        market.status = MarketStatus::Resolved;
        market.resolved_outcome = Some(OutcomeReport::Categorical(0u16));
        Markets::<Runtime>::insert(market_id, market.clone());

        // 29700000000 / 19800000000 = 1.5, because loser amount is half of winner amount
        let actual_payoff = 29700000000;
        let total_pot_amount = loser_amount + winner_amount;
        let total_fees = Percent::from_percent(1) * total_pot_amount;
        assert_eq!(actual_payoff, total_pot_amount - total_fees);

        let slashable_balance = 19800000000;
        let winner_fees = Percent::from_percent(1) * winner_amount;
        assert_eq!(slashable_balance + winner_fees, winner_amount);

        let free_winner_asset_alice_before = AssetManager::free_balance(winner_asset, &ALICE);
        assert_eq!(free_winner_asset_alice_before, slashable_balance);
        let free_base_asset_alice_before = AssetManager::free_balance(market.base_asset, &ALICE);
        let free_base_asset_pot_before =
            AssetManager::free_balance(market.base_asset, &Parimutuel::pot_account(market_id));
        assert_eq!(free_base_asset_pot_before, total_pot_amount - total_fees);

        assert_ok!(Parimutuel::claim_rewards(RuntimeOrigin::signed(ALICE), market_id));

        assert_eq!(
            free_winner_asset_alice_before - AssetManager::free_balance(winner_asset, &ALICE),
            slashable_balance
        );

        assert_eq!(
            AssetManager::free_balance(market.base_asset, &ALICE) - free_base_asset_alice_before,
            actual_payoff
        );

        assert_eq!(
            AssetManager::free_balance(market.base_asset, &Parimutuel::pot_account(market_id)),
            0
        );
    });
}

#[test]
fn buy_fails_if_market_type_is_scalar() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        let mut market = market_mock::<Runtime>();
        let range: RangeInclusive<u128> = 0..=100;
        market.market_type = MarketType::Scalar(range);
        market.status = MarketStatus::Active;
        Markets::<Runtime>::insert(market_id, market);

        let winner_asset = Asset::ParimutuelShare(market_id, 0u16);
        let winner_amount =
            <Runtime as Config>::MinBetSize::get() + <Runtime as Config>::MinBetSize::get();
        assert_noop!(
            Parimutuel::buy(RuntimeOrigin::signed(ALICE), winner_asset, winner_amount),
            Error::<Runtime>::NotCategorical
        );
    });
}

#[test]
fn claim_rewards_fails_if_market_type_is_scalar() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        let mut market = market_mock::<Runtime>();
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

#[test]
fn claim_rewards_fails_if_not_resolved() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        let mut market = market_mock::<Runtime>();
        market.status = MarketStatus::Active;
        Markets::<Runtime>::insert(market_id, market);

        assert_noop!(
            Parimutuel::claim_rewards(RuntimeOrigin::signed(ALICE), market_id),
            Error::<Runtime>::MarketIsNotResolvedYet
        );
    });
}

#[test]
fn claim_rewards_fails_if_scoring_rule_not_parimutuel() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        let mut market = market_mock::<Runtime>();
        market.status = MarketStatus::Resolved;
        market.resolved_outcome = Some(OutcomeReport::Categorical(0u16));
        market.scoring_rule = ScoringRule::CPMM;
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
        let mut market = market_mock::<Runtime>();
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
        let mut market = market_mock::<Runtime>();
        market.status = MarketStatus::Active;
        Markets::<Runtime>::insert(market_id, market);

        let winner_asset = Asset::ParimutuelShare(market_id, 0u16);
        let winner_amount =
            <Runtime as Config>::MinBetSize::get() + <Runtime as Config>::MinBetSize::get();
        assert_ok!(Parimutuel::buy(RuntimeOrigin::signed(ALICE), winner_asset, winner_amount));

        let loser_asset = Asset::ParimutuelShare(market_id, 1u16);
        let loser_amount = <Runtime as Config>::MinBetSize::get();
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
        let mut market = market_mock::<Runtime>();
        market.market_type = MarketType::Categorical(10u16);
        market.status = MarketStatus::Active;
        Markets::<Runtime>::insert(market_id, market);

        let winner_asset = Asset::ParimutuelShare(market_id, 0u16);
        let winner_amount =
            <Runtime as Config>::MinBetSize::get() + <Runtime as Config>::MinBetSize::get();
        assert_ok!(Parimutuel::buy(RuntimeOrigin::signed(ALICE), winner_asset, winner_amount));

        let loser_asset = Asset::ParimutuelShare(market_id, 1u16);
        let loser_amount = <Runtime as Config>::MinBetSize::get();
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
        let mut market = market_mock::<Runtime>();
        market.market_type = MarketType::Categorical(10u16);
        market.status = MarketStatus::Active;
        Markets::<Runtime>::insert(market_id, market);

        let alice_asset = Asset::ParimutuelShare(market_id, 0u16);
        let alice_amount =
            <Runtime as Config>::MinBetSize::get() + <Runtime as Config>::MinBetSize::get();
        assert_ok!(Parimutuel::buy(RuntimeOrigin::signed(ALICE), alice_asset, alice_amount));

        let bob_asset = Asset::ParimutuelShare(market_id, 1u16);
        let bob_amount = <Runtime as Config>::MinBetSize::get();
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

#[test]
fn refund_fails_if_not_parimutuel_outcome() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        let mut market = market_mock::<Runtime>();
        market.market_type = MarketType::Categorical(10u16);
        market.resolved_outcome = Some(OutcomeReport::Categorical(0u16));
        market.status = MarketStatus::Resolved;
        Markets::<Runtime>::insert(market_id, market);

        assert_noop!(
            Parimutuel::claim_refunds(
                RuntimeOrigin::signed(ALICE),
                Asset::CategoricalOutcome(market_id, 0u16)
            ),
            Error::<Runtime>::NotParimutuelOutcome
        );
    });
}

#[test]
fn refund_fails_if_market_not_resolved() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        let mut market = market_mock::<Runtime>();
        market.market_type = MarketType::Categorical(10u16);
        market.status = MarketStatus::Active;
        Markets::<Runtime>::insert(market_id, market);

        let asset = Asset::ParimutuelShare(market_id, 0u16);
        assert_noop!(
            Parimutuel::claim_refunds(RuntimeOrigin::signed(ALICE), asset),
            Error::<Runtime>::MarketIsNotResolvedYet
        );
    });
}

#[test]
fn refund_fails_if_invalid_scoring_rule() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        let mut market = market_mock::<Runtime>();
        market.market_type = MarketType::Categorical(10u16);
        market.resolved_outcome = Some(OutcomeReport::Categorical(0u16));
        market.status = MarketStatus::Resolved;
        // invalid scoring rule
        market.scoring_rule = ScoringRule::CPMM;
        Markets::<Runtime>::insert(market_id, market);

        let asset = Asset::ParimutuelShare(market_id, 0u16);
        assert_noop!(
            Parimutuel::claim_refunds(RuntimeOrigin::signed(ALICE), asset),
            Error::<Runtime>::InvalidScoringRule
        );
    });
}

#[test]
fn refund_fails_if_invalid_outcome_asset() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        let mut market = market_mock::<Runtime>();
        market.market_type = MarketType::Categorical(10u16);
        market.resolved_outcome = Some(OutcomeReport::Categorical(0u16));
        market.status = MarketStatus::Resolved;
        Markets::<Runtime>::insert(market_id, market);

        let asset = Asset::ParimutuelShare(market_id, 20u16);
        assert_noop!(
            Parimutuel::claim_refunds(RuntimeOrigin::signed(ALICE), asset),
            Error::<Runtime>::InvalidOutcomeAsset
        );
    });
}

#[test]
fn refund_fails_if_no_resolved_outcome() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        let mut market = market_mock::<Runtime>();
        market.market_type = MarketType::Categorical(10u16);
        market.status = MarketStatus::Resolved;
        market.resolved_outcome = None;
        Markets::<Runtime>::insert(market_id, market);

        let asset = Asset::ParimutuelShare(market_id, 0u16);
        assert_noop!(
            Parimutuel::claim_refunds(RuntimeOrigin::signed(ALICE), asset),
            Error::<Runtime>::NoResolvedOutcome
        );
    });
}

#[test]
fn refund_fails_if_refund_not_allowed() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        let mut market = market_mock::<Runtime>();
        market.market_type = MarketType::Categorical(10u16);
        market.status = MarketStatus::Active;
        Markets::<Runtime>::insert(market_id, market);

        let asset = Asset::ParimutuelShare(market_id, 0u16);
        let amount = <Runtime as Config>::MinBetSize::get();
        assert_ok!(Parimutuel::buy(RuntimeOrigin::signed(ALICE), asset, amount));

        let mut market = Markets::<Runtime>::get(market_id).unwrap();
        market.resolved_outcome = Some(OutcomeReport::Categorical(0u16));
        market.status = MarketStatus::Resolved;
        Markets::<Runtime>::insert(market_id, market);

        let asset = Asset::ParimutuelShare(market_id, 0u16);
        assert_noop!(
            Parimutuel::claim_refunds(RuntimeOrigin::signed(ALICE), asset),
            Error::<Runtime>::RefundNotAllowed
        );
    });
}

#[test]
fn refund_fails_if_refundable_balance_is_zero() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        let mut market = market_mock::<Runtime>();
        market.market_type = MarketType::Categorical(10u16);
        market.status = MarketStatus::Active;
        Markets::<Runtime>::insert(market_id, market);

        let asset = Asset::ParimutuelShare(market_id, 0u16);
        let amount = <Runtime as Config>::MinBetSize::get();
        assert_ok!(Parimutuel::buy(RuntimeOrigin::signed(ALICE), asset, amount));

        let mut market = Markets::<Runtime>::get(market_id).unwrap();
        market.resolved_outcome = Some(OutcomeReport::Categorical(1u16));
        market.status = MarketStatus::Resolved;
        Markets::<Runtime>::insert(market_id, market);

        let asset = Asset::ParimutuelShare(market_id, 0u16);
        assert_ok!(Parimutuel::claim_refunds(RuntimeOrigin::signed(ALICE), asset));

        // already refunded above
        assert_noop!(
            Parimutuel::claim_refunds(RuntimeOrigin::signed(ALICE), asset),
            Error::<Runtime>::RefundableBalanceIsZero
        );
    });
}

#[test]
fn refund_emits_event() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        let mut market = market_mock::<Runtime>();
        market.market_type = MarketType::Categorical(10u16);
        market.status = MarketStatus::Active;
        Markets::<Runtime>::insert(market_id, market);

        let asset = Asset::ParimutuelShare(market_id, 0u16);
        let amount = <Runtime as Config>::MinBetSize::get();
        assert_ok!(Parimutuel::buy(RuntimeOrigin::signed(ALICE), asset, amount));

        let mut market = Markets::<Runtime>::get(market_id).unwrap();
        market.resolved_outcome = Some(OutcomeReport::Categorical(1u16));
        market.status = MarketStatus::Resolved;
        Markets::<Runtime>::insert(market_id, market);

        let asset = Asset::ParimutuelShare(market_id, 0u16);
        assert_ok!(Parimutuel::claim_refunds(RuntimeOrigin::signed(ALICE), asset));

        let amount_minus_fees = amount - (Percent::from_percent(1) * amount);

        System::assert_last_event(
            Event::BalanceRefunded {
                market_id,
                asset,
                refunded_balance: amount_minus_fees,
                sender: ALICE,
            }
            .into(),
        );
    });
}
