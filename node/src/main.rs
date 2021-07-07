#![warn(unused_extern_crates)]

mod chain_spec;
mod cli;
mod command;
#[cfg(not(feature = "parachain"))]
mod rpc;
#[macro_use]
mod service;

#[cfg(feature = "parachain")]
const DEFAULT_PARACHAIN_ID: u32 = 2049;

fn main() -> sc_cli::Result<()> {
    command::run()
}
