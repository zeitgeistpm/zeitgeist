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

//! Opaque types. These are used by the CLI to instantiate machinery that don't need to know
//! the specifics of the runtime. They can then be made to be agnostic over specific formats
//! of data like extrinsics, allowing for them to continue syncing the network through upgrades
//! to even the core data structures.

use crate::Header;
use alloc::vec::Vec;
use sp_runtime::{generic, impl_opaque_keys};

pub type Block = generic::Block<Header, sp_runtime::OpaqueExtrinsic>;

#[cfg(feature = "parachain")]
impl_opaque_keys! {
    pub struct SessionKeys {
        pub nimbus: crate::AuthorInherent,
    }
}

#[cfg(not(feature = "parachain"))]
impl_opaque_keys! {
    pub struct SessionKeys {
        pub aura: crate::Aura,
        pub grandpa: crate::Grandpa,
    }
}
