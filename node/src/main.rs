#![warn(unused_extern_crates)]

mod chain_spec;
mod cli;
mod command;
mod rpc;
#[macro_use]
mod service;

#[cfg(feature = "parachain")]
const KUSAMA_PARACHAIN_ID: u32 = 2101;
#[cfg(feature = "parachain")]
const BATTERY_STATION_PARACHAIN_ID: u32 = 2050;

fn main() -> sc_cli::Result<()> {
    command::run()
}
