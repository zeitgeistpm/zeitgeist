#[cfg(feature = "parachain")]
mod cli_parachain;

#[cfg(feature = "parachain")]
pub use cli_parachain::RelayChainCli;

use clap::Parser;
use sc_cli::{ChainSpec, RuntimeVersion, SubstrateCli};

const COPYRIGHT_START_YEAR: i32 = 2021;
const IMPL_NAME: &str = "Zeitgeist Node";
const SUPPORT_URL: &str = "https://github.com/zeitgeistpm/zeitgeist/issues";

#[cfg(feature = "parachain")]
type RunCmd = cumulus_client_cli::RunCmd;
#[cfg(not(feature = "parachain"))]
type RunCmd = sc_cli::RunCmd;

#[derive(Debug, clap::Subcommand)]
pub enum Subcommand {
    /// The custom benchmark subcommmand benchmarking runtime pallets.
    #[cfg(feature = "runtime-benchmarks")]
    #[clap(name = "benchmark", about = "Benchmark runtime pallets.")]
    Benchmark(frame_benchmarking_cli::BenchmarkCmd),

    /// Build a chain specification.
    BuildSpec(sc_cli::BuildSpecCmd),

    /// Validate blocks.
    CheckBlock(sc_cli::CheckBlockCmd),

    /// Export blocks.
    ExportBlocks(sc_cli::ExportBlocksCmd),

    /// Export block header in hex format.
    #[cfg(feature = "parachain")]
    ExportHeader(sc_cli::CheckBlockCmd),

    /// Export the genesis state of the parachain.
    #[cfg(feature = "parachain")]
    #[clap(name = "export-genesis-state")]
    ExportGenesisState(cli_parachain::ExportGenesisStateCommand),

    /// Export the genesis wasm of the parachain.
    #[cfg(feature = "parachain")]
    #[clap(name = "export-genesis-wasm")]
    ExportGenesisWasm(cli_parachain::ExportGenesisWasmCommand),

    /// Export the state of a given block into a chain spec.
    ExportState(sc_cli::ExportStateCmd),

    /// Import blocks.
    ImportBlocks(sc_cli::ImportBlocksCmd),

    /// Key management cli utilities
    #[clap(subcommand)]
    Key(sc_cli::KeySubcommand),

    #[cfg(feature = "parachain")]
    /// Remove the whole chain.
    PurgeChain(cumulus_client_cli::PurgeChainCmd),

    #[cfg(not(feature = "parachain"))]
    /// Remove the whole chain.
    PurgeChain(sc_cli::PurgeChainCmd),

    /// Revert the chain to a previous state.
    Revert(sc_cli::RevertCmd),

    /// Try some command against runtime state.
    #[cfg(feature = "try-runtime")]
    TryRuntime(try_runtime_cli::TryRuntimeCmd),

    /// Try some command against runtime state. Note: `try-runtime` feature must be enabled.
    #[cfg(not(feature = "try-runtime"))]
    TryRuntime,
}

#[derive(Debug, Parser)]
#[clap(
    propagate_version = true,
    args_conflicts_with_subcommands = true,
    subcommand_negates_reqs = true
)]
pub struct Cli {
    #[clap(flatten)]
    pub run: RunCmd,

    #[clap(subcommand)]
    pub subcommand: Option<Subcommand>,

    /// Relaychain arguments
    #[cfg(feature = "parachain")]
    #[clap(raw = true)]
    pub relaychain_args: Vec<String>,

    /// Id of the parachain this collator collates for.
    #[cfg(feature = "parachain")]
    #[clap(long)]
    pub parachain_id: Option<u32>,
}

impl SubstrateCli for Cli {
    fn author() -> String {
        env!("CARGO_PKG_AUTHORS").into()
    }

    fn copyright_start_year() -> i32 {
        COPYRIGHT_START_YEAR
    }

    fn description() -> String {
        env!("CARGO_PKG_DESCRIPTION").into()
    }

    fn impl_name() -> String {
        IMPL_NAME.into()
    }

    fn impl_version() -> String {
        env!("SUBSTRATE_CLI_IMPL_VERSION").into()
    }

    fn load_spec(&self, id: &str) -> Result<Box<dyn sc_service::ChainSpec>, String> {
        load_spec(
            id,
            #[cfg(feature = "parachain")]
            match id {
                "battery_station_staging" => {
                    self.parachain_id.unwrap_or(super::BATTERY_STATION_PARACHAIN_ID).into()
                }
                _ => self.parachain_id.unwrap_or(super::KUSAMA_PARACHAIN_ID).into(),
            },
        )
    }

    fn native_runtime_version(_: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
        &zeitgeist_runtime::VERSION
    }

    fn support_url() -> String {
        SUPPORT_URL.into()
    }
}

pub fn load_spec(
    id: &str,
    #[cfg(feature = "parachain")] parachain_id: cumulus_primitives_core::ParaId,
) -> Result<Box<dyn sc_service::ChainSpec>, String> {
    Ok(match id {
        "" | "dev" => Box::new(crate::chain_spec::dev_config(
            #[cfg(feature = "parachain")]
            parachain_id,
        )?),
        "battery_station" => Box::new(crate::chain_spec::ChainSpec::from_json_bytes(
            #[cfg(feature = "parachain")]
            &include_bytes!("../res/bs_parachain.json")[..],
            #[cfg(not(feature = "parachain"))]
            &include_bytes!("../res/bs.json")[..],
        )?),
        "battery_station_staging" => Box::new(crate::chain_spec::battery_station_staging_config(
            #[cfg(feature = "parachain")]
            parachain_id,
        )?),
        "zeitgeist" => Box::new(crate::chain_spec::ChainSpec::from_json_bytes(
            #[cfg(feature = "parachain")]
            &include_bytes!("../res/zeitgeist_parachain.json")[..],
            #[cfg(not(feature = "parachain"))]
            &include_bytes!("../res/zeitgeist.json")[..],
        )?),
        "zeitgeist_staging" => Box::new(crate::chain_spec::zeitgeist_staging_config(
            #[cfg(feature = "parachain")]
            parachain_id,
        )?),
        path => {
            Box::new(crate::chain_spec::ChainSpec::from_json_file(std::path::PathBuf::from(path))?)
        }
    })
}
