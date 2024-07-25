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

use zeitgeist_primitives::{constants::MILLISECS_PER_BLOCK, types::OutcomeReport};

// TODO(#1239) MarketDoesNotExist
// TODO(#1239) MarketAlreadyReported
// TODO(#1239) Trusted markets resolve immediately
// TODO(#1239) NotAllowedToReport with timestamps
// TODO(#1239) Reports are allowed after the oracle duration
// TODO(#1239) Outsider can't report if they can't pay for the bond

#[test]
fn it_allows_to_report_the_outcome_of_a_market() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 100;
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Permissionless,
            0..end,
            ScoringRule::AmmCdaHybrid,
        );

        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        run_to_block(grace_period + 1);

        let market = MarketCommons::market(&0).unwrap();
        assert_eq!(market.status, MarketStatus::Closed);
        assert!(market.report.is_none());

        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(BOB),
            0,
            OutcomeReport::Categorical(1)
        ));

        let market_after = MarketCommons::market(&0).unwrap();
        let report = market_after.report.unwrap();
        assert_eq!(market_after.status, MarketStatus::Reported);
        assert_eq!(report.outcome, OutcomeReport::Categorical(1));
        assert_eq!(report.by, market_after.oracle);
    });
}

#[test]
fn report_fails_before_grace_period_is_over() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 100;
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Permissionless,
            0..end,
            ScoringRule::AmmCdaHybrid,
        );

        run_to_block(end);

        let market = MarketCommons::market(&0).unwrap();
        assert_eq!(market.status, MarketStatus::Closed);
        assert!(market.report.is_none());

        assert_noop!(
            PredictionMarkets::report(RuntimeOrigin::signed(BOB), 0, OutcomeReport::Categorical(1)),
            Error::<Runtime>::NotAllowedToReportYet
        );
    });
}

// TODO(#1239) This test is misnamed - this does NOT ensure that reports outside of the oracle duration are
// not allowed.
#[test]
fn it_allows_only_oracle_to_report_the_outcome_of_a_market_during_oracle_duration_blocks() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 100;
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Permissionless,
            0..end,
            ScoringRule::AmmCdaHybrid,
        );

        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        run_to_block(grace_period);

        let market = MarketCommons::market(&0).unwrap();
        assert_eq!(market.status, MarketStatus::Closed);
        assert!(market.report.is_none());

        assert_noop!(
            PredictionMarkets::report(
                RuntimeOrigin::signed(CHARLIE),
                0,
                OutcomeReport::Categorical(1)
            ),
            Error::<Runtime>::ReporterNotOracle
        );

        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(BOB),
            0,
            OutcomeReport::Categorical(1)
        ));

        let market_after = MarketCommons::market(&0).unwrap();
        let report = market_after.report.unwrap();
        assert_eq!(market_after.status, MarketStatus::Reported);
        assert_eq!(report.outcome, OutcomeReport::Categorical(1));
        assert_eq!(report.by, market_after.oracle);
    });
}

#[test]
fn it_allows_only_oracle_to_report_the_outcome_of_a_market_during_oracle_duration_moment() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            Asset::Ztg,
            Perbill::zero(),
            BOB,
            MarketPeriod::Timestamp(0..100_000_000),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(2),
            Some(MarketDisputeMechanism::Court),
            ScoringRule::AmmCdaHybrid
        ));

        assert_ok!(PredictionMarkets::buy_complete_set(RuntimeOrigin::signed(BOB), 0, CENT));

        // set the timestamp
        let market = MarketCommons::market(&0).unwrap();
        // set the timestamp

        set_timestamp_for_on_initialize(100_000_000);
        run_to_block(2); // Trigger `on_initialize`; must be at least block #2.
        let grace_period: u64 = market.deadlines.grace_period * MILLISECS_PER_BLOCK as u64;
        Timestamp::set_timestamp(100_000_000 + grace_period);

        assert_noop!(
            PredictionMarkets::report(RuntimeOrigin::signed(EVE), 0, OutcomeReport::Categorical(1)),
            Error::<Runtime>::ReporterNotOracle
        );
        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(BOB),
            0,
            OutcomeReport::Categorical(1)
        ));
    });
}

// TODO(#1239) Use test_case!
#[test]
fn report_fails_on_mismatched_outcome_for_categorical_market() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 100;
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Permissionless,
            0..end,
            ScoringRule::AmmCdaHybrid,
        );
        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        run_to_block(grace_period + 1);
        assert_noop!(
            PredictionMarkets::report(RuntimeOrigin::signed(BOB), 0, OutcomeReport::Scalar(123)),
            Error::<Runtime>::OutcomeMismatch,
        );
        let market = MarketCommons::market(&0).unwrap();
        assert_eq!(market.status, MarketStatus::Closed);
        assert!(market.report.is_none());
    });
}

#[test]
fn report_fails_on_out_of_range_outcome_for_categorical_market() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 100;
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Permissionless,
            0..end,
            ScoringRule::AmmCdaHybrid,
        );
        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        run_to_block(grace_period + 1);
        assert_noop!(
            PredictionMarkets::report(RuntimeOrigin::signed(BOB), 0, OutcomeReport::Categorical(2)),
            Error::<Runtime>::OutcomeMismatch,
        );
        let market = MarketCommons::market(&0).unwrap();
        assert_eq!(market.status, MarketStatus::Closed);
        assert!(market.report.is_none());
    });
}

#[test]
fn report_fails_on_mismatched_outcome_for_scalar_market() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 100;
        simple_create_scalar_market(
            Asset::Ztg,
            MarketCreation::Permissionless,
            0..end,
            ScoringRule::AmmCdaHybrid,
        );
        let market = MarketCommons::market(&0).unwrap();
        let grace_period = end + market.deadlines.grace_period;
        run_to_block(grace_period + 1);
        assert_noop!(
            PredictionMarkets::report(RuntimeOrigin::signed(BOB), 0, OutcomeReport::Categorical(0)),
            Error::<Runtime>::OutcomeMismatch,
        );
        let market = MarketCommons::market(&0).unwrap();
        assert_eq!(market.status, MarketStatus::Closed);
        assert!(market.report.is_none());
    });
}

#[test]
fn it_allows_anyone_to_report_an_unreported_market() {
    ExtBuilder::default().build().execute_with(|| {
        let end = 2;
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Permissionless,
            0..end,
            ScoringRule::AmmCdaHybrid,
        );

        let market = MarketCommons::market(&0).unwrap();
        // Just skip to waaaay overdue.
        run_to_block(end + market.deadlines.grace_period + market.deadlines.oracle_duration + 1);

        assert_ok!(PredictionMarkets::report(
            RuntimeOrigin::signed(ALICE), // alice reports her own market now
            0,
            OutcomeReport::Categorical(1),
        ));

        let market = MarketCommons::market(&0).unwrap();
        assert_eq!(market.status, MarketStatus::Reported);
        assert_eq!(market.report.unwrap().by, ALICE);
        // but oracle was bob
        assert_eq!(market.oracle, BOB);

        // make sure it still resolves
        run_to_block(
            frame_system::Pallet::<Runtime>::block_number() + market.deadlines.dispute_duration,
        );

        let market_after = MarketCommons::market(&0).unwrap();
        assert_eq!(market_after.status, MarketStatus::Resolved);
    });
}

// TODO(#1239) Use `test_case`
#[test]
fn report_fails_on_market_state_proposed() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            Asset::Ztg,
            Perbill::zero(),
            BOB,
            MarketPeriod::Timestamp(0..100_000_000),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Advised,
            MarketType::Categorical(2),
            Some(MarketDisputeMechanism::Court),
            ScoringRule::AmmCdaHybrid
        ));
        assert_noop!(
            PredictionMarkets::report(RuntimeOrigin::signed(BOB), 0, OutcomeReport::Categorical(1)),
            Error::<Runtime>::MarketIsNotClosed,
        );
    });
}

#[test]
fn report_fails_on_market_state_closed_for_advised_market() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            Asset::Ztg,
            Perbill::zero(),
            BOB,
            MarketPeriod::Timestamp(0..100_000_000),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Advised,
            MarketType::Categorical(2),
            Some(MarketDisputeMechanism::Court),
            ScoringRule::AmmCdaHybrid
        ));
        assert_noop!(
            PredictionMarkets::report(RuntimeOrigin::signed(BOB), 0, OutcomeReport::Categorical(1)),
            Error::<Runtime>::MarketIsNotClosed,
        );
    });
}

#[test]
fn report_fails_on_market_state_active() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            Asset::Ztg,
            Perbill::zero(),
            BOB,
            MarketPeriod::Timestamp(0..100_000_000),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(2),
            Some(MarketDisputeMechanism::Court),
            ScoringRule::AmmCdaHybrid
        ));
        assert_noop!(
            PredictionMarkets::report(RuntimeOrigin::signed(BOB), 0, OutcomeReport::Categorical(1)),
            Error::<Runtime>::MarketIsNotClosed,
        );
    });
}

#[test]
fn report_fails_on_market_state_resolved() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            Asset::Ztg,
            Perbill::zero(),
            BOB,
            MarketPeriod::Timestamp(0..100_000_000),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Advised,
            MarketType::Categorical(2),
            Some(MarketDisputeMechanism::Court),
            ScoringRule::AmmCdaHybrid
        ));
        let _ = MarketCommons::mutate_market(&0, |market| {
            market.status = MarketStatus::Resolved;
            Ok(())
        });
        assert_noop!(
            PredictionMarkets::report(RuntimeOrigin::signed(BOB), 0, OutcomeReport::Categorical(1)),
            Error::<Runtime>::MarketIsNotClosed,
        );
    });
}

#[test]
fn report_fails_if_reporter_is_not_the_oracle() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(PredictionMarkets::create_market(
            RuntimeOrigin::signed(ALICE),
            Asset::Ztg,
            Perbill::zero(),
            BOB,
            MarketPeriod::Timestamp(0..100_000_000),
            get_deadlines(),
            gen_metadata(2),
            MarketCreation::Permissionless,
            MarketType::Categorical(2),
            Some(MarketDisputeMechanism::Court),
            ScoringRule::AmmCdaHybrid,
        ));
        let market = MarketCommons::market(&0).unwrap();
        set_timestamp_for_on_initialize(100_000_000);
        // Trigger hooks which close the market.
        run_to_block(2);
        let grace_period: u64 = market.deadlines.grace_period * MILLISECS_PER_BLOCK as u64;
        set_timestamp_for_on_initialize(100_000_000 + grace_period + MILLISECS_PER_BLOCK as u64);
        assert_noop!(
            PredictionMarkets::report(
                RuntimeOrigin::signed(CHARLIE),
                0,
                OutcomeReport::Categorical(1)
            ),
            Error::<Runtime>::ReporterNotOracle,
        );
    });
}
