#![warn(unused_extern_crates)]

mod chain_spec;
mod cli;
mod command;
mod command_helper;
mod rpc;
#[macro_use]
mod service;

cfg_if::cfg_if!(
    if #[cfg(feature = "parachain")] {
        const KUSAMA_PARACHAIN_ID: u32 = 2101;
        const BATTERY_STATION_PARACHAIN_ID: u32 = 2050;
        const KUSAMA_BLOCK_DURATION: core::time::Duration = core::time::Duration::from_secs(6);
        const SOFT_DEADLINE_PERCENT: sp_runtime::Percent = sp_runtime::Percent::from_percent(100);
    }
);

fn main() -> sc_cli::Result<()> {
    command::run()
}
