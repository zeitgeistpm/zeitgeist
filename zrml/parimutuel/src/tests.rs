// Copyright 2022-2023 Forecasting Technologies LTD.
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
    mock::{Authorized, AuthorizedDisputeResolutionUser, ExtBuilder, Runtime, RuntimeOrigin, BOB},
    mock_storage::pallet as mock_storage,
    AuthorizedOutcomeReports, Error,
};
use frame_support::{assert_noop, assert_ok, dispatch::DispatchError};
use zeitgeist_primitives::{
    traits::DisputeApi,
    types::{AuthorityReport, MarketDisputeMechanism, MarketStatus, OutcomeReport},
};
use zrml_market_commons::Markets;

#[test]
fn buy_shares() {
    ExtBuilder::default().build().execute_with(|| {
        Markets::<Runtime>::insert(0, market_mock::<Runtime>());
    });
}

#[test]
fn claim_rewards() {
    ExtBuilder::default().build().execute_with(|| {
        Markets::<Runtime>::insert(0, market_mock::<Runtime>());
    });
}
