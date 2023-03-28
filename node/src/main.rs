// Copyright 2023 Forecasting Technologies LTD.
// Copyright 2021-2022 Zeitgeist PM LLC.
//
// This file is part of Zeitgeist.
//
// Zeitgeist is free software: you can redistribute it and/or modify it
// under the terms of the GNU General Public License as published by the
// Free Software Foundation, either version 3 of the License, or (at
// your option) any later version.
//
// Zeitgeist is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Zeitgeist. If not, see <https://www.gnu.org/licenses/>.

#![warn(unused_extern_crates)]

mod benchmarking;
mod chain_spec;
mod cli;
mod command;
mod rpc;
#[macro_use]
mod service;

pub const BATTERY_STATION_RUNTIME_NOT_AVAILABLE: &str =
    "Battery Station runtime is not available. Please compile the node with `--features \
     with-battery-station-runtime` to enable it.";
pub const ZEITGEIST_RUNTIME_NOT_AVAILABLE: &str = "Zeitgeist runtime is not available. Please \
                                                   compile the node with `--features \
                                                   with-zeitgeist-runtime` to enable it.";

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
