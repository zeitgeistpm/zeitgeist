// Copyright 2022 Forecasting Technologies LTD.
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
    global_disputes_pallet_api::GlobalDisputesPalletApi,
    mock::*,
    types::{OutcomeInfo, WinnerInfo},
    Error, Event, Locks, MarketIdOf, Outcomes, Winners,
};
use frame_support::{
    assert_noop, assert_ok,
    traits::{Currency, ReservableCurrency},
    BoundedVec,
};
use pallet_balances::{BalanceLock, Error as BalancesError};
use sp_runtime::traits::Zero;
use zeitgeist_primitives::{
    constants::mock::{GlobalDisputeLockId, MinOutcomeVoteAmount, VotingOutcomeFee, BASE},
    types::OutcomeReport,
};

#[test]
fn crowdfund_works() {
    ExtBuilder::default().build().execute_with(|| {

    });
}
