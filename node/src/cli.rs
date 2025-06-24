// Copyright 2023-2025 Forecasting Technologies LTD.
// Copyright 2021-2022 Zeitgeist PM LLC.
// Copyright 2017-2020 Parity Technologies (UK) Ltd.
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

#[cfg(feature = "parachain")]
mod cli_parachain;

use super::service::{FullBackend, FullClient, IdentifyVariant};
#[cfg(feature = "with-battery-station-runtime")]
use battery_station_runtime::RuntimeApi as BatteryStationRuntimeApi;
use clap::Parser;
#[cfg(feature = "parachain")]
pub use cli_parachain::RelayChainCli;
use sc_cli::{ChainSpec, SubstrateCli};
use sc_client_api::{KeysIter, PairsIter};
use sp_consensus::BlockStatus;
use sp_runtime::{
    generic::SignedBlock,
    traits::{Block as BlockT, NumberFor},
    Justifications,
};
use sp_storage::{ChildInfo, StorageData, StorageKey};
use sp_trie::MerkleValue;
use std::{sync::Arc, time::Duration};
use zeitgeist_primitives::types::{Block, Header};
pub use zeitgeist_primitives::types::{BlockNumber, Hash};
#[cfg(feature = "with-zeitgeist-runtime")]
use zeitgeist_runtime::RuntimeApi as ZeitgeistRuntimeApi;

const COPYRIGHT_START_YEAR: i32 = 2021;
const IMPL_NAME: &str = "Zeitgeist Node";
const SUPPORT_URL: &str = "https://github.com/zeitgeistpm/zeitgeist/issues";

#[cfg(feature = "parachain")]
type RunCmd = cumulus_client_cli::RunCmd;
#[cfg(not(feature = "parachain"))]
type RunCmd = sc_cli::RunCmd;

pub fn load_spec(id: &str) -> Result<Box<dyn sc_service::ChainSpec>, String> {
    Ok(match id {
        #[cfg(feature = "with-battery-station-runtime")]
        "" | "dev" => Box::new(crate::chain_spec::dev_config()?),
        "battery_station" => Box::new(crate::chain_spec::DummyChainSpec::from_json_bytes(
            #[cfg(feature = "parachain")]
            &include_bytes!("../res/bs_parachain.json")[..],
            #[cfg(not(feature = "parachain"))]
            &include_bytes!("../res/bs.json")[..],
        )?),
        #[cfg(feature = "with-battery-station-runtime")]
        "battery_station_staging" => Box::new(crate::chain_spec::battery_station_staging_config()?),
        #[cfg(not(feature = "with-battery-station-runtime"))]
        "battery_station_staging" => panic!("{}", crate::BATTERY_STATION_RUNTIME_NOT_AVAILABLE),
        "zeitgeist" => Box::new(crate::chain_spec::DummyChainSpec::from_json_bytes(
            #[cfg(feature = "parachain")]
            &include_bytes!("../res/zeitgeist_parachain.json")[..],
            #[cfg(not(feature = "parachain"))]
            &include_bytes!("../res/zeitgeist.json")[..],
        )?),
        #[cfg(feature = "with-zeitgeist-runtime")]
        "zeitgeist_staging" => Box::new(crate::chain_spec::zeitgeist_staging_config()?),
        #[cfg(not(feature = "with-zeitgeist-runtime"))]
        "zeitgeist_staging" => panic!("{}", crate::ZEITGEIST_RUNTIME_NOT_AVAILABLE),
        path => {
            let spec = Box::new(crate::chain_spec::DummyChainSpec::from_json_file(
                std::path::PathBuf::from(path),
            )?) as Box<dyn ChainSpec>;

            match spec {
                spec if spec.is_zeitgeist() => {
                    #[cfg(feature = "with-zeitgeist-runtime")]
                    return Ok(Box::new(
                        crate::chain_spec::zeitgeist::ZeitgeistChainSpec::from_json_file(
                            std::path::PathBuf::from(path),
                        )?,
                    ));
                    #[cfg(not(feature = "with-zeitgeist-runtime"))]
                    panic!("{}", crate::ZEITGEIST_RUNTIME_NOT_AVAILABLE);
                }
                _ => {
                    #[cfg(feature = "with-battery-station-runtime")]
                    return Ok(
                        Box::new(
                            crate::chain_spec::battery_station::BatteryStationChainSpec::from_json_file(
                                std::path::PathBuf::from(path)
                            )?
                        )
                    );
                    #[cfg(not(feature = "with-battery-station-runtime"))]
                    panic!("{}", crate::BATTERY_STATION_RUNTIME_NOT_AVAILABLE);
                }
            }
        }
    })
}

#[derive(Debug, clap::Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum Subcommand {
    /// The custom benchmark subcommmand benchmarking runtime pallets.
    #[clap(subcommand)]
    Benchmark(frame_benchmarking_cli::BenchmarkCmd),

    /// Build a chain specification.
    BuildSpec(sc_cli::BuildSpecCmd),

    /// Db meta columns information.
    ChainInfo(sc_cli::ChainInfoCmd),

    /// Validate blocks.
    CheckBlock(sc_cli::CheckBlockCmd),

    /// Export blocks.
    ExportBlocks(sc_cli::ExportBlocksCmd),

    /// Export block header in hex format.
    #[cfg(feature = "parachain")]
    ExportHeader(sc_cli::CheckBlockCmd),

    /// Export the genesis state of the parachain.
    #[cfg(feature = "parachain")]
    #[command(alias = "export-genesis-state")]
    ExportGenesisHead(cumulus_client_cli::ExportGenesisHeadCommand),

    /// Export the genesis wasm of the parachain.
    #[cfg(feature = "parachain")]
    ExportGenesisWasm(cumulus_client_cli::ExportGenesisWasmCommand),

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

    /// Disable automatic hardware benchmarks.
    ///
    /// By default these benchmarks are automatically ran at startup and measure
    /// the CPU speed, the memory bandwidth and the disk speed.
    ///
    /// The results are then printed out in the logs, and also sent as part of
    /// telemetry, if telemetry is enabled.
    #[clap(long)]
    pub no_hardware_benchmarks: bool,

    /// Relaychain arguments
    #[cfg(feature = "parachain")]
    #[clap(raw = true)]
    pub relaychain_args: Vec<String>,

    /// Id of the parachain this collator collates for.
    #[cfg(feature = "parachain")]
    #[clap(long)]
    pub parachain_id: Option<u32>,

    #[allow(missing_docs)]
    #[clap(flatten)]
    pub storage_monitor: sc_storage_monitor::StorageMonitorParams,

    #[clap(long, default_value = "2000", value_parser=block_authoring_duration_parser)]
    pub block_authoring_duration: Duration,
}

fn block_authoring_duration_parser(s: &str) -> Result<Duration, String> {
    Ok(Duration::from_millis(clap_num::number_range(s, 250, 2_000)?))
}

#[cfg(feature = "parachain")]
impl Cli {
    #[allow(clippy::borrowed_box)]
    pub(crate) fn runtime_version(spec: &Box<dyn sc_service::ChainSpec>) -> sc_cli::RuntimeVersion {
        match spec {
            #[cfg(feature = "with-zeitgeist-runtime")]
            spec if spec.is_zeitgeist() => zeitgeist_runtime::VERSION,
            #[cfg(feature = "with-battery-station-runtime")]
            _ => battery_station_runtime::VERSION,
            #[cfg(not(feature = "with-battery-station-runtime"))]
            _ => panic!("{}", crate::BATTERY_STATION_RUNTIME_NOT_AVAILABLE),
        }
    }
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
        load_spec(id)
    }

    fn support_url() -> String {
        SUPPORT_URL.into()
    }
}

/// A client instance of Zeitgeist.
#[derive(Clone)]
pub enum Client {
    #[cfg(feature = "with-battery-station-runtime")]
    BatteryStation(Arc<FullClient<BatteryStationRuntimeApi>>),
    #[cfg(feature = "with-zeitgeist-runtime")]
    Zeitgeist(Arc<FullClient<ZeitgeistRuntimeApi>>),
}

#[cfg(feature = "with-battery-station-runtime")]
impl From<Arc<FullClient<BatteryStationRuntimeApi>>> for Client {
    fn from(client: Arc<FullClient<BatteryStationRuntimeApi>>) -> Self {
        Self::BatteryStation(client)
    }
}

#[cfg(feature = "with-zeitgeist-runtime")]
impl From<Arc<FullClient<ZeitgeistRuntimeApi>>> for Client {
    fn from(client: Arc<FullClient<ZeitgeistRuntimeApi>>) -> Self {
        Self::Zeitgeist(client)
    }
}

macro_rules! match_client {
    ($self:ident, $method:ident($($param:ident),*)) => {
        match $self {
            #[cfg(feature = "with-battery-station-runtime")]
            Self::BatteryStation(client) => client.$method($($param),*),
            #[cfg(feature = "with-zeitgeist-runtime")]
            Self::Zeitgeist(client) => client.$method($($param),*),
        }
    };
}

impl sc_client_api::UsageProvider<Block> for Client {
    fn usage_info(&self) -> sc_client_api::ClientInfo<Block> {
        match_client!(self, usage_info())
    }
}

impl sc_client_api::BlockBackend<Block> for Client {
    fn block_body(
        &self,
        hash: <Block as BlockT>::Hash,
    ) -> sp_blockchain::Result<Option<Vec<<Block as BlockT>::Extrinsic>>> {
        match_client!(self, block_body(hash))
    }

    fn block_indexed_body(
        &self,
        hash: <Block as BlockT>::Hash,
    ) -> sp_blockchain::Result<Option<Vec<Vec<u8>>>> {
        match_client!(self, block_indexed_body(hash))
    }

    fn block(
        &self,
        hash: <Block as BlockT>::Hash,
    ) -> sp_blockchain::Result<Option<SignedBlock<Block>>> {
        match_client!(self, block(hash))
    }

    fn block_status(&self, hash: <Block as BlockT>::Hash) -> sp_blockchain::Result<BlockStatus> {
        match_client!(self, block_status(hash))
    }

    fn justifications(
        &self,
        hash: <Block as BlockT>::Hash,
    ) -> sp_blockchain::Result<Option<Justifications>> {
        match_client!(self, justifications(hash))
    }

    fn block_hash(
        &self,
        number: NumberFor<Block>,
    ) -> sp_blockchain::Result<Option<<Block as BlockT>::Hash>> {
        match_client!(self, block_hash(number))
    }

    fn indexed_transaction(
        &self,
        hash: <Block as BlockT>::Hash,
    ) -> sp_blockchain::Result<Option<Vec<u8>>> {
        match_client!(self, indexed_transaction(hash))
    }

    fn has_indexed_transaction(
        &self,
        hash: <Block as BlockT>::Hash,
    ) -> sp_blockchain::Result<bool> {
        match_client!(self, has_indexed_transaction(hash))
    }

    fn requires_full_sync(&self) -> bool {
        match_client!(self, requires_full_sync())
    }
}

impl sc_client_api::StorageProvider<Block, FullBackend> for Client {
    fn storage(
        &self,
        hash: <Block as BlockT>::Hash,
        key: &StorageKey,
    ) -> sp_blockchain::Result<Option<StorageData>> {
        match_client!(self, storage(hash, key))
    }

    fn storage_hash(
        &self,
        hash: <Block as BlockT>::Hash,
        key: &StorageKey,
    ) -> sp_blockchain::Result<Option<<Block as BlockT>::Hash>> {
        match_client!(self, storage_hash(hash, key))
    }

    fn storage_keys(
        &self,
        hash: <Block as BlockT>::Hash,
        prefix: Option<&StorageKey>,
        start_key: Option<&StorageKey>,
    ) -> sp_blockchain::Result<KeysIter<<FullBackend as sc_client_api::Backend<Block>>::State, Block>>
    {
        match_client!(self, storage_keys(hash, prefix, start_key))
    }

    fn storage_pairs(
        &self,
        hash: <Block as BlockT>::Hash,
        key_prefix: Option<&StorageKey>,
        start_key: Option<&StorageKey>,
    ) -> sp_blockchain::Result<
        PairsIter<<FullBackend as sc_client_api::Backend<Block>>::State, Block>,
    > {
        match_client!(self, storage_pairs(hash, key_prefix, start_key))
    }

    fn child_storage(
        &self,
        hash: <Block as BlockT>::Hash,
        child_info: &ChildInfo,
        key: &StorageKey,
    ) -> sp_blockchain::Result<Option<StorageData>> {
        match_client!(self, child_storage(hash, child_info, key))
    }

    fn child_storage_keys(
        &self,
        hash: <Block as BlockT>::Hash,
        child_info: ChildInfo,
        prefix: Option<&StorageKey>,
        start_key: Option<&StorageKey>,
    ) -> sp_blockchain::Result<KeysIter<<FullBackend as sc_client_api::Backend<Block>>::State, Block>>
    {
        match_client!(self, child_storage_keys(hash, child_info, prefix, start_key))
    }

    fn child_storage_hash(
        &self,
        hash: <Block as BlockT>::Hash,
        child_info: &ChildInfo,
        key: &StorageKey,
    ) -> sp_blockchain::Result<Option<<Block as BlockT>::Hash>> {
        match_client!(self, child_storage_hash(hash, child_info, key))
    }

    fn closest_merkle_value(
        &self,
        hash: <Block as BlockT>::Hash,
        key: &StorageKey,
    ) -> sp_blockchain::Result<Option<MerkleValue<<Block as BlockT>::Hash>>> {
        match_client!(self, closest_merkle_value(hash, key))
    }

    fn child_closest_merkle_value(
        &self,
        hash: <Block as BlockT>::Hash,
        child_info: &ChildInfo,
        key: &StorageKey,
    ) -> sp_blockchain::Result<Option<MerkleValue<<Block as BlockT>::Hash>>> {
        match_client!(self, child_closest_merkle_value(hash, child_info, key))
    }
}

impl sp_blockchain::HeaderBackend<Block> for Client {
    fn header(&self, hash: Hash) -> sp_blockchain::Result<Option<Header>> {
        match_client!(self, header(hash))
    }

    fn info(&self) -> sp_blockchain::Info<Block> {
        match_client!(self, info())
    }

    fn status(&self, hash: Hash) -> sp_blockchain::Result<sp_blockchain::BlockStatus> {
        match_client!(self, status(hash))
    }

    fn number(&self, hash: Hash) -> sp_blockchain::Result<Option<BlockNumber>> {
        match_client!(self, number(hash))
    }

    fn hash(&self, number: NumberFor<Block>) -> sp_blockchain::Result<Option<Hash>> {
        match_client!(self, hash(number))
    }
}
