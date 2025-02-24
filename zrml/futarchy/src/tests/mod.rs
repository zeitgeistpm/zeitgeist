// Copyright 2024-2025 Forecasting Technologies LTD.
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

#![cfg(all(feature = "mock", test))]

mod submit_proposal;

use crate::{
    mock::{
        ext_builder::ExtBuilder,
        runtime::{Futarchy, Runtime, RuntimeOrigin, System},
        types::{MockOracle, MockScheduler},
        utility,
    },
    types::Proposal,
    Config, Error, Event, Proposals, ProposalsOf,
};
use frame_support::{
    assert_noop, assert_ok,
    dispatch::RawOrigin,
    traits::{schedule::DispatchTime, Bounded},
};
use sp_runtime::DispatchError;

/// Utility struct for managing test accounts.
pub(crate) struct Account {
    id: <Runtime as frame_system::Config>::AccountId,
}

impl Account {
    // TODO Not a pressing issue, but double booking accounts should be illegal.
    pub(crate) fn new(id: <Runtime as frame_system::Config>::AccountId) -> Account {
        Account { id }
    }

    pub(crate) fn signed(&self) -> RuntimeOrigin {
        RuntimeOrigin::signed(self.id)
    }
}
