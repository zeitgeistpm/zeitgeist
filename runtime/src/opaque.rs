//! Opaque types. These are used by the CLI to instantiate machinery that don't need to know
//! the specifics of the runtime. They can then be made to be agnostic over specific formats
//! of data like extrinsics, allowing for them to continue syncing the network through upgrades
//! to even the core data structures.

use super::*;

pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

/// Opaque block type.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;

pub type SessionHandlers = ();

#[cfg(feature = "parachain")]
impl_opaque_keys! {
    pub struct SessionKeys {
    }
}

#[cfg(not(feature = "parachain"))]
impl_opaque_keys! {
    pub struct SessionKeys {
        pub aura: Aura,
        pub grandpa: Grandpa,
    }
}
