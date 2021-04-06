use sc_cli::{
    self, ChainSpec, ImportParams, KeystoreParams, NetworkParams, RuntimeVersion, SharedParams,
    SubstrateCli,
};
use sc_service::config::{BasePath, PrometheusConfig};
use std::{net::SocketAddr, path::PathBuf};
use structopt::StructOpt;

/// Sub-commands supported by the collator.
#[derive(Debug, StructOpt)]
pub enum Subcommand {
    /// Export the genesis state of the parachain.
    #[structopt(name = "export-genesis-state")]
    ExportGenesisState(ExportGenesisStateCommand),

    /// Export the genesis wasm of the parachain.
    #[structopt(name = "export-genesis-wasm")]
    ExportGenesisWasm(ExportGenesisWasmCommand),

    /// Build a chain specification.
    BuildSpec(sc_cli::BuildSpecCmd),

    /// Validate blocks.
    CheckBlock(sc_cli::CheckBlockCmd),

    /// Export blocks.
    ExportBlocks(sc_cli::ExportBlocksCmd),

    /// Export the state of a given block into a chain spec.
    ExportState(sc_cli::ExportStateCmd),

    /// Import blocks.
    ImportBlocks(sc_cli::ImportBlocksCmd),

    /// Remove the whole chain.
    PurgeChain(sc_cli::PurgeChainCmd),

    /// Revert the chain to a previous state.
    Revert(sc_cli::RevertCmd),
}

#[derive(Debug, StructOpt)]
#[structopt(settings = &[
	structopt::clap::AppSettings::GlobalVersion,
	structopt::clap::AppSettings::ArgsNegateSubcommands,
	structopt::clap::AppSettings::SubcommandsNegateReqs,
])]
pub struct Cli {
    #[structopt(subcommand)]
    pub subcommand: Option<Subcommand>,

    #[structopt(flatten)]
    pub run: RunCmd,

    /// Run node as collator.
    ///
    /// Note that this is the same as running with `--validator`.
    #[structopt(long, conflicts_with = "validator")]
    pub collator: bool,

    /// Relaychain arguments
    #[structopt(raw = true)]
    pub relaychain_args: Vec<String>,
}

impl sc_cli::SubstrateCli for Cli {
    fn author() -> String {
        env!("CARGO_PKG_AUTHORS").into()
    }

    fn copyright_start_year() -> i32 {
        2017
    }

    fn description() -> String {
        "Parachain Collator Template\n\nThe command-line arguments provided first will be passed to \
     the parachain node, while the arguments provided after -- will be passed to the relaychain \
     node.\n\nrococo-collator [parachain-args] -- [relaychain-args]"
      .into()
    }

    fn impl_name() -> String {
        "Parachain Collator Template".into()
    }

    fn impl_version() -> String {
        env!("SUBSTRATE_CLI_IMPL_VERSION").into()
    }

    fn load_spec(&self, id: &str) -> Result<Box<dyn sc_service::ChainSpec>, String> {
        crate::cli::load_spec(id, self.run.parachain_id.unwrap_or(200).into())
    }

    fn native_runtime_version(_: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
        &zeitgeist_runtime::VERSION
    }

    fn support_url() -> String {
        "https://github.com/substrate-developer-hub/substrate-parachain-template/issues/new".into()
    }
}

#[derive(Debug)]
pub struct RelayChainCli {
    /// The actual relay chain cli object.
    pub base: polkadot_cli::RunCmd,

    /// The base path that should be used by the relay chain.
    pub base_path: Option<PathBuf>,

    /// Optional chain id that should be passed to the relay chain.
    pub chain_id: Option<String>,
}

impl RelayChainCli {
    /// Create a new instance of `Self`.
    pub fn new<'a>(
        base_path: Option<PathBuf>,
        chain_id: Option<String>,
        relay_chain_args: impl Iterator<Item = &'a String>,
    ) -> Self {
        Self {
            base_path,
            chain_id,
            base: polkadot_cli::RunCmd::from_iter(relay_chain_args),
        }
    }
}

impl sc_cli::CliConfiguration<Self> for RelayChainCli {
    fn announce_block(&self) -> sc_cli::Result<bool> {
        self.base.base.announce_block()
    }

    fn base_path(&self) -> sc_cli::Result<Option<BasePath>> {
        Ok(self
            .shared_params()
            .base_path()
            .or_else(|| self.base_path.clone().map(Into::into)))
    }

    fn chain_id(&self, is_dev: bool) -> sc_cli::Result<String> {
        let chain_id = self.base.base.chain_id(is_dev)?;

        Ok(if chain_id.is_empty() {
            self.chain_id.clone().unwrap_or_default()
        } else {
            chain_id
        })
    }

    fn default_heap_pages(&self) -> sc_cli::Result<Option<u64>> {
        self.base.base.default_heap_pages()
    }

    fn disable_grandpa(&self) -> sc_cli::Result<bool> {
        self.base.base.disable_grandpa()
    }

    fn force_authoring(&self) -> sc_cli::Result<bool> {
        self.base.base.force_authoring()
    }

    fn import_params(&self) -> Option<&ImportParams> {
        self.base.base.import_params()
    }

    fn init<C>(&self) -> sc_cli::Result<()>
    where
        C: SubstrateCli,
    {
        unreachable!("PolkadotCli is never initialized; qed");
    }

    fn keystore_params(&self) -> Option<&KeystoreParams> {
        self.base.base.keystore_params()
    }

    fn max_runtime_instances(&self) -> sc_cli::Result<Option<usize>> {
        self.base.base.max_runtime_instances()
    }

    fn network_params(&self) -> Option<&NetworkParams> {
        self.base.base.network_params()
    }

    fn prometheus_config(
        &self,
        default_listen_port: u16,
    ) -> sc_cli::Result<Option<PrometheusConfig>> {
        self.base.base.prometheus_config(default_listen_port)
    }

    fn role(&self, is_dev: bool) -> sc_cli::Result<sc_service::Role> {
        self.base.base.role(is_dev)
    }

    fn rpc_cors(&self, is_dev: bool) -> sc_cli::Result<Option<Vec<String>>> {
        self.base.base.rpc_cors(is_dev)
    }

    fn rpc_http(&self, default_listen_port: u16) -> sc_cli::Result<Option<SocketAddr>> {
        self.base.base.rpc_http(default_listen_port)
    }

    fn rpc_ipc(&self) -> sc_cli::Result<Option<String>> {
        self.base.base.rpc_ipc()
    }

    fn rpc_methods(&self) -> sc_cli::Result<sc_service::config::RpcMethods> {
        self.base.base.rpc_methods()
    }

    fn rpc_ws(&self, default_listen_port: u16) -> sc_cli::Result<Option<SocketAddr>> {
        self.base.base.rpc_ws(default_listen_port)
    }

    fn rpc_ws_max_connections(&self) -> sc_cli::Result<Option<usize>> {
        self.base.base.rpc_ws_max_connections()
    }

    fn shared_params(&self) -> &SharedParams {
        self.base.base.shared_params()
    }

    fn state_cache_child_ratio(&self) -> sc_cli::Result<Option<usize>> {
        self.base.base.state_cache_child_ratio()
    }

    fn telemetry_external_transport(
        &self,
    ) -> sc_cli::Result<Option<sc_service::config::ExtTransport>> {
        self.base.base.telemetry_external_transport()
    }

    fn transaction_pool(&self) -> sc_cli::Result<sc_service::config::TransactionPoolOptions> {
        self.base.base.transaction_pool()
    }
}

impl sc_cli::DefaultConfigurationValues for RelayChainCli {
    fn p2p_listen_port() -> u16 {
        30334
    }

    fn prometheus_listen_port() -> u16 {
        9616
    }

    fn rpc_http_listen_port() -> u16 {
        9934
    }

    fn rpc_ws_listen_port() -> u16 {
        9945
    }
}

impl sc_cli::SubstrateCli for RelayChainCli {
    fn impl_name() -> String {
        "Parachain Collator Template".into()
    }

    fn impl_version() -> String {
        env!("SUBSTRATE_CLI_IMPL_VERSION").into()
    }

    fn description() -> String {
        "Parachain Collator Template\n\nThe command-line arguments provided first will be passed to \
     the parachain node, while the arguments provided after -- will be passed to the relaychain \
     node.\n\nrococo-collator [parachain-args] -- [relaychain-args]"
      .into()
    }

    fn author() -> String {
        env!("CARGO_PKG_AUTHORS").into()
    }

    fn support_url() -> String {
        "https://github.com/substrate-developer-hub/substrate-parachain-template/issues/new".into()
    }

    fn copyright_start_year() -> i32 {
        2017
    }

    fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
        <polkadot_cli::Cli as sc_cli::SubstrateCli>::from_iter(
            [RelayChainCli::executable_name().to_string()].iter(),
        )
        .load_spec(id)
    }

    fn native_runtime_version(chain_spec: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
        polkadot_cli::Cli::native_runtime_version(chain_spec)
    }
}
/// Command for exporting the genesis state of the parachain
#[derive(Debug, StructOpt)]
pub struct ExportGenesisStateCommand {
    /// Output file name or stdout if unspecified.
    #[structopt(parse(from_os_str))]
    pub output: Option<PathBuf>,

    /// Id of the parachain this state is for.
    #[structopt(long, default_value = "100")]
    pub parachain_id: u32,

    /// Write output in binary. Default is to write in hex.
    #[structopt(short, long)]
    pub raw: bool,

    /// The name of the chain for that the genesis state should be exported.
    #[structopt(long)]
    pub chain: Option<String>,
}

/// Command for exporting the genesis wasm file.
#[derive(Debug, StructOpt)]
pub struct ExportGenesisWasmCommand {
    /// Output file name or stdout if unspecified.
    #[structopt(parse(from_os_str))]
    pub output: Option<PathBuf>,

    /// Write output in binary. Default is to write in hex.
    #[structopt(short, long)]
    pub raw: bool,

    /// The name of the chain for that the genesis wasm file should be exported.
    #[structopt(long)]
    pub chain: Option<String>,
}

#[derive(Debug, structopt::StructOpt)]
pub struct RunCmd {
    #[structopt(flatten)]
    pub base: sc_cli::RunCmd,

    /// Id of the parachain this collator collates for.
    #[structopt(long)]
    pub parachain_id: Option<u32>,
}

impl std::ops::Deref for RunCmd {
    type Target = sc_cli::RunCmd;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
