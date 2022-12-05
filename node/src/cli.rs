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

use super::service::{
    AdditionalRuntimeApiCollection, FullBackend, FullClient, IdentifyVariant, RuntimeApiCollection,
};
use clap::Parser;
#[cfg(feature = "parachain")]
pub use cli_parachain::RelayChainCli;
use sc_cli::{ChainSpec, RuntimeVersion, SubstrateCli};
use sc_client_api::{Backend as BackendT, BlockchainEvents, KeyIterator};
use sp_api::{CallApiAt, NumberFor, ProvideRuntimeApi};
use sp_blockchain::HeaderBackend;
use sp_consensus::BlockStatus;
use sp_runtime::{
    generic::{BlockId, SignedBlock},
    traits::{BlakeTwo256, Block as BlockT},
    Justifications,
};
use sp_storage::{ChildInfo, StorageData, StorageKey};
use std::sync::Arc;
pub use zeitgeist_primitives::types::{AccountId, Balance, BlockNumber, Hash, Index};
use zeitgeist_primitives::types::{Block, Header};
#[cfg(feature = "with-battery-station-runtime")]
use {
    super::service::BatteryStationExecutor,
    battery_station_runtime::RuntimeApi as BatteryStationRuntimeApi,
};
#[cfg(feature = "with-zeitgeist-runtime")]
use {super::service::ZeitgeistExecutor, zeitgeist_runtime::RuntimeApi as ZeitgeistRuntimeApi};

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
    ExportGenesisState(cumulus_client_cli::ExportGenesisStateCommand),

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

    fn native_runtime_version(spec: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
        match spec {
            spec if spec.is_zeitgeist() => {
                #[cfg(feature = "with-zeitgeist-runtime")]
                return &zeitgeist_runtime::VERSION;
                #[cfg(not(feature = "with-zeitgeist-runtime"))]
                panic!("{}", crate::ZEITGEIST_RUNTIME_NOT_AVAILABLE);
            }
            _spec => {
                #[cfg(feature = "with-battery-station-runtime")]
                return &battery_station_runtime::VERSION;
                #[cfg(not(feature = "with-battery-station-runtime"))]
                panic!("{}", crate::BATTERY_STATION_RUNTIME_NOT_AVAILABLE);
            }
        }
    }

    fn support_url() -> String {
        SUPPORT_URL.into()
    }
}

/// Config that abstracts over all available client implementations.
///
/// For a concrete type there exists [`Client`].
pub trait AbstractClient<Block, Backend>:
    BlockchainEvents<Block>
    + Sized
    + Send
    + Sync
    + ProvideRuntimeApi<Block>
    + HeaderBackend<Block>
    + CallApiAt<Block, StateBackend = Backend::State>
where
    Block: BlockT,
    Backend: BackendT<Block>,
    Backend::State: sp_api::StateBackend<BlakeTwo256>,
    Self::Api: RuntimeApiCollection<StateBackend = Backend::State>
        + AdditionalRuntimeApiCollection<StateBackend = Backend::State>,
{
}

impl<Block, Backend, Client> AbstractClient<Block, Backend> for Client
where
    Block: BlockT,
    Backend: BackendT<Block>,
    Backend::State: sp_api::StateBackend<BlakeTwo256>,
    Client: BlockchainEvents<Block>
        + ProvideRuntimeApi<Block>
        + HeaderBackend<Block>
        + Sized
        + Send
        + Sync
        + CallApiAt<Block, StateBackend = Backend::State>,
    Client::Api: RuntimeApiCollection<StateBackend = Backend::State>
        + AdditionalRuntimeApiCollection<StateBackend = Backend::State>,
{
}

/// Execute something with the client instance.
///
/// As there exist multiple chains inside Zeitgeist, like Zeitgeist itself,
/// Battery Station etc., there can exist different kinds of client types. As these
/// client types differ in the generics that are being used, we can not easily
/// return them from a function. For returning them from a function there exists
/// [`Client`]. However, the problem on how to use this client instance still
/// exists. This trait "solves" it in a dirty way. It requires a type to
/// implement this trait and than the [`execute_with_client`](ExecuteWithClient::execute_with_client)
/// function can be called with any possible client
/// instance.
///
/// In a perfect world, we could make a closure work in this way.
pub trait ExecuteWithClient {
    /// The return type when calling this instance.
    type Output;

    /// Execute whatever should be executed with the given client instance.
    fn execute_with_client<Client, Api, Backend>(self, client: Arc<Client>) -> Self::Output
    where
        <Api as sp_api::ApiExt<Block>>::StateBackend: sp_api::StateBackend<BlakeTwo256>,
        Backend: sc_client_api::Backend<Block>,
        Backend::State: sp_api::StateBackend<BlakeTwo256>,
        Api: RuntimeApiCollection<StateBackend = Backend::State>
            + AdditionalRuntimeApiCollection<StateBackend = Backend::State>,
        Client: AbstractClient<Block, Backend, Api = Api> + 'static;
}

/// A handle to a Zeitgeist client instance.
///
/// The Zeitgeist service supports multiple different runtimes (Zeitgeist, Battery
/// Station, etc.). As each runtime has a specialized client, we need to hide them
/// behind a trait. This is this trait.
///
/// When wanting to work with the inner client, you need to use `execute_with`.
pub trait ClientHandle {
    /// Execute the given something with the client.
    fn execute_with<T: ExecuteWithClient>(&self, t: T) -> T::Output;
}

/// A client instance of Zeitgeist.
#[derive(Clone)]
pub enum Client {
    #[cfg(feature = "with-battery-station-runtime")]
    BatteryStation(Arc<FullClient<BatteryStationRuntimeApi, BatteryStationExecutor>>),
    #[cfg(feature = "with-zeitgeist-runtime")]
    Zeitgeist(Arc<FullClient<ZeitgeistRuntimeApi, ZeitgeistExecutor>>),
}

#[cfg(feature = "with-battery-station-runtime")]
impl From<Arc<FullClient<BatteryStationRuntimeApi, BatteryStationExecutor>>> for Client {
    fn from(client: Arc<FullClient<BatteryStationRuntimeApi, BatteryStationExecutor>>) -> Self {
        Self::BatteryStation(client)
    }
}

#[cfg(feature = "with-zeitgeist-runtime")]
impl From<Arc<FullClient<ZeitgeistRuntimeApi, ZeitgeistExecutor>>> for Client {
    fn from(client: Arc<FullClient<ZeitgeistRuntimeApi, ZeitgeistExecutor>>) -> Self {
        Self::Zeitgeist(client)
    }
}

impl ClientHandle for Client {
    fn execute_with<T: ExecuteWithClient>(&self, t: T) -> T::Output {
        match self {
            #[cfg(feature = "with-battery-station-runtime")]
            Self::BatteryStation(client) => {
                T::execute_with_client::<_, _, FullBackend>(t, client.clone())
            }
            #[cfg(feature = "with-zeitgeist-runtime")]
            Self::Zeitgeist(client) => {
                T::execute_with_client::<_, _, FullBackend>(t, client.clone())
            }
        }
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
        id: &BlockId<Block>,
    ) -> sp_blockchain::Result<Option<Vec<<Block as BlockT>::Extrinsic>>> {
        match_client!(self, block_body(id))
    }

    fn block_indexed_body(
        &self,
        id: &BlockId<Block>,
    ) -> sp_blockchain::Result<Option<Vec<Vec<u8>>>> {
        match_client!(self, block_indexed_body(id))
    }

    fn block(&self, id: &BlockId<Block>) -> sp_blockchain::Result<Option<SignedBlock<Block>>> {
        match_client!(self, block(id))
    }

    fn block_status(&self, id: &BlockId<Block>) -> sp_blockchain::Result<BlockStatus> {
        match_client!(self, block_status(id))
    }

    fn justifications(&self, id: &BlockId<Block>) -> sp_blockchain::Result<Option<Justifications>> {
        match_client!(self, justifications(id))
    }

    fn block_hash(
        &self,
        number: NumberFor<Block>,
    ) -> sp_blockchain::Result<Option<<Block as BlockT>::Hash>> {
        match_client!(self, block_hash(number))
    }

    fn indexed_transaction(
        &self,
        hash: &<Block as BlockT>::Hash,
    ) -> sp_blockchain::Result<Option<Vec<u8>>> {
        match_client!(self, indexed_transaction(hash))
    }

    fn has_indexed_transaction(
        &self,
        hash: &<Block as BlockT>::Hash,
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
        id: &BlockId<Block>,
        key: &StorageKey,
    ) -> sp_blockchain::Result<Option<StorageData>> {
        match_client!(self, storage(id, key))
    }

    fn storage_keys(
        &self,
        id: &BlockId<Block>,
        key_prefix: &StorageKey,
    ) -> sp_blockchain::Result<Vec<StorageKey>> {
        match_client!(self, storage_keys(id, key_prefix))
    }

    fn storage_hash(
        &self,
        id: &BlockId<Block>,
        key: &StorageKey,
    ) -> sp_blockchain::Result<Option<<Block as BlockT>::Hash>> {
        match_client!(self, storage_hash(id, key))
    }

    fn storage_pairs(
        &self,
        id: &BlockId<Block>,
        key_prefix: &StorageKey,
    ) -> sp_blockchain::Result<Vec<(StorageKey, StorageData)>> {
        match_client!(self, storage_pairs(id, key_prefix))
    }

    fn storage_keys_iter<'a>(
        &self,
        id: &BlockId<Block>,
        prefix: Option<&'a StorageKey>,
        start_key: Option<&StorageKey>,
    ) -> sp_blockchain::Result<
        KeyIterator<'a, <FullBackend as sc_client_api::Backend<Block>>::State, Block>,
    > {
        match_client!(self, storage_keys_iter(id, prefix, start_key))
    }

    fn child_storage(
        &self,
        id: &BlockId<Block>,
        child_info: &ChildInfo,
        key: &StorageKey,
    ) -> sp_blockchain::Result<Option<StorageData>> {
        match_client!(self, child_storage(id, child_info, key))
    }

    fn child_storage_keys(
        &self,
        id: &BlockId<Block>,
        child_info: &ChildInfo,
        key_prefix: &StorageKey,
    ) -> sp_blockchain::Result<Vec<StorageKey>> {
        match_client!(self, child_storage_keys(id, child_info, key_prefix))
    }

    fn child_storage_keys_iter<'a>(
        &self,
        id: &BlockId<Block>,
        child_info: ChildInfo,
        prefix: Option<&'a StorageKey>,
        start_key: Option<&StorageKey>,
    ) -> sp_blockchain::Result<
        KeyIterator<'a, <FullBackend as sc_client_api::Backend<Block>>::State, Block>,
    > {
        match_client!(self, child_storage_keys_iter(id, child_info, prefix, start_key))
    }

    fn child_storage_hash(
        &self,
        id: &BlockId<Block>,
        child_info: &ChildInfo,
        key: &StorageKey,
    ) -> sp_blockchain::Result<Option<<Block as BlockT>::Hash>> {
        match_client!(self, child_storage_hash(id, child_info, key))
    }
}

impl sp_blockchain::HeaderBackend<Block> for Client {
    fn header(&self, id: BlockId<Block>) -> sp_blockchain::Result<Option<Header>> {
        let id = &id;
        match_client!(self, header(id))
    }

    fn info(&self) -> sp_blockchain::Info<Block> {
        match_client!(self, info())
    }

    fn status(&self, id: BlockId<Block>) -> sp_blockchain::Result<sp_blockchain::BlockStatus> {
        match_client!(self, status(id))
    }

    fn number(&self, hash: Hash) -> sp_blockchain::Result<Option<BlockNumber>> {
        match_client!(self, number(hash))
    }

    fn hash(&self, number: BlockNumber) -> sp_blockchain::Result<Option<Hash>> {
        match_client!(self, hash(number))
    }
}
