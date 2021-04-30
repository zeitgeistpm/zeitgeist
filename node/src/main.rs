#![warn(unused_extern_crates)]

#[macro_use]
mod service;
mod cli;
mod command;

#[cfg(feature = "parachain")]
const DEFAULT_PARACHAIN_ID: u32 = 9123;

fn main() -> sc_cli::Result<()> {
    command::run()
}
