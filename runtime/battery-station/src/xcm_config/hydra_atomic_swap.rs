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
#![cfg(feature = "parachain")]

use crate::RuntimeCall;
use cumulus_primitives_core::Xcm;
use frame_support::{parameter_types, traits::Contains};
use xcm::prelude::*;

parameter_types! {
    pub const BasiliskParachainId: u32 = 2090;
    pub BasiliskMultiLocation: MultiLocation = MultiLocation::new(1, X1(Parachain(BasiliskParachainId::get())));
}

pub struct AllowHydraDxAtomicSwap;

impl Contains<(MultiLocation, Xcm<RuntimeCall>)> for AllowHydraDxAtomicSwap {
    fn contains((ref origin, ref msg): &(MultiLocation, Xcm<RuntimeCall>)) -> bool {
        // allow root to execute XCM
        if origin == &MultiLocation::here() {
            return true;
        }

        // TODO just copy the same code as for the hydra dx main net filter since the basilisk node uses the same code as the mainnet here https://github.com/galacticcouncil/Basilisk-node/blob/24ffc88d5cbc75e2f00f43c95f7d48db2d3a618f/Cargo.toml#L30
        false
    }
}
