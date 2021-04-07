use crate::service::Executor;
use cumulus_client_consensus_relay_chain::{
    build_relay_chain_consensus, BuildRelayChainConsensusParams,
};
use cumulus_client_network::build_block_announce_validator;
use cumulus_client_service::{
    prepare_node_config, start_collator, start_full_node, StartCollatorParams, StartFullNodeParams,
};
use cumulus_primitives_core::ParaId;
use polkadot_primitives::v0::CollatorPair;
use sc_service::{Configuration, PartialComponents, Role, TFullBackend, TFullClient, TaskManager};
use sc_telemetry::{Telemetry, TelemetryWorker, TelemetryWorkerHandle};
use sp_runtime::traits::BlakeTwo256;
use sp_trie::PrefixedMemoryDB;
use std::sync::Arc;
use zeitgeist_runtime::{opaque::Block, RuntimeApi};

/// Start a parachain node.
pub async fn new_full(
    parachain_config: Configuration,
    collator_key: CollatorPair,
    polkadot_config: Configuration,
    id: ParaId,
    validator: bool,
) -> sc_service::error::Result<(TaskManager, Arc<TFullClient<Block, RuntimeApi, Executor>>)> {
    do_new_full(
        parachain_config,
        collator_key,
        polkadot_config,
        id,
        validator,
        |_| Default::default(),
    )
    .await
}

pub fn new_partial(
    config: &Configuration,
) -> Result<
    PartialComponents<
        TFullClient<Block, RuntimeApi, Executor>,
        TFullBackend<Block>,
        (),
        sp_consensus::import_queue::BasicQueue<Block, PrefixedMemoryDB<BlakeTwo256>>,
        sc_transaction_pool::FullPool<Block, TFullClient<Block, RuntimeApi, Executor>>,
        (Option<Telemetry>, Option<TelemetryWorkerHandle>),
    >,
    sc_service::Error,
> {
    let inherent_data_providers = sp_inherents::InherentDataProviders::new();

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

    let registry = config.prometheus_registry();

    let transaction_pool = sc_transaction_pool::BasicPool::new_full(
        config.transaction_pool.clone(),
        config.role.is_authority().into(),
        config.prometheus_registry(),
        task_manager.spawn_handle(),
        client.clone(),
    );

    let import_queue = cumulus_client_consensus_relay_chain::import_queue(
        client.clone(),
        client.clone(),
        inherent_data_providers.clone(),
        &task_manager.spawn_essential_handle(),
        registry.clone(),
    )?;

    let params = PartialComponents {
        backend,
        client,
        import_queue,
        inherent_data_providers,
        keystore_container,
        other: (telemetry, telemetry_worker_handle),
        select_chain: (),
        task_manager,
        transaction_pool,
    };

    Ok(params)
}

/// Start a node with the given parachain `Configuration` and relay chain `Configuration`.
///
/// This is the actual implementation that is abstract over the executor and the runtime api.
#[sc_tracing::logging::prefix_logs_with("Parachain")]
async fn do_new_full<RB>(
    parachain_config: Configuration,
    collator_key: CollatorPair,
    polkadot_config: Configuration,
    id: ParaId,
    validator: bool,
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
    params
        .inherent_data_providers
        .register_provider(sp_timestamp::InherentDataProvider)
        .unwrap();
    let (mut telemetry, telemetry_worker_handle) = params.other;

    let polkadot_full_node = cumulus_client_service::build_polkadot_full_node(
        polkadot_config,
        collator_key.clone(),
        telemetry_worker_handle,
    )
    .map_err(|e| match e {
        polkadot_service::Error::Sub(x) => x,
        s => format!("{}", s).into(),
    })?;

    let client = params.client.clone();
    let backend = params.backend.clone();
    let block_announce_validator = build_block_announce_validator(
        polkadot_full_node.client.clone(),
        id,
        Box::new(polkadot_full_node.network.clone()),
        polkadot_full_node.backend.clone(),
    );

    let prometheus_registry = parachain_config.prometheus_registry().cloned();
    let transaction_pool = params.transaction_pool.clone();
    let mut task_manager = params.task_manager;
    let import_queue = params.import_queue;
    let (network, network_status_sinks, system_rpc_tx, start_network) =
        sc_service::build_network(sc_service::BuildNetworkParams {
            config: &parachain_config,
            client: client.clone(),
            transaction_pool: transaction_pool.clone(),
            spawn_handle: task_manager.spawn_handle(),
            import_queue,
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
        network_status_sinks,
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

    if validator {
        let proposer_factory = sc_basic_authorship::ProposerFactory::with_proof_recording(
            task_manager.spawn_handle(),
            client.clone(),
            transaction_pool,
            prometheus_registry.as_ref(),
            telemetry.as_ref().map(|x| x.handle()),
        );
        let spawner = task_manager.spawn_handle();

        let parachain_consensus = build_relay_chain_consensus(BuildRelayChainConsensusParams {
            para_id: id,
            proposer_factory,
            inherent_data_providers: params.inherent_data_providers,
            block_import: client.clone(),
            relay_chain_client: polkadot_full_node.client.clone(),
            relay_chain_backend: polkadot_full_node.backend.clone(),
        });

        let params = StartCollatorParams {
            para_id: id,
            block_status: client.clone(),
            announce_block,
            client: client.clone(),
            task_manager: &mut task_manager,
            collator_key,
            relay_chain_full_node: polkadot_full_node,
            spawner,
            backend,
            parachain_consensus,
        };

        start_collator(params).await?;
    } else {
        let params = StartFullNodeParams {
            client: client.clone(),
            announce_block,
            task_manager: &mut task_manager,
            para_id: id,
            polkadot_full_node,
        };

        start_full_node(params)?;
    }

    start_network.start_network();

    Ok((task_manager, client))
}
