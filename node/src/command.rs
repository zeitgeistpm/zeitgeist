use crate::cli::{Cli, Subcommand};
use sc_cli::{ExecutionStrategy, SubstrateCli};
use sc_service::PartialComponents;
#[cfg(feature = "parachain")]
use {
    parity_scale_codec::Encode, sp_core::hexdisplay::HexDisplay,
    sp_runtime::traits::Block as BlockT, std::io::Write,
};

pub fn run() -> sc_cli::Result<()> {
    let mut cli = <Cli as SubstrateCli>::from_args();

    match &cli.subcommand {
        #[cfg(feature = "runtime-benchmarks")]
        Some(Subcommand::Benchmark(cmd)) => {
            if cfg!(feature = "runtime-benchmarks") {
                let runner = cli.create_runner(cmd)?;

                runner.sync_run(|config| {
                    cmd.run::<zeitgeist_runtime::Block, crate::service::Executor>(config)
                })
            } else {
                Err("Benchmarking wasn't enabled when building the node. You can enable it with \
                     `--features runtime-benchmarks`."
                    .into())
            }
        }
        Some(Subcommand::BuildSpec(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
        }
        Some(Subcommand::CheckBlock(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let PartialComponents { client, task_manager, import_queue, .. } =
                    crate::service::new_partial(&config)?;
                Ok((cmd.run(client, import_queue), task_manager))
            })
        }
        Some(Subcommand::ExportBlocks(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let PartialComponents { client, task_manager, .. } =
                    crate::service::new_partial(&config)?;
                Ok((cmd.run(client, config.database), task_manager))
            })
        }
        #[cfg(feature = "parachain")]
        Some(Subcommand::ExportGenesisState(params)) => {
            let mut builder = sc_cli::LoggerBuilder::new("");
            builder.with_profiling(sc_tracing::TracingReceiver::Log, "");
            let _ = builder.init();

            let block: zeitgeist_runtime::Block =
                cumulus_client_service::genesis::generate_genesis_block(&crate::cli::load_spec(
                    &params.chain.clone().unwrap_or_default(),
                    params.parachain_id.into(),
                )?)?;
            let raw_header = block.header().encode();
            let output_buf = if params.raw {
                raw_header
            } else {
                format!("0x{:?}", HexDisplay::from(&block.header().encode())).into_bytes()
            };

            if let Some(output) = &params.output {
                std::fs::write(output, output_buf)?;
            } else {
                std::io::stdout().write_all(&output_buf)?;
            }

            Ok(())
        }
        #[cfg(feature = "parachain")]
        Some(Subcommand::ExportGenesisWasm(params)) => {
            let mut builder = sc_cli::LoggerBuilder::new("");
            builder.with_profiling(sc_tracing::TracingReceiver::Log, "");
            let _ = builder.init();

            let raw_wasm_blob =
                extract_genesis_wasm(cli.load_spec(&params.chain.clone().unwrap_or_default())?)?;
            let output_buf = if params.raw {
                raw_wasm_blob
            } else {
                format!("0x{:?}", HexDisplay::from(&raw_wasm_blob)).into_bytes()
            };

            if let Some(output) = &params.output {
                std::fs::write(output, output_buf)?;
            } else {
                std::io::stdout().write_all(&output_buf)?;
            }

            Ok(())
        }
        Some(Subcommand::ExportState(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let PartialComponents { client, task_manager, .. } =
                    crate::service::new_partial(&config)?;
                Ok((cmd.run(client, config.chain_spec), task_manager))
            })
        }
        Some(Subcommand::ImportBlocks(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let PartialComponents { client, task_manager, import_queue, .. } =
                    crate::service::new_partial(&config)?;
                Ok((cmd.run(client, import_queue), task_manager))
            })
        }
        Some(Subcommand::Key(cmd)) => cmd.run(&cli),
        #[cfg(feature = "parachain")]
        Some(Subcommand::PurgeChain(cmd)) => {
            let runner = cli.create_runner(cmd)?;

            runner.sync_run(|config| {
                let polkadot_cli = crate::cli::RelayChainCli::new(
                    &config,
                    [crate::cli::RelayChainCli::executable_name()]
                        .iter()
                        .chain(cli.relaychain_args.iter()),
                );

                let polkadot_config = SubstrateCli::create_configuration(
                    &polkadot_cli,
                    &polkadot_cli,
                    config.task_executor.clone(),
                )
                .map_err(|err| format!("Relay chain argument error: {}", err))?;

                cmd.run(config, polkadot_config)
            })
        }
        #[cfg(not(feature = "parachain"))]
        Some(Subcommand::PurgeChain(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| cmd.run(config.database))
        }
        Some(Subcommand::Revert(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let PartialComponents { client, task_manager, backend, .. } =
                    crate::service::new_partial(&config)?;
                Ok((cmd.run(client, backend), task_manager))
            })
        }
        None => none_command(&mut cli),
    }
}

#[cfg(feature = "parachain")]
fn extract_genesis_wasm(chain_spec: Box<dyn sc_service::ChainSpec>) -> sc_cli::Result<Vec<u8>> {
    let mut storage = chain_spec.build_storage()?;

    storage
        .top
        .remove(sp_core::storage::well_known_keys::CODE)
        .ok_or_else(|| "Could not find wasm file in genesis state!".into())
}

#[cfg(feature = "parachain")]
fn none_command(cli: &mut Cli) -> sc_cli::Result<()> {
    manage_execution(&mut cli.run.base.import_params.execution_strategies.execution);

    let runner = cli.create_runner(&cli.run.normalize())?;

    runner.run_node_until_exit(|parachain_config| async move {
        let parachain_id_extension =
            crate::chain_spec::Extensions::try_get(&*parachain_config.chain_spec)
                .map(|e| e.parachain_id);

        let polkadot_cli = crate::cli::RelayChainCli::new(
            &parachain_config,
            [crate::cli::RelayChainCli::executable_name()].iter().chain(cli.relaychain_args.iter()),
        );

        let parachain_id = cumulus_primitives_core::ParaId::from(
            cli.run.parachain_id.or(parachain_id_extension).unwrap_or(crate::DEFAULT_PARACHAIN_ID),
        );

        let parachain_account = polkadot_parachain::primitives::AccountIdConversion::<
            polkadot_primitives::v0::AccountId,
        >::into_account(&parachain_id);

        let block: zeitgeist_runtime::Block =
            cumulus_client_service::genesis::generate_genesis_block(&parachain_config.chain_spec)
                .map_err(|e| format!("{:?}", e))?;
        let genesis_state = format!("0x{:?}", HexDisplay::from(&block.header().encode()));

        let task_executor = parachain_config.task_executor.clone();
        let polkadot_config =
            SubstrateCli::create_configuration(&polkadot_cli, &polkadot_cli, task_executor)
                .map_err(|err| format!("Relay chain argument error: {}", err))?;

        log::info!("Parachain id: {:?}", parachain_id);
        log::info!("Parachain Account: {}", parachain_account);
        log::info!("Parachain genesis state: {}", genesis_state);

        crate::service::new_full(parachain_config, parachain_id, polkadot_config)
            .await
            .map(|r| r.0)
            .map_err(Into::into)
    })
}

#[cfg(not(feature = "parachain"))]
fn none_command(cli: &mut Cli) -> sc_cli::Result<()> {
    manage_execution(&mut cli.run.import_params.execution_strategies.execution);

    let runner = cli.create_runner(&cli.run)?;

    runner.run_node_until_exit(|config| async move {
        match config.role {
            sc_cli::Role::Light => crate::service::new_light(config),
            _ => crate::service::new_full(config),
        }
        .map_err(sc_cli::Error::Service)
    })
}

// * If not set, makes WASM the default execution
//
// * If set, forbids non WASM executions
fn manage_execution(execution_strategy: &mut Option<ExecutionStrategy>) {
    let invalid_execution = || panic!("WASM execution is the only possible option");
    if let Some(elem) = execution_strategy {
        match elem {
            ExecutionStrategy::Both => invalid_execution(),
            ExecutionStrategy::Native => invalid_execution(),
            ExecutionStrategy::NativeElseWasm => invalid_execution(),
            ExecutionStrategy::Wasm => {}
        }
    } else {
        *execution_strategy = Some(ExecutionStrategy::Wasm);
    }
}
