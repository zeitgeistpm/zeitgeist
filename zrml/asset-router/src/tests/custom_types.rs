// Copyright 2024 Forecasting Technologies LTD.
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

type Aid = AssetInDestruction<u32>;

#[test]
fn asset_in_destruction_created_properly() {
    let aid = Aid::new(2);
    assert_eq!(*aid.asset(), 2);
    assert_eq!(*aid.state(), DestructionState::Accounts);
}

#[test]
fn asset_in_destruction_transitions_states_properly() {
    let mut aid = Aid::new(2);

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
    let mut aid = Aid::new(2);

    aid.transit_indestructible();
    assert_eq!(*aid.state(), DestructionState::Indestructible);

    aid.transit_state();
    assert_eq!(*aid.state(), DestructionState::Indestructible);
}

#[test]
fn asset_in_destruction_ordering_works() {
    // Order by destruction state first.
    let asset_1 = Aid::new(0);
    let mut asset_2 = asset_1;
    assert_eq!(asset_2.transit_state(), Some(&DestructionState::Approvals));
    let mut asset_3 = asset_2;
    assert_eq!(asset_3.transit_state(), Some(&DestructionState::Finalization));
    let mut asset_4 = asset_3;
    assert_eq!(asset_4.transit_state(), Some(&DestructionState::Destroyed));
    let mut asset_5 = asset_1;
    asset_5.transit_indestructible();

    let mut asset_vec = vec![asset_5, asset_4, asset_3, asset_2, asset_1];
    let mut expected = vec![asset_1, asset_2, asset_3, asset_4, asset_5];
    asset_vec.sort();
    assert_eq!(asset_vec, expected);

    // On equal destruction state, order by asset id.
    let mut asset_dif_id_1 = Aid::new(1);
    asset_dif_id_1.transit_state();
    let mut asset_dif_id_2 = Aid::new(2);
    asset_dif_id_2.transit_state();

    asset_vec.push(asset_dif_id_1);
    asset_vec.push(asset_dif_id_2);
    asset_vec.sort();
    expected = vec![asset_1, asset_dif_id_2, asset_dif_id_1, asset_2, asset_3, asset_4, asset_5];
    assert_eq!(asset_vec, expected);
}
