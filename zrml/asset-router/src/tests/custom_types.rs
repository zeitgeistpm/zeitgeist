// Copyright 2023-2024 Forecasting Technologies LTD.
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

use crate::{AssetInDestruction, DestructionState};

type AID = AssetInDestruction<u32>;

#[test]
fn asset_in_destruction_created_properly() {
    let aid = AID::new(2);
    assert_eq!(*aid.asset(), 2);
    assert_eq!(*aid.state(), DestructionState::Accounts);
}

#[test]
fn asset_in_destruction_transitions_states_properly() {
    let mut aid = AID::new(2);

    aid.transit_state();
    assert_eq!(*aid.state(), DestructionState::Approvals);

    aid.transit_state();
    assert_eq!(*aid.state(), DestructionState::Finalization);

    aid.transit_state();
    assert_eq!(*aid.state(), DestructionState::Destroyed);

    aid.transit_state();
    assert_eq!(*aid.state(), DestructionState::Destroyed);
}

#[test]
fn asset_in_destruction_indestructible_state_works() {
    let mut aid = AID::new(2);

    aid.transit_indestructible();
    assert_eq!(*aid.state(), DestructionState::Indestructible);

    aid.transit_state();
    assert_eq!(*aid.state(), DestructionState::Indestructible);
}
