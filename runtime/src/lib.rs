#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

extern crate alloc;

#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

// Prohibit trying to build both runtimes or no runtime at all
#[cfg(any(
    all(feature = "runtime-battery-station", feature = "runtime-zeitgeist"),
    not(any(feature = "runtime-battery-station", feature = "runtime-zeitgeist"))
))]
compile_error!("Only exactly one feature of the following is required: runtime-battery-station, runtime-zeitgeist (default)");

mod common;
#[cfg(feature = "runtime-battery-station")]
pub mod battery_station;
#[cfg(feature = "runtime-zeitgeist")]
pub mod zeitgeist;

// Expose runtime
#[cfg(feature = "runtime-battery-station")]
pub use battery_station::Runtime;
#[cfg(feature = "runtime-zeitgeist")]
pub use zeitgeist::Runtime;

// Expose functions and types required to construct node CLI
pub use common::{opaque::Block, SignedPayload};
#[cfg(feature = "std")]
pub use common::native_version;