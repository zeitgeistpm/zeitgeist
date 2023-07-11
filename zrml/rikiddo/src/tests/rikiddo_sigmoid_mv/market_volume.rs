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

use substrate_fixed::{types::extra::U64, FixedU128};

use super::{ema_create_test_struct, Rikiddo};
use crate::{
    traits::RikiddoMV,
    types::{FeeSigmoid, RikiddoConfig, TimestampedVolume},
};

#[test]
fn rikiddo_updates_mv_and_returns_some() {
    let emv = ema_create_test_struct(1, 2.0);
    let mut rikiddo =
        Rikiddo::new(RikiddoConfig::default(), FeeSigmoid::default(), emv.clone(), emv);
    rikiddo.update_volume(&TimestampedVolume { timestamp: 0, volume: 1u32.into() }).unwrap();
    let res =
        rikiddo.update_volume(&TimestampedVolume { timestamp: 2, volume: 2u32.into() }).unwrap();
    assert_eq!(res, Some(1u32.into()));
}

#[test]
fn rikiddo_updates_mv_and_returns_none() {
    let mut rikiddo = Rikiddo::default();
    let vol = TimestampedVolume::default();
    assert_eq!(rikiddo.update_volume(&vol).unwrap(), None);
}

#[test]
fn rikiddo_clear_clears_market_data() {
    let mut rikiddo = Rikiddo::default();
    let rikiddo_clone = rikiddo.clone();
    let _ = rikiddo.update_volume(&<TimestampedVolume<FixedU128<U64>>>::default());
    assert_ne!(rikiddo, rikiddo_clone);
    rikiddo.clear();
    assert_eq!(rikiddo, rikiddo_clone);
}
