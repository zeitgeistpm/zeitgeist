use crate::service::{
    AdditionalRuntimeApiCollection, CommonRuntimeApiCollection, ExecutorDispatch,
};
use cumulus_client_network::build_block_announce_validator;
use cumulus_client_service::{
    prepare_node_config, start_collator, start_full_node, StartCollatorParams, StartFullNodeParams,
};
use cumulus_primitives_core::ParaId;
use cumulus_primitives_parachain_inherent::ParachainInherentData;
use nimbus_consensus::{build_nimbus_consensus, BuildNimbusConsensusParams};
use nimbus_primitives::NimbusId;
use sc_executor::{NativeElseWasmExecutor, NativeExecutionDispatch};
use sc_service::{Configuration, PartialComponents, Role, TFullBackend, TFullClient, TaskManager};
use sc_telemetry::{Telemetry, TelemetryWorker, TelemetryWorkerHandle};
use sp_api::ConstructRuntimeApi;
use std::sync::Arc;
use zeitgeist_runtime::{opaque::Block, RuntimeApi};

type FullBackend = TFullBackend<Block>;
type FullClient<RuntimeApi, Executor> =
    TFullClient<Block, RuntimeApi, NativeElseWasmExecutor<Executor>>;
pub type ParachainPartialComponents<Executor, RuntimeApi> = PartialComponents<
    FullClient<RuntimeApi, Executor>,
    FullBackend,
    (),
    sc_consensus::DefaultImportQueue<Block, FullClient<RuntimeApi, Executor>>,
    sc_transaction_pool::FullPool<Block, FullClient<RuntimeApi, Executor>>,
    (Option<Telemetry>, Option<TelemetryWorkerHandle>),
>;

/// Start a parachain node.
pub async fn new_full(
    parachain_config: Configuration,
    parachain_id: ParaId,
    polkadot_config: Configuration,
) -> sc_service::error::Result<(TaskManager, Arc<FullClient<RuntimeApi, ExecutorDispatch>>)> {
    do_new_full::<RuntimeApi, ExecutorDispatch>(parachain_config, polkadot_config, parachain_id)
        .await
}

/// Builds the PartialComponents for a parachain or development service
///
/// Use this function if you don't actually need the full service, but just the partial in order to
/// be able to perform chain operations.
#[allow(clippy::type_complexity)]
pub fn new_partial<RuntimeApi, Executor>(
    config: &Configuration,
) -> Result<ParachainPartialComponents<Executor, RuntimeApi>, sc_service::error::Error>
where
    RuntimeApi:
        ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>> + Send + Sync + 'static,
    RuntimeApi::RuntimeApi: CommonRuntimeApiCollection<
            StateBackend = sc_client_api::StateBackendFor<FullBackend, Block>,
        > + AdditionalRuntimeApiCollection<
            StateBackend = sc_client_api::StateBackendFor<FullBackend, Block>,
        >,
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

    let executor = NativeElseWasmExecutor::<Executor>::new(
        config.wasm_method,
        config.default_heap_pages,
        config.max_runtime_instances,
    );

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

    let import_queue = nimbus_consensus::import_queue(
        client.clone(),
        client.clone(),
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
        other: (telemetry, telemetry_worker_handle),
    })
}

/// Start a node with the given parachain `Configuration` and relay chain `Configuration`.
///
/// This is the actual implementation that is abstract over the executor and the runtime api.
#[sc_tracing::logging::prefix_logs_with("🌔 Zeitgeist Parachain")]
async fn do_new_full<RuntimeApi, Executor>(
    parachain_config: Configuration,
    polkadot_config: Configuration,
    id: polkadot_primitives::v0::Id,
) -> sc_service::error::Result<(TaskManager, Arc<FullClient<RuntimeApi, Executor>>)>
where
    RuntimeApi:
        ConstructRuntimeApi<Block, FullClient<RuntimeApi, Executor>> + Send + Sync + 'static,
    RuntimeApi::RuntimeApi: CommonRuntimeApiCollection<
            StateBackend = sc_client_api::StateBackendFor<FullBackend, Block>,
        > + AdditionalRuntimeApiCollection<
            StateBackend = sc_client_api::StateBackendFor<FullBackend, Block>,
        >,
    Executor: NativeExecutionDispatch + 'static,
{
    if matches!(parachain_config.role, Role::Light) {
        return Err("Light client not supported!".into());
    }

    let parachain_config = prepare_node_config(parachain_config);

    let params = new_partial::<RuntimeApi, Executor>(&parachain_config)?;
    let (mut telemetry, telemetry_worker_handle) = params.other;

    let relay_chain_full_node =
        cumulus_client_service::build_polkadot_full_node(polkadot_config, telemetry_worker_handle)
            .map_err(|e| match e {
                polkadot_service::Error::Sub(x) => x,
                s => format!("{}", s).into(),
            })?;

    let client = params.client.clone();
    let backend = params.backend.clone();
    let block_announce_validator = build_block_announce_validator(
        relay_chain_full_node.client.clone(),
        id,
        Box::new(relay_chain_full_node.network.clone()),
        relay_chain_full_node.backend.clone(),
    );

    let collator = parachain_config.role.is_authority();
    let prometheus_registry = parachain_config.prometheus_registry().cloned();
    let transaction_pool = params.transaction_pool.clone();
    let mut task_manager = params.task_manager;
    let import_queue = cumulus_client_service::SharedImportQueue::new(params.import_queue);
    let (network, system_rpc_tx, start_network) =
        sc_service::build_network(sc_service::BuildNetworkParams {
            config: &parachain_config,
            client: client.clone(),
            transaction_pool: transaction_pool.clone(),
            spawn_handle: task_manager.spawn_handle(),
            import_queue: import_queue.clone(),
            block_announce_validator_builder: Some(Box::new(|_| block_announce_validator)),
            warp_sync: None,
        })?;

    let rpc_extensions_builder = {
        let client = client.clone();
        let pool = transaction_pool.clone();

        Box::new(move |deny_unsafe, _| {
            let deps =
                crate::rpc::FullDeps { client: client.clone(), pool: pool.clone(), deny_unsafe };

            Ok(crate::rpc::create_full(deps))
        })
    };

    let skip_prediction = parachain_config.force_authoring;

    sc_service::spawn_tasks(sc_service::SpawnTasksParams {
        backend: backend.clone(),
        client: client.clone(),
        config: parachain_config,
        keystore: params.keystore_container.sync_keystore(),
        network: network.clone(),
        rpc_extensions_builder,
        system_rpc_tx,
        task_manager: &mut task_manager,
        telemetry: telemetry.as_mut(),
        transaction_pool: transaction_pool.clone(),
    })?;

    let announce_block = {
        let network = network.clone();
        Arc::new(move |hash, data| network.announce_block(hash, data))
    };

    if collator {
        let proposer_factory = sc_basic_authorship::ProposerFactory::with_proof_recording(
            task_manager.spawn_handle(),
            client.clone(),
            transaction_pool,
            prometheus_registry.as_ref(),
            telemetry.as_ref().map(|t| t.handle()),
        );

        let relay_chain_backend = relay_chain_full_node.backend.clone();
        let relay_chain_client = relay_chain_full_node.client.clone();

        let parachain_consensus = build_nimbus_consensus(BuildNimbusConsensusParams {
            para_id: id,
            proposer_factory,
            block_import: client.clone(),
            relay_chain_client: relay_chain_full_node.client.clone(),
            relay_chain_backend: relay_chain_full_node.backend.clone(),
            parachain_client: client.clone(),
            keystore: params.keystore_container.sync_keystore(),
            skip_prediction,
            create_inherent_data_providers: move |_, (relay_parent, validation_data, author_id)| {
                let parachain_inherent = ParachainInherentData::create_at_with_client(
                    relay_parent,
                    &relay_chain_client,
                    &*relay_chain_backend,
                    &validation_data,
                    id,
                );
                async move {
                    let time = sp_timestamp::InherentDataProvider::from_system_time();

                    let parachain_inherent = parachain_inherent.ok_or_else(|| {
                        Box::<dyn std::error::Error + Send + Sync>::from(
                            "Failed to create parachain inherent",
                        )
                    })?;

                    let author = nimbus_primitives::InherentDataProvider::<NimbusId>(author_id);

                    Ok((time, parachain_inherent, author))
                }
            },
        });

        let spawner = task_manager.spawn_handle();

        let params = StartCollatorParams {
            para_id: id,
            block_status: client.clone(),
            announce_block,
            client: client.clone(),
            task_manager: &mut task_manager,
            spawner,
            relay_chain_full_node,
            parachain_consensus,
            import_queue,
        };

        start_collator(params).await?;
    } else {
        let params = StartFullNodeParams {
            client: client.clone(),
            announce_block,
            task_manager: &mut task_manager,
            para_id: id,
            relay_chain_full_node,
        };

        start_full_node(params)?;
    }

    start_network.start_network();

    Ok((task_manager, client))
}
