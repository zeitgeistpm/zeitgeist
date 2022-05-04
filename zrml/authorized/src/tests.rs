#![cfg(test)]

use crate::{
    market_mock,
    mock::{Authorized, ExtBuilder, Origin, Runtime, ALICE, BOB},
    Error, Outcomes,
};
use frame_support::assert_noop;
use zeitgeist_primitives::{
    traits::DisputeApi,
    types::{MarketDisputeMechanism, MarketStatus, OutcomeReport},
};
use zrml_market_commons::Markets;

#[test]
fn authorize_market_outcome_inserts_a_new_outcome() {
    ExtBuilder::default().build().execute_with(|| {
        Markets::<Runtime>::insert(0, market_mock::<Runtime>(ALICE));
        Authorized::authorize_market_outcome(Origin::signed(ALICE), 0, OutcomeReport::Scalar(1))
            .unwrap();
        assert_eq!(Outcomes::<Runtime>::get(0).unwrap(), OutcomeReport::Scalar(1));
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
        let mut market = market_mock::<Runtime>(ALICE);
        market.status = MarketStatus::Active;
        Markets::<Runtime>::insert(0, market);
        assert_noop!(
            Authorized::authorize_market_outcome(
                Origin::signed(ALICE),
                0,
                OutcomeReport::Categorical(123)
            ),
            Error::<Runtime>::MarketDoesNotHaveDisputeMechanismAuthorized
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
        assert_noop!(
            Authorized::on_resolution(&[], &0, &market_mock::<Runtime>(ALICE)),
            Error::<Runtime>::ReportNotFound
        );
    });
}

#[test]
fn on_resolution_removes_stored_outcomes() {
    ExtBuilder::default().build().execute_with(|| {
        let market = market_mock::<Runtime>(ALICE);
        Markets::<Runtime>::insert(0, &market);
        Authorized::authorize_market_outcome(Origin::signed(ALICE), 0, OutcomeReport::Scalar(1))
            .unwrap();
        let _ = Authorized::on_resolution(&[], &0, &market).unwrap();
        assert_eq!(Outcomes::<Runtime>::get(0), None);
    });
}

#[test]
fn on_resolution_returns_the_reported_outcome() {
    ExtBuilder::default().build().execute_with(|| {
        let market = market_mock::<Runtime>(ALICE);
        Markets::<Runtime>::insert(0, &market);
        Authorized::authorize_market_outcome(Origin::signed(ALICE), 0, OutcomeReport::Scalar(1))
            .unwrap();
        assert_eq!(Authorized::on_resolution(&[], &0, &market).unwrap(), OutcomeReport::Scalar(1));
    });
}
