use crate::service::Executor;
use cumulus_client_network::build_block_announce_validator;
use cumulus_client_service::{
    prepare_node_config, start_collator, start_full_node, StartCollatorParams, StartFullNodeParams,
};
use cumulus_primitives_core::ParaId;
use nimbus_consensus::{build_nimbus_consensus, BuildNimbusConsensusParams};
use nimbus_primitives::NimbusId;
use sc_service::{Configuration, PartialComponents, Role, TFullBackend, TFullClient, TaskManager};
use sc_telemetry::{Telemetry, TelemetryWorker, TelemetryWorkerHandle};
use std::sync::Arc;
use zeitgeist_runtime::{opaque::Block, RuntimeApi};

type FullClient<RuntimeApi, Executor> = TFullClient<Block, RuntimeApi, Executor>;

/// Start a parachain node.
pub async fn new_full(
    parachain_config: Configuration,
    parachain_id: ParaId,
    polkadot_config: Configuration,
) -> sc_service::error::Result<(TaskManager, Arc<TFullClient<Block, RuntimeApi, Executor>>)> {
    do_new_full(parachain_config, parachain_id, polkadot_config, |_| Default::default()).await
}

pub fn new_partial(
    config: &Configuration,
) -> Result<
    PartialComponents<
        TFullClient<Block, RuntimeApi, Executor>,
        TFullBackend<Block>,
        (),
        sp_consensus::DefaultImportQueue<Block, TFullClient<Block, RuntimeApi, Executor>>,
        sc_transaction_pool::FullPool<Block, FullClient<RuntimeApi, Executor>>,
        (Option<Telemetry>, Option<TelemetryWorkerHandle>),
    >,
    sc_service::Error,
> {
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

    let (client, backend, keystore_container, task_manager) =
        sc_service::new_full_parts::<Block, RuntimeApi, Executor>(
            &config,
            telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
        )?;

    let client = Arc::new(client);

    let telemetry_worker_handle = telemetry.as_ref().map(|(worker, _)| worker.handle());

    let telemetry = telemetry.map(|(worker, telemetry)| {
        task_manager.spawn_handle().spawn("telemetry", worker.run());
        telemetry
    });

    let transaction_pool = sc_transaction_pool::BasicPool::new_full(
        config.transaction_pool.clone(),
        config.role.is_authority().into(),
        config.prometheus_registry(),
        task_manager.spawn_handle(),
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
    )?;

    Ok(PartialComponents {
        backend,
        client,
        import_queue,
        other: (telemetry, telemetry_worker_handle),
        keystore_container,
        select_chain: (),
        task_manager,
        transaction_pool,
    })
}

/// Start a node with the given parachain `Configuration` and relay chain `Configuration`.
///
/// This is the actual implementation that is abstract over the executor and the runtime api.
#[sc_tracing::logging::prefix_logs_with("🌔 Zeitgeist Parachain")]
async fn do_new_full<RB>(
    parachain_config: Configuration,
    parachain_id: ParaId,
    polkadot_config: Configuration,
    rpc_ext_builder: RB,
) -> sc_service::error::Result<(TaskManager, Arc<TFullClient<Block, RuntimeApi, Executor>>)>
where
    RB: Fn(
            Arc<TFullClient<Block, RuntimeApi, Executor>>,
        ) -> jsonrpc_core::IoHandler<sc_rpc::Metadata>
        + Send
        + 'static,
{
    if matches!(parachain_config.role, Role::Light) {
        return Err("Light client not supported!".into());
    }

    let parachain_config = prepare_node_config(parachain_config);

    let params = new_partial(&parachain_config)?;

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
        parachain_id,
        Box::new(relay_chain_full_node.network.clone()),
        relay_chain_full_node.backend.clone(),
    );

    let is_collator = parachain_config.role.is_authority();
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
            on_demand: None,
            block_announce_validator_builder: Some(Box::new(|_| block_announce_validator)),
        })?;

    let rpc_client = client.clone();
    let rpc_extensions_builder = Box::new(move |_, _| rpc_ext_builder(rpc_client.clone()));

    sc_service::spawn_tasks(sc_service::SpawnTasksParams {
        backend: backend.clone(),
        client: client.clone(),
        config: parachain_config,
        keystore: params.keystore_container.sync_keystore(),
        network: network.clone(),
        on_demand: None,
        remote_blockchain: None,
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

    if is_collator {
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
            block_import: client.clone(),
            create_inherent_data_providers: move |_, (relay_parent, validation_data, author_id)| {
                let parachain_inherent =
                cumulus_primitives_parachain_inherent::ParachainInherentData::
                create_at_with_client(
                    relay_parent,
                    &relay_chain_client,
                    &*relay_chain_backend,
                    &validation_data,
                    parachain_id,
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
            keystore: params.keystore_container.sync_keystore(),
            para_id: parachain_id,
            parachain_client: client.clone(),
            proposer_factory,
            relay_chain_backend: relay_chain_full_node.backend.clone(),
            relay_chain_client: relay_chain_full_node.client.clone(),
        });

        let spawner = task_manager.spawn_handle();

        let params = StartCollatorParams {
            announce_block,
            block_status: client.clone(),
            client: client.clone(),
            import_queue,
            para_id: parachain_id,
            parachain_consensus,
            relay_chain_full_node,
            spawner,
            task_manager: &mut task_manager,
        };

        start_collator(params).await?;
    } else {
        let params = StartFullNodeParams {
            announce_block,
            client: client.clone(),
            para_id: parachain_id,
            relay_chain_full_node,
            task_manager: &mut task_manager,
        };

        start_full_node(params)?;
    }

    start_network.start_network();

    Ok((task_manager, client))
}
