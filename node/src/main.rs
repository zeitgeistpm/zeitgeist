#![warn(unused_extern_crates)]

mod chain_spec;
mod cli;
mod command;
mod rpc;
#[macro_use]
mod service;

#[cfg(feature = "parachain")]
const DEFAULT_PARACHAIN_ID: u32 = 2050;

fn main() -> sc_cli::Result<()> {
    command::run()
}
