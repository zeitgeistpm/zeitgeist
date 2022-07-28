#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

extern crate alloc;

#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

mod common;
#[cfg(feature = "runtime-battery-station")]
pub mod battery_station;
#[cfg(feature = "runtime-zeitgeist")]
pub mod zeitgeist;