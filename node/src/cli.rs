#[cfg(feature = "parachain")]
mod cli_parachain;

#[cfg(feature = "parachain")]
pub use cli_parachain::RelayChainCli;

use sc_cli::{ChainSpec, RuntimeVersion, SubstrateCli};

const COPYRIGHT_START_YEAR: i32 = 2021;
const IMPL_NAME: &str = "Zeitgeist Node";
const SUPPORT_URL: &str = "support.anonymous.an";

#[cfg(feature = "parachain")]
type RunCmd = cli_parachain::RunCmd;
#[cfg(not(feature = "parachain"))]
type RunCmd = sc_cli::RunCmd;

#[derive(Debug, structopt::StructOpt)]
pub enum Subcommand {
    /// The custom benchmark subcommmand benchmarking runtime pallets.
    #[cfg(feature = "runtime-benchmark")]
    #[structopt(name = "benchmark", about = "Benchmark runtime pallets.")]
    Benchmark(frame_benchmarking_cli::BenchmarkCmd),

    /// Build a chain specification.
    BuildSpec(sc_cli::BuildSpecCmd),

    /// Validate blocks.
    CheckBlock(sc_cli::CheckBlockCmd),

    /// Export blocks.
    ExportBlocks(sc_cli::ExportBlocksCmd),

    /// Export the genesis state of the parachain.
    #[cfg(feature = "parachain")]
    #[structopt(name = "export-genesis-state")]
    ExportGenesisState(cli_parachain::ExportGenesisStateCommand),

    /// Export the genesis wasm of the parachain.
    #[cfg(feature = "parachain")]
    #[structopt(name = "export-genesis-wasm")]
    ExportGenesisWasm(cli_parachain::ExportGenesisWasmCommand),

    /// Export the state of a given block into a chain spec.
    ExportState(sc_cli::ExportStateCmd),

    /// Import blocks.
    ImportBlocks(sc_cli::ImportBlocksCmd),

    #[cfg(not(feature = "parachain"))]
    /// Key management cli utilities
    Key(sc_cli::KeySubcommand),

    #[cfg(feature = "parachain")]
    /// Remove the whole chain.
    PurgeChain(cumulus_client_cli::PurgeChainCmd),

    #[cfg(not(feature = "parachain"))]
    /// Remove the whole chain.
    PurgeChain(sc_cli::PurgeChainCmd),

    /// Revert the chain to a previous state.
    Revert(sc_cli::RevertCmd),
}

#[derive(Debug, structopt::StructOpt)]
#[structopt(settings = &[
	structopt::clap::AppSettings::GlobalVersion,
	structopt::clap::AppSettings::ArgsNegateSubcommands,
	structopt::clap::AppSettings::SubcommandsNegateReqs,
])]
pub struct Cli {
    #[structopt(flatten)]
    pub run: RunCmd,

    #[structopt(subcommand)]
    pub subcommand: Option<Subcommand>,

    /// Run node as collator.
    ///
    /// Note that this is the same as running with `--validator`.
    #[cfg(feature = "parachain")]
    #[structopt(long, conflicts_with = "validator")]
    pub collator: bool,

    /// Relaychain arguments
    #[cfg(feature = "parachain")]
    #[structopt(raw = true)]
    pub relaychain_args: Vec<String>,
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
            self.run
                .parachain_id
                .unwrap_or(crate::DEFAULT_PARACHAIN_ID)
                .into(),
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
        "dev" => Box::new(crate::chain_spec::dev_config(
            #[cfg(feature = "parachain")]
            parachain_id,
        )?),
        "" | "local" => Box::new(crate::chain_spec::local_testnet_config(
            #[cfg(feature = "parachain")]
            parachain_id,
        )?),
        "battery_park" => Box::new(crate::chain_spec::ChainSpec::from_json_bytes(
            #[cfg(feature = "parachain")]
            &include_bytes!("../res/bp_parachain.json")[..],
            #[cfg(not(feature = "parachain"))]
            &include_bytes!("../res/bp.json")[..],
        )?),
        "battery_park_staging" => Box::new(crate::chain_spec::battery_park_staging_config(
            #[cfg(feature = "parachain")]
            parachain_id,
        )?),
        path => Box::new(crate::chain_spec::ChainSpec::from_json_file(
            std::path::PathBuf::from(path),
        )?),
    })
}
