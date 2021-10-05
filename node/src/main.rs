mod chain_spec;
mod cli;
mod command;
#[macro_use]
mod service;

const DEFAULT_PARACHAIN_ID: u32 = 2050;

fn main() -> sc_cli::Result<()> {
    command::run()
}
