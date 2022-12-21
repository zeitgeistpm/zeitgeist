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

#![cfg(test)]

use crate::{
    market_mock,
    mock::{Authorized, AuthorizedDisputeResolutionUser, ExtBuilder, Origin, Runtime, BOB},
    mock_storage::pallet as mock_storage,
    AuthorizedOutcomeReports, Error,
};
use frame_support::{assert_noop, assert_ok, dispatch::DispatchError};
use zeitgeist_primitives::{
    traits::{DisputeApi, DisputeResolutionApi},
    types::{AuthorityReport, MarketDispute, MarketDisputeMechanism, MarketStatus, OutcomeReport},
};
use zrml_market_commons::Markets;

#[test]
fn authorize_market_outcome_inserts_a_new_outcome() {
    ExtBuilder::default().build().execute_with(|| {
        Markets::<Runtime>::insert(0, market_mock::<Runtime>());
        assert_ok!(Authorized::authorize_market_outcome(
            Origin::signed(AuthorizedDisputeResolutionUser::get()),
            0,
            OutcomeReport::Scalar(1)
        ));
        let now = frame_system::Pallet::<Runtime>::block_number();
        let resolve_at = now + <Runtime as crate::Config>::CorrectionPeriod::get();
        assert_eq!(
            AuthorizedOutcomeReports::<Runtime>::get(0).unwrap(),
            AuthorityReport { outcome: OutcomeReport::Scalar(1), resolve_at }
        );
    });
}

#[test]
fn authorize_market_outcome_resets_dispute_resolution() {
    ExtBuilder::default().build().execute_with(|| {
        Markets::<Runtime>::insert(0, market_mock::<Runtime>());

        assert_ok!(Authorized::authorize_market_outcome(
            Origin::signed(AuthorizedDisputeResolutionUser::get()),
            0,
            OutcomeReport::Scalar(1),
        ));
        let now = frame_system::Pallet::<Runtime>::block_number();
        let resolve_at_0 = now + <Runtime as crate::Config>::CorrectionPeriod::get();
        assert_eq!(
            AuthorizedOutcomeReports::<Runtime>::get(0).unwrap(),
            AuthorityReport { outcome: OutcomeReport::Scalar(1), resolve_at: resolve_at_0 }
        );

        assert_eq!(mock_storage::MarketIdsPerDisputeBlock::<Runtime>::get(resolve_at_0), vec![0]);

        frame_system::Pallet::<Runtime>::set_block_number(resolve_at_0 - 1);
        let now = frame_system::Pallet::<Runtime>::block_number();
        let resolve_at_1 = now + <Runtime as crate::Config>::CorrectionPeriod::get();

        assert_ok!(Authorized::authorize_market_outcome(
            Origin::signed(AuthorizedDisputeResolutionUser::get()),
            0,
            OutcomeReport::Scalar(2)
        ));

        assert_eq!(mock_storage::MarketIdsPerDisputeBlock::<Runtime>::get(resolve_at_0), vec![]);
        assert_eq!(mock_storage::MarketIdsPerDisputeBlock::<Runtime>::get(resolve_at_1), vec![0]);
    });
}

#[test]
fn authorize_market_outcome_fails_if_market_does_not_exist() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            Authorized::authorize_market_outcome(
                Origin::signed(AuthorizedDisputeResolutionUser::get()),
                0,
                OutcomeReport::Scalar(1)
            ),
            zrml_market_commons::Error::<Runtime>::MarketDoesNotExist
        );
    });
}

#[test]
fn authorize_market_outcome_fails_on_non_authorized_market() {
    ExtBuilder::default().build().execute_with(|| {
        let mut market = market_mock::<Runtime>();
        market.dispute_mechanism = MarketDisputeMechanism::Court;
        Markets::<Runtime>::insert(0, market);
        assert_noop!(
            Authorized::authorize_market_outcome(
                Origin::signed(AuthorizedDisputeResolutionUser::get()),
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
        let mut market = market_mock::<Runtime>();
        market.status = MarketStatus::Active;
        Markets::<Runtime>::insert(0, market);
        assert_noop!(
            Authorized::authorize_market_outcome(
                Origin::signed(AuthorizedDisputeResolutionUser::get()),
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
        Markets::<Runtime>::insert(0, market_mock::<Runtime>());
        assert_noop!(
            Authorized::authorize_market_outcome(
                Origin::signed(AuthorizedDisputeResolutionUser::get()),
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
        Markets::<Runtime>::insert(0, market_mock::<Runtime>());
        assert_noop!(
            Authorized::authorize_market_outcome(Origin::signed(BOB), 0, OutcomeReport::Scalar(1)),
            DispatchError::BadOrigin,
        );
    });
}

#[test]
fn on_resolution_fails_if_no_report_was_submitted() {
    ExtBuilder::default().build().execute_with(|| {
        let report = Authorized::on_resolution(&[], &0, &market_mock::<Runtime>()).unwrap();
        assert!(report.is_none());
    });
}

#[test]
fn on_resolution_removes_stored_outcomes() {
    ExtBuilder::default().build().execute_with(|| {
        let market = market_mock::<Runtime>();
        Markets::<Runtime>::insert(0, &market);
        assert_ok!(Authorized::authorize_market_outcome(
            Origin::signed(AuthorizedDisputeResolutionUser::get()),
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
        let market = market_mock::<Runtime>();
        Markets::<Runtime>::insert(0, &market);
        // Authorize outcome, then overwrite it.
        assert_ok!(Authorized::authorize_market_outcome(
            Origin::signed(AuthorizedDisputeResolutionUser::get()),
            0,
            OutcomeReport::Scalar(1)
        ));
        assert_ok!(Authorized::authorize_market_outcome(
            Origin::signed(AuthorizedDisputeResolutionUser::get()),
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
fn authorized_market_outcome_can_handle_multiple_markets() {
    ExtBuilder::default().build().execute_with(|| {
        Markets::<Runtime>::insert(0, market_mock::<Runtime>());
        Markets::<Runtime>::insert(1, market_mock::<Runtime>());
        assert_ok!(Authorized::authorize_market_outcome(
            Origin::signed(AuthorizedDisputeResolutionUser::get()),
            0,
            OutcomeReport::Scalar(123)
        ));
        assert_ok!(Authorized::authorize_market_outcome(
            Origin::signed(AuthorizedDisputeResolutionUser::get()),
            1,
            OutcomeReport::Scalar(456)
        ));
        let now = frame_system::Pallet::<Runtime>::block_number();
        let resolve_at = now + <Runtime as crate::Config>::CorrectionPeriod::get();
        assert_eq!(
            AuthorizedOutcomeReports::<Runtime>::get(0).unwrap(),
            AuthorityReport { outcome: OutcomeReport::Scalar(123), resolve_at }
        );
        assert_eq!(
            AuthorizedOutcomeReports::<Runtime>::get(1).unwrap(),
            AuthorityReport { outcome: OutcomeReport::Scalar(456), resolve_at }
        );
    });
}

#[test]
fn get_auto_resolve_works() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(42);
        let market = market_mock::<Runtime>();
        Markets::<Runtime>::insert(0, &market);
        assert_ok!(Authorized::authorize_market_outcome(
            Origin::signed(AuthorizedDisputeResolutionUser::get()),
            0,
            OutcomeReport::Scalar(1)
        ));
        let now = frame_system::Pallet::<Runtime>::block_number();
        let resolve_at = now + <Runtime as crate::Config>::CorrectionPeriod::get();
        assert_eq!(Authorized::get_auto_resolve(&[], &0, &market).unwrap(), Some(resolve_at),);
    });
}

#[test]
fn get_auto_resolve_returns_none_without_market_storage() {
    ExtBuilder::default().build().execute_with(|| {
        let market = market_mock::<Runtime>();
        assert_eq!(Authorized::get_auto_resolve(&[], &0, &market).unwrap(), None,);
    });
}

#[test]
fn has_failed_works_without_report() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(42);
        let market = market_mock::<Runtime>();
        Markets::<Runtime>::insert(0, &market);
        let now = frame_system::Pallet::<Runtime>::block_number();
        let last_dispute = MarketDispute { at: now, by: BOB, outcome: OutcomeReport::Scalar(1) };

        assert!(!Authorized::has_failed(&[last_dispute.clone()], &0, &market).unwrap());

        frame_system::Pallet::<Runtime>::set_block_number(
            now + <Runtime as crate::Config>::ReportPeriod::get() + 1,
        );

        assert!(Authorized::has_failed(&[last_dispute], &0, &market).unwrap());
    });
}

#[test]
fn has_failed_works_with_renewed_reports() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(42);
        let market = market_mock::<Runtime>();
        Markets::<Runtime>::insert(0, &market);
        let now = frame_system::Pallet::<Runtime>::block_number();
        let last_dispute = MarketDispute { at: now, by: BOB, outcome: OutcomeReport::Scalar(1) };

        // assume `authorize_market_outcome` is renewed indefintiely
        // by a fallible authority (one account id)
        assert_ok!(Authorized::authorize_market_outcome(
            Origin::signed(AuthorizedDisputeResolutionUser::get()),
            0,
            OutcomeReport::Scalar(1)
        ));

        frame_system::Pallet::<Runtime>::set_block_number(
            now + <Runtime as crate::Config>::ReportPeriod::get() - 1,
        );

        assert!(!Authorized::has_failed(&[last_dispute.clone()], &0, &market).unwrap());

        frame_system::Pallet::<Runtime>::set_block_number(
            now + <Runtime as crate::Config>::ReportPeriod::get() + 1,
        );

        assert!(Authorized::has_failed(&[last_dispute], &0, &market).unwrap());
    });
}

#[test]
fn authorize_market_outcome_fails_with_report_period_expired() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(42);
        let market = market_mock::<Runtime>();
        Markets::<Runtime>::insert(0, &market);

        let dispute_at = 42;
        let last_dispute =
            MarketDispute { at: dispute_at, by: BOB, outcome: OutcomeReport::Scalar(42) };
        // get_disputes returns a sample dispute in the mock
        assert_eq!(
            <Runtime as crate::Config>::DisputeResolution::get_disputes(&0).pop().unwrap(),
            last_dispute
        );

        frame_system::Pallet::<Runtime>::set_block_number(
            dispute_at + <Runtime as crate::Config>::ReportPeriod::get() + 1,
        );

        assert_noop!(
            Authorized::authorize_market_outcome(
                Origin::signed(AuthorizedDisputeResolutionUser::get()),
                0,
                OutcomeReport::Scalar(1)
            ),
            Error::<Runtime>::ReportPeriodExpired
        );
    });
}
