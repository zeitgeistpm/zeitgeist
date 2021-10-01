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
