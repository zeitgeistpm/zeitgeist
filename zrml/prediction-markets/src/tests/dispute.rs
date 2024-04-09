// Copyright 2022-2024 Forecasting Technologies LTD.
// Copyright 2021-2022 Zeitgeist PM LLC.
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

use crate::MarketIdsPerDisputeBlock;
use zeitgeist_primitives::types::{Bond, OutcomeReport};

// TODO(#1239) fails if market doesn't exist
// TODO(#1239) fails if market is trusted
// TODO(#1239) fails if user can't afford the bond

#[test]
fn it_allows_to_dispute_the_outcome_of_a_market() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 2;
        simple_create_categorical_market(
            BaseAsset::Ztg,
            MarketCreation::Permissionless,
            0..end,
            ScoringRule::AmmCdaHybrid,
        );
        let market_id = 0;

        // Run to the end of the trading phase.
        let market = MarketCommons::market(&market_id).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        run_to_block(grace_period + 1);

        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(BOB),
            market_id,
            OutcomeReport::Categorical(1)
        ));

        let dispute_at = grace_period + 2;
        run_to_block(dispute_at);

        assert_ok!(PredictionMarkets::dispute(RuntimeOrigin::signed(CHARLIE), 0));
        let market = MarketCommons::market(&market_id).unwrap();
        assert_eq!(market.status, MarketStatus::Disputed);

        // Ensure that the MDM interacts correctly with auto resolution.
        assert_ok!(Authorized::authorize_market_outcome(
            RuntimeOrigin::signed(AuthorizedDisputeResolutionUser::get()),
            market_id,
            OutcomeReport::Categorical(0),
        ));
        let dispute_ends_at =
            dispute_at + <Runtime as zrml_authorized::Config>::CorrectionPeriod::get();
        let market_ids = MarketIdsPerDisputeBlock::<Runtime>::get(dispute_ends_at);
        assert_eq!(market_ids.len(), 1);
        assert_eq!(market_ids[0], 0);
    });
}

#[test]
fn dispute_fails_disputed_already() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 2;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            BaseAsset::Ztg,
            Perbill::zero(),
            BOB,
            MarketPeriod::Block(0..end),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(<Runtime as Config>::MinCategories::get()),
            Some(MarketDisputeMechanism::Authorized),
            ScoringRule::AmmCdaHybrid,
        ));

        // Run to the end of the trading phase.
        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        run_to_block(grace_period + 1);

        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(BOB),
            0,
            OutcomeReport::Categorical(1)
        ));

        let dispute_at = grace_period + 2;
        run_to_block(dispute_at);

        assert_ok!(PredictionMarkets::dispute(RuntimeOrigin::signed(CHARLIE), 0));

        assert_noop!(
            PredictionMarkets::dispute(RuntimeOrigin::signed(CHARLIE), 0),
            Error::<Runtime>::InvalidMarketStatus,
        );
    });
}

#[test]
fn dispute_fails_if_market_not_reported() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 2;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            BaseAsset::Ztg,
            Perbill::zero(),
            BOB,
            MarketPeriod::Block(0..end),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(<Runtime as Config>::MinCategories::get()),
            Some(MarketDisputeMechanism::Authorized),
            ScoringRule::AmmCdaHybrid,
        ));

        // Run to the end of the trading phase.
        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        run_to_block(grace_period + 1);

        // no report happening here...

        let dispute_at = grace_period + 2;
        run_to_block(dispute_at);

        assert_noop!(
            PredictionMarkets::dispute(RuntimeOrigin::signed(CHARLIE), 0),
            Error::<Runtime>::InvalidMarketStatus,
        );
    });
}

#[test]
fn dispute_reserves_dispute_bond() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 2;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            BaseAsset::Ztg,
            Perbill::zero(),
            BOB,
            MarketPeriod::Block(0..end),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(<Runtime as Config>::MinCategories::get()),
            Some(MarketDisputeMechanism::Authorized),
            ScoringRule::AmmCdaHybrid,
        ));

        // Run to the end of the trading phase.
        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        run_to_block(grace_period + 1);

        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(BOB),
            0,
            OutcomeReport::Categorical(1)
        ));

        let dispute_at = grace_period + 2;
        run_to_block(dispute_at);

        let free_charlie_before = Balances::free_balance(CHARLIE);
        let reserved_charlie = Balances::reserved_balance(CHARLIE);
        assert_eq!(reserved_charlie, 0);

        assert_ok!(PredictionMarkets::dispute(RuntimeOrigin::signed(CHARLIE), 0,));

        let free_charlie_after = Balances::free_balance(CHARLIE);
        assert_eq!(free_charlie_before - free_charlie_after, DisputeBond::get());

        let reserved_charlie = Balances::reserved_balance(CHARLIE);
        assert_eq!(reserved_charlie, DisputeBond::get());
    });
}

#[test]
fn dispute_updates_market() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 2;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            BaseAsset::Ztg,
            Perbill::zero(),
            BOB,
            MarketPeriod::Block(0..end),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(<Runtime as Config>::MinCategories::get()),
            Some(MarketDisputeMechanism::Authorized),
            ScoringRule::AmmCdaHybrid,
        ));

        // Run to the end of the trading phase.
        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        run_to_block(grace_period + 1);

        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(BOB),
            0,
            OutcomeReport::Categorical(1)
        ));

        let dispute_at = grace_period + 2;
        run_to_block(dispute_at);

        let market = MarketCommons::market(&0).unwrap();
        assert_eq!(market.status, MarketStatus::Reported);
        assert_eq!(market.bonds.dispute, None);

        assert_ok!(PredictionMarkets::dispute(RuntimeOrigin::signed(CHARLIE), 0,));

        let market = MarketCommons::market(&0).unwrap();
        assert_eq!(market.status, MarketStatus::Disputed);
        assert_eq!(
            market.bonds.dispute,
            Some(Bond { who: CHARLIE, value: DisputeBond::get(), is_settled: false })
        );
    });
}

#[test]
fn dispute_emits_event() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 2;
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            BaseAsset::Ztg,
            Perbill::zero(),
            BOB,
            MarketPeriod::Block(0..end),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(<Runtime as Config>::MinCategories::get()),
            Some(MarketDisputeMechanism::Authorized),
            ScoringRule::AmmCdaHybrid,
        ));

        // Run to the end of the trading phase.
        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        run_to_block(grace_period + 1);

        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(BOB),
            0,
            OutcomeReport::Categorical(1)
        ));

        let dispute_at = grace_period + 2;
        run_to_block(dispute_at);

        assert_ok!(PredictionMarkets::dispute(RuntimeOrigin::signed(CHARLIE), 0,));

        System::assert_last_event(
            Event::MarketDisputed(0u32.into(), MarketStatus::Disputed, CHARLIE).into(),
        );
    });
}

#[test_case(MarketStatus::Active; "active")]
#[test_case(MarketStatus::Closed; "closed")]
#[test_case(MarketStatus::Proposed; "proposed")]
#[test_case(MarketStatus::Resolved; "resolved")]
fn dispute_fails_unless_reported_or_disputed_market(status: MarketStatus) {
    ExtBuilder::default().build().execute_with(|| {
        // Creates a permissionless market.
        simple_create_categorical_market(
            BaseAsset::Ztg,
            MarketCreation::Permissionless,
            0..2,
            ScoringRule::AmmCdaHybrid,
        );

        assert_ok!(MarketCommons::mutate_market(&0, |market_inner| {
            market_inner.status = status;
            Ok(())
        }));

        assert_noop!(
            PredictionMarkets::dispute(RuntimeOrigin::signed(EVE), 0),
            Error::<Runtime>::InvalidMarketStatus
        );
    });
}

#[test]
fn does_trigger_market_transition_api() {
    ExtBuilder::default().build().execute_with(|| {
        StateTransitionMock::ensure_empty_state();
        let end = 2;
        simple_create_categorical_market(
            BaseAsset::Ztg,
            MarketCreation::Permissionless,
            0..end,
            ScoringRule::AmmCdaHybrid,
        );

        // Run to the end of the trading phase.
        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        run_to_block(grace_period + 1);

        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(BOB),
            0,
            OutcomeReport::Categorical(1)
        ));

        let dispute_at = grace_period + 2;
        run_to_block(dispute_at);

        assert_ok!(PredictionMarkets::dispute(RuntimeOrigin::signed(CHARLIE), 0));
        assert!(StateTransitionMock::on_dispute_triggered());
    });
}
