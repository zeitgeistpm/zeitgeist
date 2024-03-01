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
use crate::MarketIdsForEdit;
use sp_runtime::DispatchError;

// TODO(#1239) request_edit fails if market is not proposed
// TODO(#1239) request_edit fails if edit already in progress

#[test]
fn it_allows_request_edit_origin_to_request_edits_for_markets() {
    ExtBuilder::default().build().execute_with(|| {
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Advised,
            2..4,
            ScoringRule::Lmsr,
        );

        // make sure it's in status proposed
        let market = MarketCommons::market(&0);
        assert_eq!(market.unwrap().status, MarketStatus::Proposed);

        let edit_reason = vec![0_u8; <Runtime as Config>::MaxEditReasonLen::get() as usize];
        // Make sure it fails from the random joe
        assert_noop!(
            PredictionMarkets::request_edit(RuntimeOrigin::signed(BOB), 0, edit_reason.clone()),
            DispatchError::BadOrigin
        );

        // Now it should work from SUDO
        assert_ok!(PredictionMarkets::request_edit(
            RuntimeOrigin::signed(SUDO),
            0,
            edit_reason.clone()
        ));
        System::assert_last_event(
            Event::MarketRequestedEdit(
                0,
                edit_reason.try_into().expect("Conversion to BoundedVec failed"),
            )
            .into(),
        );

        assert!(MarketIdsForEdit::<Runtime>::contains_key(0));
    });
}

#[test]
fn request_edit_fails_on_bad_origin() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        // Creates an advised market.
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Advised,
            2..4,
            ScoringRule::Lmsr,
        );

        // make sure it's in status proposed
        let market = MarketCommons::market(&0);
        assert_eq!(market.unwrap().status, MarketStatus::Proposed);

        let edit_reason = vec![0_u8; <Runtime as Config>::MaxEditReasonLen::get() as usize];
        // Make sure it fails from the random joe
        assert_noop!(
            PredictionMarkets::request_edit(RuntimeOrigin::signed(BOB), 0, edit_reason),
            DispatchError::BadOrigin
        );
    });
}

#[test]
fn edit_request_fails_if_edit_reason_is_too_long() {
    ExtBuilder::default().build().execute_with(|| {
        // Creates an advised market.
        simple_create_categorical_market(
            Asset::Ztg,
            MarketCreation::Advised,
            0..2,
            ScoringRule::Lmsr,
        );

        let market = MarketCommons::market(&0);
        assert_eq!(market.unwrap().status, MarketStatus::Proposed);

        let edit_reason = vec![0_u8; <Runtime as Config>::MaxEditReasonLen::get() as usize + 1];

        assert_noop!(
            PredictionMarkets::request_edit(RuntimeOrigin::signed(SUDO), 0, edit_reason),
            Error::<Runtime>::EditReasonLengthExceedsMaxEditReasonLen
        );
    });
}
