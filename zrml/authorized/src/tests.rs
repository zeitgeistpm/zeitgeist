#![cfg(test)]

use crate::{
    market_mock,
    mock::{Authorized, ExtBuilder, Origin, Runtime, ALICE, BOB},
    AuthorizedOutcomeReports, Error,
};
use frame_support::{assert_noop, assert_ok};
use zeitgeist_primitives::{
    traits::DisputeApi,
    types::{MarketDisputeMechanism, MarketStatus, OutcomeReport},
};
use zrml_market_commons::Markets;

#[test]
fn authorize_market_outcome_inserts_a_new_outcome() {
    ExtBuilder::default().build().execute_with(|| {
        Markets::<Runtime>::insert(0, market_mock::<Runtime>(ALICE));
        assert_ok!(Authorized::authorize_market_outcome(
            Origin::signed(ALICE),
            0,
            OutcomeReport::Scalar(1)
        ));
        assert_eq!(AuthorizedOutcomeReports::<Runtime>::get(0).unwrap(), OutcomeReport::Scalar(1));
    });
}

#[test]
fn authorize_market_outcome_fails_on_non_authorized_market() {
    ExtBuilder::default().build().execute_with(|| {
        let mut market = market_mock::<Runtime>(ALICE);
        market.mdm = MarketDisputeMechanism::Court;
        Markets::<Runtime>::insert(0, market);
        assert_noop!(
            Authorized::authorize_market_outcome(
                Origin::signed(ALICE),
                0,
                OutcomeReport::Scalar(1)
            ),
            Error::<Runtime>::MarketDoesNotHaveDisputeMechanismAuthorized
        );
    });
}

#[test]
fn authorize_market_outcome_fails_on_undisputed_market() {
    ExtBuilder::default().build().execute_with(|| {
        let mut market = market_mock::<Runtime>(ALICE);
        market.status = MarketStatus::Active;
        Markets::<Runtime>::insert(0, market);
        assert_noop!(
            Authorized::authorize_market_outcome(
                Origin::signed(ALICE),
                0,
                OutcomeReport::Scalar(1)
            ),
            Error::<Runtime>::MarketIsNotDisputed
        );
    });
}

#[test]
fn authorize_market_outcome_fails_on_invalid_report() {
    ExtBuilder::default().build().execute_with(|| {
        Markets::<Runtime>::insert(0, market_mock::<Runtime>(ALICE));
        assert_noop!(
            Authorized::authorize_market_outcome(
                Origin::signed(ALICE),
                0,
                OutcomeReport::Categorical(123)
            ),
            Error::<Runtime>::OutcomeMismatch
        );
    });
}

#[test]
fn authorize_market_outcome_fails_on_unauthorized_account() {
    ExtBuilder::default().build().execute_with(|| {
        Markets::<Runtime>::insert(0, market_mock::<Runtime>(ALICE));
        assert_noop!(
            Authorized::authorize_market_outcome(Origin::signed(BOB), 0, OutcomeReport::Scalar(1)),
            Error::<Runtime>::NotAuthorizedForThisMarket,
        );
    });
}

#[test]
fn on_resolution_fails_if_no_report_was_submitted() {
    ExtBuilder::default().build().execute_with(|| {
        let report = Authorized::on_resolution(&[], &0, &market_mock::<Runtime>(ALICE)).unwrap();
        assert!(report.is_none());
    });
}

#[test]
fn on_resolution_removes_stored_outcomes() {
    ExtBuilder::default().build().execute_with(|| {
        let market = market_mock::<Runtime>(ALICE);
        Markets::<Runtime>::insert(0, &market);
        assert_ok!(Authorized::authorize_market_outcome(
            Origin::signed(ALICE),
            0,
            OutcomeReport::Scalar(1)
        ));
        assert_ok!(Authorized::authorize_market_outcome(
            Origin::signed(ALICE),
            0,
            OutcomeReport::Scalar(2)
        ));
        assert_ok!(Authorized::on_resolution(&[], &0, &market));
        assert_eq!(AuthorizedOutcomeReports::<Runtime>::get(0), None);
    });
}

#[test]
fn on_resolution_returns_the_reported_outcome() {
    ExtBuilder::default().build().execute_with(|| {
        let market = market_mock::<Runtime>(ALICE);
        Markets::<Runtime>::insert(0, &market);
        // Authorize outcome, then overwrite it.
        assert_ok!(Authorized::authorize_market_outcome(
            Origin::signed(ALICE),
            0,
            OutcomeReport::Scalar(1)
        ));
        assert_ok!(Authorized::authorize_market_outcome(
            Origin::signed(ALICE),
            0,
            OutcomeReport::Scalar(2)
        ));
        assert_eq!(
            Authorized::on_resolution(&[], &0, &market).unwrap(),
            Some(OutcomeReport::Scalar(2))
        );
    });
}

#[test]
fn authorize_market_outcome_allows_using_same_account_on_multiple_markets() {
    ExtBuilder::default().build().execute_with(|| {
        Markets::<Runtime>::insert(0, market_mock::<Runtime>(ALICE));
        Markets::<Runtime>::insert(1, market_mock::<Runtime>(ALICE));
        assert_ok!(Authorized::authorize_market_outcome(
            Origin::signed(ALICE),
            0,
            OutcomeReport::Scalar(123)
        ));
        assert_ok!(Authorized::authorize_market_outcome(
            Origin::signed(ALICE),
            1,
            OutcomeReport::Scalar(456)
        ));
        assert_eq!(
            AuthorizedOutcomeReports::<Runtime>::get(0).unwrap(),
            OutcomeReport::Scalar(123)
        );
        assert_eq!(
            AuthorizedOutcomeReports::<Runtime>::get(1).unwrap(),
            OutcomeReport::Scalar(456)
        );
    });
}
