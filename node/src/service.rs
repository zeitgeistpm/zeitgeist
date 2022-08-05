#[cfg(feature = "parachain")]
mod service_parachain;
#[cfg(not(feature = "parachain"))]
mod service_standalone;

use sp_runtime::traits::BlakeTwo256;
use zeitgeist_primitives::types::{AccountId, Balance, Index, MarketId, PoolId};
use zeitgeist_primitives::types::Block;

#[cfg(feature = "parachain")]
pub use service_parachain::{new_full, new_partial, FullClient, FullBackend, ParachainPartialComponents};
#[cfg(not(feature = "parachain"))]
pub use service_standalone::{new_full, new_partial, FullClient, FullBackend};
use std::sync::Arc;
use sc_service::{error::Error as ServiceError, ChainSpec, Configuration, TaskManager, PartialComponents};
use sp_trie::PrefixedMemoryDB;
use sp_api::ConstructRuntimeApi;
use sc_executor::{NativeExecutionDispatch};
use super::cli::Client;

#[cfg(feature = "with-battery-station-runtime")]
pub struct BatteryStationExecutor;

#[cfg(feature = "with-battery-station-runtime")]
impl sc_executor::NativeExecutionDispatch for BatteryStationExecutor {
    #[cfg(feature = "runtime-benchmarks")]
    type ExtendHostFunctions = frame_benchmarking::benchmarking::HostFunctions;
    #[cfg(not(feature = "runtime-benchmarks"))]
    type ExtendHostFunctions = ();

    fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
        battery_station_runtime::api::dispatch(method, data)
    }

    fn native_version() -> sc_executor::NativeVersion {
        battery_station_runtime::native_version()
    }
}

#[cfg(feature = "with-zeitgeist-runtime")]
pub struct ZeitgeistExecutor;

#[cfg(feature = "with-zeitgeist-runtime")]
impl sc_executor::NativeExecutionDispatch for ZeitgeistExecutor {
    #[cfg(feature = "runtime-benchmarks")]
    type ExtendHostFunctions = frame_benchmarking::benchmarking::HostFunctions;
    #[cfg(not(feature = "runtime-benchmarks"))]
    type ExtendHostFunctions = ();

    fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
        zeitgeist_runtime::api::dispatch(method, data)
    }

    fn native_version() -> sc_executor::NativeVersion {
        zeitgeist_runtime::native_version()
    }
}

/// Can be called for a `Configuration` to check if it is a configuration for
/// the `Zeitgeist` network.
pub trait IdentifyVariant {
	/// Returns `true` if this is a configuration for the `Battery Station` network.
	fn is_battery_station(&self) -> bool;

	/// Returns `true` if this is a configuration for the `Zeitgeist` network.
	fn is_zeitgeist(&self) -> bool;
}

impl IdentifyVariant for Box<dyn ChainSpec> {
	fn is_battery_station(&self) -> bool {
		self.id().starts_with("battery_station")
	}

	fn is_zeitgeist(&self) -> bool {
		self.id().starts_with("zeitgeist")
	}
}

/// A set of common runtime APIs between standalone an parachain runtimes.
pub trait RuntimeApiCollection:
    sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
    + sp_api::ApiExt<Block>
    + sp_block_builder::BlockBuilder<Block>
    + substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>
    + pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance>
    + sp_api::Metadata<Block>
    + sp_offchain::OffchainWorkerApi<Block>
    + sp_session::SessionKeys<Block>
    + zrml_swaps_rpc::SwapsRuntimeApi<Block, PoolId, AccountId, Balance, MarketId>
where
    <Self as sp_api::ApiExt<Block>>::StateBackend: sp_api::StateBackend<BlakeTwo256>,
{
}

impl<Api> RuntimeApiCollection for Api
where
    Api: sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
        + sp_api::ApiExt<Block>
        + sp_block_builder::BlockBuilder<Block>
        + substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>
        + pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance>
        + sp_api::Metadata<Block>
        + sp_offchain::OffchainWorkerApi<Block>
        + sp_session::SessionKeys<Block>
        + zrml_swaps_rpc::SwapsRuntimeApi<Block, PoolId, AccountId, Balance, MarketId>,
    <Self as sp_api::ApiExt<Block>>::StateBackend: sp_api::StateBackend<BlakeTwo256>,
{
}

cfg_if::cfg_if! {
    if #[cfg(feature = "parachain")] {
        /// Additional APIs for parachain runtimes
        pub trait AdditionalRuntimeApiCollection:
            sp_api::ApiExt<Block>
            + nimbus_primitives::AuthorFilterAPI<Block, nimbus_primitives::NimbusId>
            + nimbus_primitives::NimbusApi<Block>
            + cumulus_primitives_core::CollectCollationInfo<Block>
        where
            <Self as sp_api::ApiExt<Block>>::StateBackend: sp_api::StateBackend<BlakeTwo256>,
        {
        }

        impl<Api> AdditionalRuntimeApiCollection for Api
        where
            Api: sp_api::ApiExt<Block>
                + nimbus_primitives::AuthorFilterAPI<Block, nimbus_primitives::NimbusId>
                + nimbus_primitives::NimbusApi<Block>
                + cumulus_primitives_core::CollectCollationInfo<Block>,
            <Self as sp_api::ApiExt<Block>>::StateBackend: sp_api::StateBackend<BlakeTwo256>,
        {
        }
    } else {
        /// Additional APIs for standalone runtimes
        pub trait AdditionalRuntimeApiCollection:
            sp_api::ApiExt<Block>
            + sp_finality_grandpa::GrandpaApi<Block>
            + sp_consensus_aura::AuraApi<Block, sp_consensus_aura::sr25519::AuthorityId>
        where
            <Self as sp_api::ApiExt<Block>>::StateBackend: sp_api::StateBackend<BlakeTwo256>,
        {
        }

        impl<Api> AdditionalRuntimeApiCollection for Api
        where
            Api: sp_api::ApiExt<Block>
                + sp_finality_grandpa::GrandpaApi<Block>
                + sp_consensus_aura::AuraApi<Block, sp_consensus_aura::sr25519::AuthorityId>,
            <Self as sp_api::ApiExt<Block>>::StateBackend: sp_api::StateBackend<BlakeTwo256>,
        {
        }
    }
}

/// Builds a new object suitable for chain operations.
#[allow(clippy::type_complexity)]
pub fn new_chain_ops(
	config: &mut Configuration,
) -> Result<
	(
		Arc<Client>,
		Arc<FullBackend>,
		sc_consensus::BasicQueue<Block, PrefixedMemoryDB<BlakeTwo256>>,
		TaskManager,
	),
	ServiceError,
> {
	match &config.chain_spec {
		#[cfg(feature = "with-zeitgeist-runtime")]
		spec if spec.is_zeitgeist() => {
			new_chain_ops_inner::<zeitgeist_runtime::RuntimeApi, ZeitgeistExecutor>(config)
		}
		#[cfg(feature = "with-battery-station-runtime")]
		_ => {
			new_chain_ops_inner::<battery_station_runtime::RuntimeApi, BatteryStationExecutor>(config)
		}
        #[cfg(not(feature = "with-battery-station-runtime"))]
		_ => panic!("Invalid chain spec"),
	}
}

#[allow(clippy::type_complexity)]
fn new_chain_ops_inner<RuntimeApi, Executor>(
	mut config: &mut Configuration,
) -> Result<
	(
		Arc<Client>,
		Arc<FullBackend>,
		sc_consensus::BasicQueue<Block, PrefixedMemoryDB<BlakeTwo256>>,
		TaskManager,
	),
	ServiceError,
>
where
	Client: From<Arc<FullClient<RuntimeApi, Executor>>>,
	RuntimeApi:
		ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>> + Send + Sync + 'static,
	RuntimeApi::RuntimeApi:
		RuntimeApiCollection<StateBackend = sc_client_api::StateBackendFor<FullBackend, Block>>
            + AdditionalRuntimeApiCollection<StateBackend = sc_client_api::StateBackendFor<FullBackend, Block>>,
	Executor: NativeExecutionDispatch + 'static,
{
	config.keystore = sc_service::config::KeystoreConfig::InMemory;
	let PartialComponents {
		client,
		backend,
		import_queue,
		task_manager,
		..
	} = new_partial::<RuntimeApi, Executor>(config)?;
	Ok((
		Arc::new(Client::from(client)),
		backend,
		import_queue,
		task_manager,
	))
}