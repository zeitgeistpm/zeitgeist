// Copyright 2022 Zeitgeist PM LLC.
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

/// Listing of parachains we integrate with.
/// For each parachain, we are interested in stating their parachain ID
/// as well as any of their token key ID that we possibly support in our
/// XCM configuration. These token keys are defined in said parachain
/// and must always match the value there defined, which is expected to
/// never change once defined since they help define the canonical id
/// of said tokens in the network, which is relevant for XCM transfers.
pub mod karura {
    pub const ID: u32 = 2000;
    pub const AUSD_KEY: &[u8] = &[0, 129];
}

pub mod zeitgeist {
    pub const ID: u32 = 2101;
    pub const ZTG_KEY: &[u8] = &[0, 1];
}
