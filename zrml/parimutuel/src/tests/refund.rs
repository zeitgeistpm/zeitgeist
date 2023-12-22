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
use frame_support::{assert_noop, assert_ok};
use sp_runtime::Percent;
use test_case::test_case;
use zeitgeist_primitives::types::{Asset, MarketStatus, MarketType, OutcomeReport, ScoringRule};
use zrml_market_commons::Markets;

#[test]
fn refund_fails_if_not_parimutuel_outcome() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        let mut market = market_mock::<Runtime>(MARKET_CREATOR);
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

#[test_case(MarketStatus::Active; "active")]
#[test_case(MarketStatus::Proposed; "proposed")]
#[test_case(MarketStatus::Suspended; "suspended")]
#[test_case(MarketStatus::Closed; "closed")]
#[test_case(MarketStatus::CollectingSubsidy; "collecting subsidy")]
#[test_case(MarketStatus::InsufficientSubsidy; "insufficient subsidy")]
#[test_case(MarketStatus::Reported; "reported")]
#[test_case(MarketStatus::Disputed; "disputed")]
fn refund_fails_if_market_not_resolved(status: MarketStatus) {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        let mut market = market_mock::<Runtime>(MARKET_CREATOR);
        market.market_type = MarketType::Categorical(10u16);
        market.status = status;
        Markets::<Runtime>::insert(market_id, market);

        let asset = Asset::ParimutuelShare(market_id, 0u16);
        assert_noop!(
            Parimutuel::claim_refunds(RuntimeOrigin::signed(ALICE), asset),
            Error::<Runtime>::MarketIsNotResolvedYet
        );
    });
}

#[test_case(ScoringRule::CPMM; "cpmm")]
#[test_case(ScoringRule::Orderbook; "orderbook")]
#[test_case(ScoringRule::Lmsr; "lmsr")]
#[test_case(ScoringRule::RikiddoSigmoidFeeMarketEma; "rikiddo sigmoid fee market ema")]
fn refund_fails_if_invalid_scoring_rule(scoring_rule: ScoringRule) {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        let mut market = market_mock::<Runtime>(MARKET_CREATOR);
        market.market_type = MarketType::Categorical(10u16);
        market.resolved_outcome = Some(OutcomeReport::Categorical(0u16));
        market.status = MarketStatus::Resolved;
        // invalid scoring rule
        market.scoring_rule = scoring_rule;
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
        let mut market = market_mock::<Runtime>(MARKET_CREATOR);
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
        let mut market = market_mock::<Runtime>(MARKET_CREATOR);
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
        let mut market = market_mock::<Runtime>(MARKET_CREATOR);
        market.market_type = MarketType::Categorical(10u16);
        market.status = MarketStatus::Active;
        Markets::<Runtime>::insert(market_id, market);

        let asset = Asset::ParimutuelShare(market_id, 0u16);
        let amount = 10 * <Runtime as Config>::MinBetSize::get();
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
        let mut market = market_mock::<Runtime>(MARKET_CREATOR);
        market.market_type = MarketType::Categorical(10u16);
        market.status = MarketStatus::Active;
        Markets::<Runtime>::insert(market_id, market);

        let asset = Asset::ParimutuelShare(market_id, 0u16);
        let amount = 2 * <Runtime as Config>::MinBetSize::get();
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
        let mut market = market_mock::<Runtime>(MARKET_CREATOR);
        market.market_type = MarketType::Categorical(10u16);
        market.status = MarketStatus::Active;
        Markets::<Runtime>::insert(market_id, market);

        let asset = Asset::ParimutuelShare(market_id, 0u16);
        let amount = 10 * <Runtime as Config>::MinBetSize::get();
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
