#![cfg(test)]

use crate::{
    market_mock,
    mock::{Authorized, ExtBuilder, Origin, Runtime, ALICE, BOB},
    Error, Outcomes,
};
use frame_support::assert_noop;
use zeitgeist_primitives::{
    traits::DisputeApi,
    types::{MarketDisputeMechanism, OutcomeReport},
};
use zrml_market_commons::Markets;

#[test]
fn authorize_market_outcome_inserts_a_new_outcome() {
    ExtBuilder::default().build().execute_with(|| {
        Markets::<Runtime>::insert(0, market_mock::<Runtime>(ALICE));
        Authorized::authorize_market_outcome(Origin::signed(ALICE), OutcomeReport::Scalar(1))
            .unwrap();
        assert_eq!(Outcomes::<Runtime>::get(0, ALICE).unwrap(), OutcomeReport::Scalar(1));
    });
}

#[test]
fn authorize_market_outcome_forbids_accounts_without_an_authorized_market() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            Authorized::authorize_market_outcome(Origin::signed(ALICE), OutcomeReport::Scalar(1)),
            Error::<Runtime>::AccountIsNotLinkedToAnyAuthorizedMarket
        );
    });
}

#[test]
fn authorize_market_outcome_forbids_more_than_one_outcome_for_a_market() {
    ExtBuilder::default().build().execute_with(|| {
        Markets::<Runtime>::insert(0, market_mock::<Runtime>(ALICE));
        Authorized::authorize_market_outcome(Origin::signed(ALICE), OutcomeReport::Scalar(1))
            .unwrap();
        Markets::<Runtime>::mutate(0, |el| {
            el.as_mut().unwrap().mdm = MarketDisputeMechanism::Authorized(BOB);
        });
        assert_noop!(
            Authorized::authorize_market_outcome(Origin::signed(BOB), OutcomeReport::Scalar(1)),
            Error::<Runtime>::MarketsCanNotHaveMoreThanOneAuthorizedAccount
        );
    });
}

#[test]
fn on_dispute_denies_non_authorized_markets() {
    ExtBuilder::default().build().execute_with(|| {
        let mut market = market_mock::<Runtime>(ALICE);
        market.mdm = MarketDisputeMechanism::Court;
        assert_noop!(
            Authorized::on_dispute(&[], &0, &market),
            Error::<Runtime>::MarketDoesNotHaveAuthorizedMechanism
        );
    });
}

#[test]
fn on_resolution_denies_non_authorized_markets() {
    ExtBuilder::default().build().execute_with(|| {
        let mut market = market_mock::<Runtime>(ALICE);
        market.mdm = MarketDisputeMechanism::Court;
        assert_noop!(
            Authorized::on_resolution(&[], &0, &market),
            Error::<Runtime>::MarketDoesNotHaveAuthorizedMechanism
        );
    });
}

#[test]
fn on_resolution_must_demand_an_already_included_outcome() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            Authorized::on_resolution(&[], &0, &market_mock::<Runtime>(ALICE)),
            Error::<Runtime>::UnknownOutcome
        );
    });
}

#[test]
fn on_resolution_removes_stored_outcomes() {
    ExtBuilder::default().build().execute_with(|| {
        let market = market_mock::<Runtime>(ALICE);
        Markets::<Runtime>::insert(0, &market);
        Authorized::authorize_market_outcome(Origin::signed(ALICE), OutcomeReport::Scalar(1))
            .unwrap();
        let _ = Authorized::on_resolution(&[], &0, &market).unwrap();
        assert_eq!(Outcomes::<Runtime>::get(0, ALICE), None);
    });
}

#[test]
fn on_resolution_returns_the_canonical_outcome() {
    ExtBuilder::default().build().execute_with(|| {
        let market = market_mock::<Runtime>(ALICE);
        Markets::<Runtime>::insert(0, &market);
        Authorized::authorize_market_outcome(Origin::signed(ALICE), OutcomeReport::Scalar(1))
            .unwrap();
        assert_eq!(Authorized::on_resolution(&[], &0, &market).unwrap(), OutcomeReport::Scalar(1));
    });
}
