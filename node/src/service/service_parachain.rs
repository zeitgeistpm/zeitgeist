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
    POLKADOT_BLOCK_DURATION, SOFT_DEADLINE_PERCENT,
};
use cumulus_client_cli::CollatorOptions;
use cumulus_client_consensus_common::{
    ParachainBlockImport as TParachainBlockImport, ParachainConsensus,
};
use cumulus_client_service::{build_network, BuildNetworkParams, CollatorSybilResistance};
#[allow(deprecated)]
// TODO(#1326): Resolve deprecation after upgrade to polkadot-v1.3.0
use cumulus_client_service::{
    build_relay_chain_interface, prepare_node_config, start_collator, start_full_node,
    StartCollatorParams, StartFullNodeParams,
};
use cumulus_primitives_core::ParaId;
use cumulus_relay_chain_interface::RelayChainInterface;
use futures::FutureExt;
use nimbus_consensus::{BuildNimbusConsensusParams, NimbusConsensus};
use nimbus_primitives::NimbusId;
use sc_client_api::Backend;
use sc_consensus::ImportQueue;
use sc_executor::{
    HeapAllocStrategy, NativeElseWasmExecutor, NativeExecutionDispatch, WasmExecutor,
    DEFAULT_HEAP_ALLOC_STRATEGY,
};
use sc_network::{config::FullNetworkConfiguration, NetworkBlock};
use sc_network_sync::SyncingService;
use sc_service::{
    error::{Error as ServiceError, Result as ServiceResult},
    Configuration, PartialComponents, TFullBackend, TFullClient, TaskManager,
};
use sc_telemetry::{Telemetry, TelemetryHandle, TelemetryWorker, TelemetryWorkerHandle};
use sc_transaction_pool_api::OffchainTransactionPoolFactory;
use sp_api::ConstructRuntimeApi;
use sp_keystore::KeystorePtr;
use std::sync::Arc;
use substrate_prometheus_endpoint::Registry;
use zeitgeist_primitives::types::{Block, Hash};

pub type FullBackend = TFullBackend<Block>;
pub type FullClient<RuntimeApi, Executor> =
    TFullClient<Block, RuntimeApi, NativeElseWasmExecutor<Executor>>;
pub type ParachainPartialComponents<Executor, RuntimeApi> = PartialComponents<
    FullClient<RuntimeApi, Executor>,
    FullBackend,
    (),
    sc_consensus::DefaultImportQueue<Block>,
    sc_transaction_pool::FullPool<Block, FullClient<RuntimeApi, Executor>>,
    (ParachainBlockImport<RuntimeApi, Executor>, Option<Telemetry>, Option<TelemetryWorkerHandle>),
>;
type ParachainBlockImport<RuntimeApi, Executor> =
    TParachainBlockImport<Block, Arc<FullClient<RuntimeApi, Executor>>, FullBackend>;

/// Start a parachain node.
pub async fn new_full<RuntimeApi, Executor>(
    parachain_config: Configuration,
    parachain_id: ParaId,
    polkadot_config: Configuration,
    hwbench: Option<sc_sysinfo::HwBench>,
    collator_options: CollatorOptions,
) -> ServiceResult<(TaskManager, Arc<FullClient<RuntimeApi, Executor>>)>
where
    RuntimeApi:
        ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>> + Send + Sync + 'static,
    RuntimeApi::RuntimeApi: RuntimeApiCollection + AdditionalRuntimeApiCollection,
    Executor: NativeExecutionDispatch + 'static,
{
    do_new_full(
        parachain_config,
        polkadot_config,
        parachain_id,
        |client,
         backend,
         prometheus_registry,
         telemetry,
         task_manager,
         relay_chain_interface,
         transaction_pool,
         _sync_oracle,
         keystore,
         force_authoring| {
            let mut proposer_factory = sc_basic_authorship::ProposerFactory::with_proof_recording(
                task_manager.spawn_handle(),
                client.clone(),
                transaction_pool,
                prometheus_registry,
                telemetry,
            );
            proposer_factory.set_soft_deadline(SOFT_DEADLINE_PERCENT);

            let provider = move |_, (relay_parent, validation_data, _author_id)| {
                let relay_chain_interface = relay_chain_interface.clone();
                async move {
                    let parachain_inherent =
                        cumulus_primitives_parachain_inherent::ParachainInherentData::create_at(
                            relay_parent,
                            &relay_chain_interface,
                            &validation_data,
                            parachain_id,
                        )
                        .await;

                    let time = sp_timestamp::InherentDataProvider::from_system_time();

                    let parachain_inherent = parachain_inherent.ok_or_else(|| {
                        Box::<dyn std::error::Error + Send + Sync>::from(
                            "Failed to create parachain inherent",
                        )
                    })?;

                    let author = nimbus_primitives::InherentDataProvider;
                    let randomness = session_keys_primitives::InherentDataProvider;

                    Ok((time, parachain_inherent, author, randomness))
                }
            };

            let client_clone = client.clone();
            let keystore_clone = keystore.clone();
            let maybe_provide_vrf_digest =
                move |nimbus_id: NimbusId,
                      parent: Hash|
                      -> Option<sp_runtime::generic::DigestItem> {
                    moonbeam_vrf::vrf_pre_digest::<Block, FullClient<RuntimeApi, Executor>>(
                        &client_clone,
                        &keystore_clone,
                        nimbus_id,
                        parent,
                    )
                };

            Ok(NimbusConsensus::build(BuildNimbusConsensusParams {
                para_id: parachain_id,
                backend,
                proposer_factory,
                block_import: client.clone(),
                parachain_client: client,
                keystore,
                skip_prediction: force_authoring,
                create_inherent_data_providers: provider,
                additional_digests_provider: maybe_provide_vrf_digest,
            }))
        },
        hwbench,
        collator_options,
    )
    .await
}

/// Builds the PartialComponents for a parachain or development service
///
/// Use this function if you don't actually need the full service, but just the partial in order to
/// be able to perform chain operations.
#[allow(clippy::type_complexity)]
pub fn new_partial<RuntimeApi, Executor>(
    config: &Configuration,
) -> Result<ParachainPartialComponents<Executor, RuntimeApi>, ServiceError>
where
    RuntimeApi:
        ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>> + Send + Sync + 'static,
    RuntimeApi::RuntimeApi: RuntimeApiCollection + AdditionalRuntimeApiCollection,
    Executor: NativeExecutionDispatch + 'static,
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
        .default_heap_pages
        .map_or(DEFAULT_HEAP_ALLOC_STRATEGY, |h| HeapAllocStrategy::Static { extra_pages: h as _ });

    let wasm_builder = WasmExecutor::builder()
        .with_execution_method(config.wasm_method)
        .with_onchain_heap_alloc_strategy(heap_pages)
        .with_offchain_heap_alloc_strategy(heap_pages)
        .with_ignore_onchain_heap_pages(true)
        .with_max_runtime_instances(config.max_runtime_instances)
        .with_runtime_cache_size(config.runtime_cache_size);

    let wasm_executor = wasm_builder.build();
    let executor = NativeElseWasmExecutor::<Executor>::new_with_wasm_executor(wasm_executor);
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
    let block_import =
        ParachainBlockImport::<RuntimeApi, Executor>::new(client.clone(), backend.clone());
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

    Ok(PartialComponents {
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
async fn do_new_full<RuntimeApi, Executor, BIC>(
    parachain_config: Configuration,
    polkadot_config: Configuration,
    id: polkadot_primitives::Id,
    build_consensus: BIC,
    hwbench: Option<sc_sysinfo::HwBench>,
    collator_options: CollatorOptions,
) -> ServiceResult<(TaskManager, Arc<FullClient<RuntimeApi, Executor>>)>
where
    RuntimeApi:
        ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>> + Send + Sync + 'static,
    RuntimeApi::RuntimeApi: RuntimeApiCollection + AdditionalRuntimeApiCollection,
    Executor: NativeExecutionDispatch + 'static,
    BIC: FnOnce(
        Arc<FullClient<RuntimeApi, Executor>>,
        Arc<sc_client_db::Backend<Block>>,
        Option<&Registry>,
        Option<TelemetryHandle>,
        &TaskManager,
        Arc<dyn RelayChainInterface>,
        Arc<sc_transaction_pool::FullPool<Block, FullClient<RuntimeApi, Executor>>>,
        Arc<SyncingService<Block>>,
        KeystorePtr,
        bool,
    ) -> Result<Box<dyn ParachainConsensus<Block>>, ServiceError>,
{
    let parachain_config = prepare_node_config(parachain_config);

    let params = new_partial::<RuntimeApi, Executor>(&parachain_config)?;
    let (_, mut telemetry, telemetry_worker_handle) = params.other;

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

    let force_authoring = parachain_config.force_authoring;
    let collator = parachain_config.role.is_authority();
    let prometheus_registry = parachain_config.prometheus_registry().cloned();
    let transaction_pool = params.transaction_pool.clone();
    let import_queue_service = params.import_queue.service();
    let net_config = FullNetworkConfiguration::new(&parachain_config.network);
    let (network, system_rpc_tx, tx_handler_controller, start_network, sync_service) =
        build_network(BuildNetworkParams {
            parachain_config: &parachain_config,
            client: client.clone(),
            transaction_pool: transaction_pool.clone(),
            spawn_handle: task_manager.spawn_handle(),
            import_queue: params.import_queue,
            para_id: id,
            relay_chain_interface: relay_chain_interface.clone(),
            net_config,
            sybil_resistance_level: CollatorSybilResistance::Resistant,
        })
        .await?;

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
                network_provider: network.clone(),
                is_validator: parachain_config.role.is_authority(),
                enable_http_requests: false,
                custom_extensions: move |_| vec![],
            })
            .run(client.clone(), task_manager.spawn_handle())
            .boxed(),
        );
    }

    let rpc_builder = {
        let client = client.clone();
        let pool = transaction_pool.clone();

        Box::new(move |deny_unsafe, _| {
            let deps =
                crate::rpc::FullDeps { client: client.clone(), pool: pool.clone(), deny_unsafe };

            crate::rpc::create_full(deps).map_err(Into::into)
        })
    };

    sc_service::spawn_tasks(sc_service::SpawnTasksParams {
        backend: backend.clone(),
        client: client.clone(),
        config: parachain_config,
        keystore: params.keystore_container.keystore(),
        network: network.clone(),
        rpc_builder,
        tx_handler_controller,
        sync_service: sync_service.clone(),
        system_rpc_tx,
        task_manager: &mut task_manager,
        telemetry: telemetry.as_mut(),
        transaction_pool: transaction_pool.clone(),
    })?;

    if let Some(hwbench) = hwbench {
        sc_sysinfo::print_hwbench(&hwbench);

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

    if collator {
        let parachain_consensus = build_consensus(
            client.clone(),
            backend,
            prometheus_registry.as_ref(),
            telemetry.as_ref().map(|t| t.handle()),
            &task_manager,
            relay_chain_interface.clone(),
            transaction_pool,
            sync_service.clone(),
            params.keystore_container.keystore(),
            force_authoring,
        )?;

        let spawner = task_manager.spawn_handle();

        let params = StartCollatorParams {
            para_id: id,
            block_status: client.clone(),
            announce_block,
            client: client.clone(),
            task_manager: &mut task_manager,
            relay_chain_interface,
            spawner,
            parachain_consensus,
            import_queue: import_queue_service,
            recovery_handle: Box::new(overseer_handle),
            collator_key: collator_key
                .ok_or_else(|| ServiceError::Other("Collator Key is None".to_string()))?,
            relay_chain_slot_duration,
            sync_service,
        };

        #[allow(deprecated)]
        // TODO(#1326): Resolve deprecation after upgrade to polkadot-v1.3.0
        start_collator(params).await?;
    } else {
        let params = StartFullNodeParams {
            client: client.clone(),
            announce_block,
            task_manager: &mut task_manager,
            para_id: id,
            recovery_handle: Box::new(overseer_handle),
            relay_chain_interface,
            relay_chain_slot_duration,
            import_queue: import_queue_service,
            sync_service,
        };

        #[allow(deprecated)]
        // TODO(#1326): Resolve deprecation after upgrade to polkadot-v1.3.0
        start_full_node(params)?;
    }

    start_network.start_network();

    Ok((task_manager, client))
}
