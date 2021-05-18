//! Opaque types. These are used by the CLI to instantiate machinery that don't need to know
//! the specifics of the runtime. They can then be made to be agnostic over specific formats
//! of data like extrinsics, allowing for them to continue syncing the network through upgrades
//! to even the core data structures.

pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

use crate::Header;
use sp_runtime::{generic, impl_opaque_keys};
use sp_std::vec::Vec;

pub type Block = generic::Block<Header, UncheckedExtrinsic>;
pub type SessionHandlers = ();

#[cfg(feature = "parachain")]
impl_opaque_keys! {
    pub struct SessionKeys {
        pub author_inherent: crate::AuthorInherent,
    }
}

#[cfg(not(feature = "parachain"))]
impl_opaque_keys! {
    pub struct SessionKeys {
        pub aura: crate::Aura,
        pub grandpa: crate::Grandpa,
    }
}
