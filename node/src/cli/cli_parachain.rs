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

use clap::Parser;
use sc_cli::{
    self, ChainSpec, ImportParams, KeystoreParams, NetworkParams, RuntimeVersion, SharedParams,
    SubstrateCli,
};
use sc_service::config::{BasePath, PrometheusConfig};
use std::{net::SocketAddr, path::PathBuf};

const BATTERY_STATION_RELAY_ID: &str = "battery_station_relay_v3";

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
    /// Parse the relay chain CLI parameters using the parachain `Configuration`.
    pub fn new<'a>(
        para_config: &sc_service::Configuration,
        relay_chain_args: impl Iterator<Item = &'a String>,
    ) -> Self {
        let extension = crate::chain_spec::Extensions::try_get(&*para_config.chain_spec);
        let chain_id = extension.map(|e| e.relay_chain.clone());
        let base_path = para_config
            .base_path
            .as_ref()
            .map(|x| x.path().join(chain_id.clone().unwrap_or_else(|| "polkadot".into())));

        Self { base_path, chain_id, base: clap::Parser::parse_from(relay_chain_args) }
    }
}

impl sc_cli::CliConfiguration<Self> for RelayChainCli {
    fn announce_block(&self) -> sc_cli::Result<bool> {
        self.base.base.announce_block()
    }

    fn base_path(&self) -> sc_cli::Result<Option<BasePath>> {
        Ok(self.shared_params().base_path()?.or_else(|| self.base_path.clone().map(Into::into)))
    }

    fn chain_id(&self, is_dev: bool) -> sc_cli::Result<String> {
        let chain_id = self.base.base.chain_id(is_dev)?;

        Ok(if chain_id.is_empty() { self.chain_id.clone().unwrap_or_default() } else { chain_id })
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

    fn init<F>(
        &self,
        _support_url: &String,
        _impl_version: &String,
        _logger_hook: F,
        _config: &sc_service::Configuration,
    ) -> sc_cli::Result<()>
    where
        F: FnOnce(&mut sc_cli::LoggerBuilder, &sc_service::Configuration),
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
        chain_spec: &Box<dyn ChainSpec>,
    ) -> sc_cli::Result<Option<PrometheusConfig>> {
        self.base.base.prometheus_config(default_listen_port, chain_spec)
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

    fn transaction_pool(
        &self,
        is_dev: bool,
    ) -> sc_cli::Result<sc_service::config::TransactionPoolOptions> {
        self.base.base.transaction_pool(is_dev)
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
    fn author() -> String {
        env!("CARGO_PKG_AUTHORS").into()
    }

    fn copyright_start_year() -> i32 {
        crate::cli::COPYRIGHT_START_YEAR
    }

    fn description() -> String {
        env!("CARGO_PKG_DESCRIPTION").into()
    }

    fn impl_name() -> String {
        crate::cli::IMPL_NAME.into()
    }

    fn impl_version() -> String {
        env!("SUBSTRATE_CLI_IMPL_VERSION").into()
    }

    fn load_spec(&self, id: &str) -> Result<Box<dyn sc_service::ChainSpec>, String> {
        if id == BATTERY_STATION_RELAY_ID {
            Ok(Box::new(polkadot_service::RococoChainSpec::from_json_bytes(
                &include_bytes!("../../res/battery_station_relay.json")[..],
            )?))
        } else {
            <polkadot_cli::Cli as SubstrateCli>::from_iter(
                [RelayChainCli::executable_name()].iter(),
            )
            .load_spec(id)
        }
    }

    fn native_runtime_version(chain_spec: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
        polkadot_cli::Cli::native_runtime_version(chain_spec)
    }

    fn support_url() -> String {
        crate::cli::SUPPORT_URL.into()
    }
}

#[derive(Debug, Parser)]
pub struct RunCmd {
    #[clap(flatten)]
    pub base: sc_cli::RunCmd,

    /// Disable automatic hardware benchmarks.
    ///
    /// By default these benchmarks are automatically ran at startup and measure
    /// the CPU speed, the memory bandwidth and the disk speed.
    ///
    /// The results are then printed out in the logs, and also sent as part of
    /// telemetry, if telemetry is enabled.
    #[clap(long)]
    pub no_hardware_benchmarks: bool,

    /// Id of the parachain this collator collates for.
    #[clap(long)]
    pub parachain_id: u32,
}

impl core::ops::Deref for RunCmd {
    type Target = sc_cli::RunCmd;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
