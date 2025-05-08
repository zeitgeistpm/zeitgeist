// Copyright 2022-2024 Forecasting Technologies LTD.
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

use crate::{
    service::{AdditionalRuntimeApiCollection, RuntimeApiCollection},
    POLKADOT_BLOCK_DURATION,
};
use cumulus_client_cli::CollatorOptions;
use cumulus_client_collator::service::CollatorService;
use cumulus_client_consensus_common::ParachainBlockImport as TParachainBlockImport;
use cumulus_client_consensus_proposer::Proposer;
use cumulus_client_network::RequireSecondedInBlockAnnounce;
use cumulus_client_service::{
    build_relay_chain_interface, prepare_node_config, start_relay_chain_tasks, DARecoveryProfile,
    StartRelayChainTasksParams,
};
use cumulus_primitives_core::ParaId;
use cumulus_relay_chain_interface::{OverseerHandle, RelayChainInterface};
use frame_benchmarking_cli::SUBSTRATE_REFERENCE_HARDWARE;
use futures::FutureExt;
use polkadot_service::CollatorPair;
use sc_client_api::Backend;
use sc_consensus::ImportQueue;
#[allow(deprecated)]
use sc_executor::{
    HeapAllocStrategy, NativeElseWasmExecutor, WasmExecutor, DEFAULT_HEAP_ALLOC_STRATEGY,
};
use sc_network::{config::FullNetworkConfiguration, NetworkBlock};
use sc_service::{
    error::{Error as ServiceError, Result as ServiceResult},
    Configuration, PartialComponents, TFullBackend, TFullClient, TaskManager,
};
use sc_telemetry::{Telemetry, TelemetryHandle, TelemetryWorker, TelemetryWorkerHandle};
use sc_transaction_pool_api::OffchainTransactionPoolFactory;
use sp_api::ConstructRuntimeApi;
use sp_keystore::KeystorePtr;
use sp_runtime::traits::Block as BlockT;
use std::sync::Arc;
use substrate_prometheus_endpoint::Registry;
use zeitgeist_primitives::types::{Block, Hash};

#[cfg(feature = "runtime-benchmarks")]
pub type HostFunctions =
    (frame_benchmarking::benchmarking::HostFunctions, sp_io::SubstrateHostFunctions);
#[cfg(not(feature = "runtime-benchmarks"))]
pub type HostFunctions = (sp_io::SubstrateHostFunctions,);

#[cfg(feature = "with-battery-station-runtime")]
pub struct BatteryStationExecutor;

#[cfg(feature = "with-battery-station-runtime")]
impl sc_executor::NativeExecutionDispatch for BatteryStationExecutor {
    type ExtendHostFunctions = HostFunctions;

    fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
        battery_station_runtime::api::dispatch(method, data)
    }

    fn native_version() -> sc_executor::NativeVersion {
        battery_station_runtime::native_version()
    }
}

#[cfg(not(feature = "with-battery-station-runtime"))]
pub struct ZeitgeistExecutor;

#[cfg(not(feature = "with-battery-station-runtime"))]
impl sc_executor::NativeExecutionDispatch for ZeitgeistExecutor {
    type ExtendHostFunctions = HostFunctions;

    fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
        zeitgeist_runtime::api::dispatch(method, data)
    }

    fn native_version() -> sc_executor::NativeVersion {
        zeitgeist_runtime::native_version()
    }
}

#[cfg(feature = "with-battery-station-runtime")]
type Executor = BatteryStationExecutor;
#[cfg(not(feature = "with-battery-station-runtime"))]
type Executor = ZeitgeistExecutor;

// TODO(#1430): As soon as the WasmExecutor is used here, enable storage weight reclaim: https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/guides/enable_pov_reclaim/index.html
#[allow(deprecated)]
pub type ParachainExecutor = NativeElseWasmExecutor<Executor>;

pub type FullClient<RuntimeApi> = TFullClient<Block, RuntimeApi, ParachainExecutor>;

pub type FullBackend = TFullBackend<Block>;

pub type ParachainBlockImport<RuntimeApi> =
    TParachainBlockImport<Block, Arc<FullClient<RuntimeApi>>, FullBackend>;

/// Assembly of PartialComponents (enough to run chain ops subcommands)
pub type ParachainPartialComponents<RuntimeApi> = PartialComponents<
    FullClient<RuntimeApi>,
    FullBackend,
    (),
    sc_consensus::DefaultImportQueue<Block>,
    sc_transaction_pool::FullPool<Block, FullClient<RuntimeApi>>,
    (ParachainBlockImport<RuntimeApi>, Option<Telemetry>, Option<TelemetryWorkerHandle>),
>;

/// Start a parachain node.
/// called `start_parachain_node` in moonkit node template
#[allow(deprecated)]
pub async fn new_full<RuntimeApi>(
    parachain_config: Configuration,
    para_id: ParaId,
    polkadot_config: Configuration,
    hwbench: Option<sc_sysinfo::HwBench>,
    collator_options: CollatorOptions,
) -> ServiceResult<(TaskManager, Arc<FullClient<RuntimeApi>>)>
where
    RuntimeApi: ConstructRuntimeApi<Block, FullClient<RuntimeApi>> + Send + Sync + 'static,
    RuntimeApi::RuntimeApi: RuntimeApiCollection + AdditionalRuntimeApiCollection,
{
    do_new_full::<RuntimeApi, sc_network::NetworkWorker<_, _>>(
        parachain_config,
        polkadot_config,
        para_id,
        hwbench,
        collator_options,
    )
    .await
}

/// Builds the PartialComponents for a parachain or development service
///
/// Use this function if you don't actually need the full service, but just the partial in order to
/// be able to perform chain operations.
pub fn new_partial<RuntimeApi>(
    config: &Configuration,
) -> Result<ParachainPartialComponents<RuntimeApi>, ServiceError>
where
    RuntimeApi: ConstructRuntimeApi<Block, FullClient<RuntimeApi>> + Send + Sync + 'static,
    RuntimeApi::RuntimeApi: RuntimeApiCollection + AdditionalRuntimeApiCollection,
{
    let telemetry = config
        .telemetry_endpoints
        .clone()
        .filter(|x| !x.is_empty())
        .map(|endpoints| -> Result<_, sc_telemetry::Error> {
            let worker = TelemetryWorker::new(16)?;
            let telemetry = worker.handle().new_telemetry(endpoints);
            Ok((worker, telemetry))
        })
        .transpose()?;

    let heap_pages = config
        .executor
        .default_heap_pages
        .map_or(DEFAULT_HEAP_ALLOC_STRATEGY, |h| HeapAllocStrategy::Static { extra_pages: h as _ });

    let wasm = WasmExecutor::builder()
        .with_execution_method(config.executor.wasm_method)
        .with_onchain_heap_alloc_strategy(heap_pages)
        .with_offchain_heap_alloc_strategy(heap_pages)
        .with_max_runtime_instances(config.executor.max_runtime_instances)
        .with_runtime_cache_size(config.executor.runtime_cache_size)
        .build();

    let executor = ParachainExecutor::new_with_wasm_executor(wasm);

    let (client, backend, keystore_container, task_manager) =
        sc_service::new_full_parts::<Block, RuntimeApi, _>(
            config,
            telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
            executor,
        )?;
    let client = Arc::new(client);

    let telemetry_worker_handle = telemetry.as_ref().map(|(worker, _)| worker.handle());

    let telemetry = telemetry.map(|(worker, telemetry)| {
        task_manager.spawn_handle().spawn("telemetry", None, worker.run());
        telemetry
    });

    let transaction_pool = sc_transaction_pool::BasicPool::new_full(
        config.transaction_pool.clone(),
        config.role.is_authority().into(),
        config.prometheus_registry(),
        task_manager.spawn_essential_handle(),
        client.clone(),
    );

    let block_import = ParachainBlockImport::<RuntimeApi>::new(client.clone(), backend.clone());

    let import_queue = nimbus_consensus::import_queue(
        client.clone(),
        block_import.clone(),
        move |_, _| async move {
            let time = sp_timestamp::InherentDataProvider::from_system_time();
            Ok((time,))
        },
        &task_manager.spawn_essential_handle(),
        config.prometheus_registry(),
        false,
    )?;

    Ok(ParachainPartialComponents {
        backend,
        client,
        import_queue,
        keystore_container,
        task_manager,
        transaction_pool,
        select_chain: (),
        other: (block_import, telemetry, telemetry_worker_handle),
    })
}

/// Start a node with the given parachain `Configuration` and relay chain `Configuration`.
///
/// This is the actual implementation that is abstract over the executor and the runtime api.
#[sc_tracing::logging::prefix_logs_with("🔮 Zeitgeist Parachain")]
async fn do_new_full<RuntimeApi, N>(
    parachain_config: Configuration,
    polkadot_config: Configuration,
    para_id: polkadot_primitives::Id,
    hwbench: Option<sc_sysinfo::HwBench>,
    collator_options: CollatorOptions,
) -> ServiceResult<(TaskManager, Arc<FullClient<RuntimeApi>>)>
where
    RuntimeApi: ConstructRuntimeApi<Block, FullClient<RuntimeApi>> + Send + Sync + 'static,
    RuntimeApi::RuntimeApi: RuntimeApiCollection + AdditionalRuntimeApiCollection,
    N: sc_network::NetworkBackend<Block, <Block as BlockT>::Hash>,
{
    let parachain_config = prepare_node_config(parachain_config);

    let params = new_partial::<RuntimeApi>(&parachain_config)?;
    let (block_import, mut telemetry, telemetry_worker_handle) = params.other;

    let client = params.client.clone();
    let backend = params.backend.clone();
    let mut task_manager = params.task_manager;

    let (relay_chain_interface, collator_key) = build_relay_chain_interface(
        polkadot_config,
        &parachain_config,
        telemetry_worker_handle,
        &mut task_manager,
        collator_options,
        hwbench.clone(),
    )
    .await
    .map_err(|e| ServiceError::Application(Box::new(e) as Box<_>))?;

    let block_announce_validator =
        RequireSecondedInBlockAnnounce::new(relay_chain_interface.clone(), para_id);

    let collator = parachain_config.role.is_authority();
    let prometheus_registry = parachain_config.prometheus_registry().cloned();
    let transaction_pool = params.transaction_pool.clone();
    let import_queue_service = params.import_queue.service();

    let net_config = FullNetworkConfiguration::<_, _, N>::new(
        &parachain_config.network,
        prometheus_registry.clone(),
    );

    let metrics = N::register_notification_metrics(
        parachain_config.prometheus_config.as_ref().map(|cfg| &cfg.registry),
    );
    let (network, system_rpc_tx, tx_handler_controller, start_network, sync_service) =
        sc_service::build_network(sc_service::BuildNetworkParams {
            config: &parachain_config,
            client: client.clone(),
            transaction_pool: transaction_pool.clone(),
            spawn_handle: task_manager.spawn_handle(),
            import_queue: params.import_queue,
            block_announce_validator_builder: Some(Box::new(|_| {
                Box::new(block_announce_validator)
            })),
            warp_sync_config: None,
            net_config,
            block_relay: None,
            metrics,
        })?;

    if parachain_config.offchain_worker.enabled {
        task_manager.spawn_handle().spawn(
            "offchain-workers-runner",
            "offchain-work",
            sc_offchain::OffchainWorkers::new(sc_offchain::OffchainWorkerOptions {
                runtime_api_provider: client.clone(),
                keystore: Some(params.keystore_container.keystore()),
                offchain_db: backend.offchain_storage(),
                transaction_pool: Some(OffchainTransactionPoolFactory::new(
                    transaction_pool.clone(),
                )),
                network_provider: Arc::new(network.clone()),
                is_validator: parachain_config.role.is_authority(),
                enable_http_requests: false,
                custom_extensions: move |_| vec![],
            })
            .run(client.clone(), task_manager.spawn_handle())
            .boxed(),
        );
    }

    let rpc_extensions_builder = {
        let client = client.clone();
        let transaction_pool = transaction_pool.clone();

        Box::new(move |_| {
            let deps =
                crate::rpc::FullDeps { client: client.clone(), pool: transaction_pool.clone() };

            crate::rpc::create_full(deps).map_err(Into::into)
        })
    };

    let force_authoring = parachain_config.force_authoring;

    sc_service::spawn_tasks(sc_service::SpawnTasksParams {
        rpc_builder: rpc_extensions_builder,
        client: client.clone(),
        transaction_pool: transaction_pool.clone(),
        task_manager: &mut task_manager,
        config: parachain_config,
        keystore: params.keystore_container.keystore(),
        backend: backend.clone(),
        network,
        sync_service: sync_service.clone(),
        system_rpc_tx,
        tx_handler_controller,
        telemetry: telemetry.as_mut(),
    })?;

    if let Some(hwbench) = hwbench {
        sc_sysinfo::print_hwbench(&hwbench);
        // Here you can check whether the hardware meets your chains' requirements. Putting a link
        // in there and swapping out the requirements for your own are probably a good idea. The
        // requirements for a para-chain are dictated by its relay-chain.
        match SUBSTRATE_REFERENCE_HARDWARE.check_hardware(&hwbench, false) {
            Err(err) if collator => {
                log::warn!(
                    "⚠️  The hardware does not meet the minimal requirements {} for role \
                     'Authority'.",
                    err
                );
            }
            _ => {}
        }

        if let Some(ref mut telemetry) = telemetry {
            let telemetry_handle = telemetry.handle();
            task_manager.spawn_handle().spawn(
                "telemetry_hwbench",
                None,
                sc_sysinfo::initialize_hwbench_telemetry(telemetry_handle, hwbench),
            );
        }
    }

    let announce_block = {
        let sync_service = sync_service.clone();
        Arc::new(move |hash, data| sync_service.announce_block(hash, data))
    };

    let relay_chain_slot_duration = POLKADOT_BLOCK_DURATION;
    let overseer_handle = relay_chain_interface
        .overseer_handle()
        .map_err(|e| sc_service::Error::Application(Box::new(e)))?;

    start_relay_chain_tasks(StartRelayChainTasksParams {
        client: client.clone(),
        announce_block: announce_block.clone(),
        para_id,
        relay_chain_interface: relay_chain_interface.clone(),
        task_manager: &mut task_manager,
        da_recovery_profile: if collator {
            DARecoveryProfile::Collator
        } else {
            DARecoveryProfile::FullNode
        },
        import_queue: import_queue_service,
        relay_chain_slot_duration,
        recovery_handle: Box::new(overseer_handle.clone()),
        sync_service: sync_service.clone(),
    })?;

    if collator {
        start_consensus(
            client.clone(),
            block_import,
            prometheus_registry.as_ref(),
            telemetry.as_ref().map(|t| t.handle()),
            &task_manager,
            relay_chain_interface.clone(),
            transaction_pool,
            params.keystore_container.keystore(),
            para_id,
            collator_key.expect("Command line arguments do not allow this. qed"),
            overseer_handle,
            announce_block,
            force_authoring,
            true,
        )?;
    }

    start_network.start_network();

    Ok((task_manager, client))
}

fn start_consensus<RuntimeApi>(
    client: Arc<FullClient<RuntimeApi>>,
    block_import: ParachainBlockImport<RuntimeApi>,
    prometheus_registry: Option<&Registry>,
    telemetry: Option<TelemetryHandle>,
    task_manager: &TaskManager,
    relay_chain_interface: Arc<dyn RelayChainInterface>,
    transaction_pool: Arc<sc_transaction_pool::FullPool<Block, FullClient<RuntimeApi>>>,
    keystore: KeystorePtr,
    para_id: ParaId,
    collator_key: CollatorPair,
    overseer_handle: OverseerHandle,
    announce_block: Arc<dyn Fn(Hash, Option<Vec<u8>>) + Send + Sync>,
    force_authoring: bool,
    full_pov_size: bool,
) -> Result<(), sc_service::Error>
where
    RuntimeApi: ConstructRuntimeApi<Block, FullClient<RuntimeApi>> + Send + Sync + 'static,
    RuntimeApi::RuntimeApi: RuntimeApiCollection + AdditionalRuntimeApiCollection,
{
    let proposer_factory = sc_basic_authorship::ProposerFactory::with_proof_recording(
        task_manager.spawn_handle(),
        client.clone(),
        transaction_pool,
        prometheus_registry,
        telemetry.clone(),
    );

    let proposer = Proposer::new(proposer_factory);

    let collator_service = CollatorService::new(
        client.clone(),
        Arc::new(task_manager.spawn_handle()),
        announce_block,
        client.clone(),
    );

    let params = nimbus_consensus::collators::basic::Params {
        para_id,
        overseer_handle,
        proposer,
        create_inherent_data_providers: move |_, _| async move { Ok(()) },
        block_import,
        relay_client: relay_chain_interface,
        para_client: client,
        keystore,
        collator_service,
        force_authoring,
        full_pov_size,
        additional_digests_provider: (),
        additional_relay_keys: vec![],
        collator_key,
        //authoring_duration: Duration::from_millis(500),
    };

    let fut =
        nimbus_consensus::collators::basic::run::<Block, _, _, FullBackend, _, _, _, _, _>(params);
    task_manager.spawn_essential_handle().spawn("nimbus", None, fut);

    Ok(())
}
