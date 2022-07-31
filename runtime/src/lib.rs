#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

extern crate alloc;

#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

mod common;
#[cfg(feature = "testnet")]
pub mod battery_station;
#[cfg(not(feature = "testnet"))]
pub mod zeitgeist;

// Expose runtime
#[cfg(feature = "testnet")]
pub use battery_station::Runtime;
#[cfg(not(feature = "testnet"))]
pub use zeitgeist::Runtime;

// Expose functions and types required to construct node CLI
pub use common::{opaque::Block, SignedPayload};
#[cfg(feature = "std")]
pub use common::native_version;