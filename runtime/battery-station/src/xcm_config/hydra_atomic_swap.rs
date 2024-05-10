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
use frame_support::traits::Contains;
use xcm::latest::{prelude::AccountId32, Junctions, MultiLocation};

// TODO: Maybe name it AllowBasiliskAtomicSwap and ask HydraDX if we can test atomic swaps on Basilisk
pub struct AllowHydraDxAtomicSwap;

impl Contains<(MultiLocation, Xcm<RuntimeCall>)> for AllowHydraDxAtomicSwap {
    fn contains((ref origin, ref msg): &(MultiLocation, Xcm<RuntimeCall>)) -> bool {
        match origin {
            MultiLocation { parents: 0, interior: Junctions::X1(AccountId32 { .. }) } => {
                // TODO if msg matches HydraDX atomic swap messages then true, otherwise false

                false
            }
            _ => false,
        }
    }
}
