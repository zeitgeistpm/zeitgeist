use super::{
    cli::{Cli, Subcommand},
    //command_helper::{inherent_benchmark_data, BenchmarkExtrinsicBuilder},
    service::{new_partial, new_chain_ops},
};
use frame_benchmarking_cli::BenchmarkCmd;
use sc_cli::SubstrateCli;
use std::sync::Arc;
#[cfg(feature = "parachain")]
use {
    sc_client_api::client::BlockBackend, sp_core::hexdisplay::HexDisplay, sp_core::Encode,
    sp_runtime::traits::Block as BlockT, std::io::Write,
};

pub fn run() -> sc_cli::Result<()> {
    let mut cli = <Cli as SubstrateCli>::from_args();

    // Set default chain on parachain to zeitgeist and on standalone to dev
    #[cfg(feature = "parachain")]
    if cli.run.base.shared_params.chain == None {
        cli.run.base.shared_params.chain = Some("zeitgeist".to_string());
    }
    #[cfg(not(feature = "parachain"))]
    if cli.run.shared_params.chain == None {
        cli.run.shared_params.dev = true;
    }

    match &cli.subcommand {
        Some(Subcommand::Benchmark(cmd)) => {
            /*
            let runner = cli.create_runner(cmd)?;

            runner.sync_run(|config| {
                let (client, backend, _, _) = new_chain_ops(&mut config)?;
                let chain_spec = &runner.config().chain_spec;

                // This switch needs to be in the client, since the client decides
                // which sub-commands it wants to support.
                match cmd {
                    BenchmarkCmd::Pallet(cmd) => {
                        if !cfg!(feature = "runtime-benchmarks") {
                            return Err("Runtime benchmarking wasn't enabled when building the \
                                        node. You can enable it with `--features \
                                        runtime-benchmarks`."
                                .into());
                        }

                        /*
                        match chain_spec {
							#[cfg(feature = "with-zeitgeist-runtime")]
							spec if spec.is_zeitgeist() => {
								return runner.sync_run(|config| {
									cmd.run::<zeitgeist_runtime::Block, service::ZeitgeistExecutor>(
										config,
									)
								})
							}
							#[cfg(feature = "with-battery-station-runtime")]
							spec if spec.is_battery_station() => {
								return runner.sync_run(|config| {
									cmd.run::<battery_station::Block, service::BatteryStationExecutor>(
										config,
									)
								})
							}
							_ => panic!("invalid chain spec"),
						}*/
                        cmd.run(config)
                    }
                    BenchmarkCmd::Block(cmd) => cmd.run(client),
                    BenchmarkCmd::Storage(cmd) => {
                        let db = backend.expose_db();
                        let storage = backend.expose_storage();

                        cmd.run(config, client, db, storage)
                    }
                    /*
                    BenchmarkCmd::Overhead(cmd) => {
                        if cfg!(feature = "parachain") {
                            Err("Overhead is only supported in standalone chain".into())
                        } else {
                            let ext_builder = BenchmarkExtrinsicBuilder::new(client.clone());
                            cmd.run(
                                config,
                                client,
                                inherent_benchmark_data()?,
                                Arc::new(ext_builder),
                            )
                        }
                    }
                    */
                }
            })
            */
            Ok(())
        }
        Some(Subcommand::BuildSpec(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
        }
        Some(Subcommand::CheckBlock(cmd)) => {
            let runner = cli.create_runner(cmd)?;
			runner.async_run(|mut config| {
				let (client, _, import_queue, task_manager) = new_chain_ops(&mut config)?;
				Ok((cmd.run(client, import_queue), task_manager))
			})
        }
        Some(Subcommand::ExportBlocks(cmd)) => {
            let runner = cli.create_runner(cmd)?;
			runner.async_run(|mut config| {
				let (client, _, _, task_manager) = new_chain_ops(&mut config)?;
				Ok((cmd.run(client, config.database), task_manager))
			})
        }
        #[cfg(feature = "parachain")]
        Some(Subcommand::ExportHeader(cmd)) => {
            let runner = cli.create_runner(cmd)?;

            runner.sync_run(|mut config| {
                let (client, _, _, _) = new_chain_ops(&mut config)?;

                match client.block(&cmd.input.parse()?) {
                    Ok(Some(block)) => {
                        println!("0x{:?}", HexDisplay::from(&block.block.header.encode()));
                        Ok(())
                    }
                    Ok(None) => Err("Unknown block".into()),
                    Err(e) => Err(format!("Error reading block: {:?}", e).into()),
                }
            })
        }
        #[cfg(feature = "parachain")]
        Some(Subcommand::ExportGenesisState(params)) => {
            let mut builder = sc_cli::LoggerBuilder::new("");
            builder.with_profiling(sc_tracing::TracingReceiver::Log, "");
            let _ = builder.init();
            let chain_spec = &crate::cli::load_spec(
                &params.chain.clone().unwrap_or_default(),
                params.parachain_id.into(),
            )?;
            let state_version = Cli::native_runtime_version(chain_spec).state_version();

            let block: zeitgeist_runtime::Block =
                cumulus_client_service::genesis::generate_genesis_block(chain_spec, state_version)?;
            let raw_header = block.header().encode();
            let buf = if params.raw {
                raw_header
            } else {
                format!("0x{:?}", HexDisplay::from(&block.header().encode())).into_bytes()
            };

            if let Some(output) = &params.output {
                std::fs::write(output, buf)?;
            } else {
                std::io::stdout().write_all(&buf)?;
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
			runner.async_run(|mut config| {
				let (client, _, _, task_manager) = new_chain_ops(&mut config)?;
				Ok((cmd.run(client, config.chain_spec), task_manager))
			})
        }
        Some(Subcommand::ImportBlocks(cmd)) => {
            let runner = cli.create_runner(cmd)?;
			runner.async_run(|mut config| {
				let (client, _, import_queue, task_manager) = new_chain_ops(&mut config)?;
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
                    config.tokio_handle.clone(),
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
            /*
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let (client, backend, _, task_manager) = new_chain_ops(&mut config)?;

                let aux_revert = Box::new(move |client, _, blocks| {
                    sc_finality_grandpa::revert(client, blocks)?;
                    Ok(())
                });

                Ok((cmd.run(client, backend, Some(aux_revert)), task_manager))
            })
            */
            Ok(())
        }
        #[cfg(feature = "try-runtime")]
        Some(Subcommand::TryRuntime(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                // we don't need any of the components of new_partial, just a runtime, or a task
                // manager to do `async_run`.
                let registry = config.prometheus_config.as_ref().map(|cfg| &cfg.registry);
                let task_manager =
                    sc_service::TaskManager::new(config.tokio_handle.clone(), registry)
                        .map_err(|e| sc_cli::Error::Service(sc_service::Error::Prometheus(e)))?;

                Ok((cmd.run::<Block, ExecutorDispatch>(config), task_manager))
            })
        }
        #[cfg(not(feature = "try-runtime"))]
        Some(Subcommand::TryRuntime) => Err("TryRuntime wasn't enabled when building the node. \
                                             You can enable it with `--features try-runtime`."
            .into()),
        None => none_command(&cli),
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
fn none_command(cli: &Cli) -> sc_cli::Result<()> {
    let runner = cli.create_runner(&cli.run.normalize())?;

    runner.run_node_until_exit(|parachain_config| async move {
        let chain_spec = &parachain_config.chain_spec;
        let parachain_id_extension =
            crate::chain_spec::Extensions::try_get(&**chain_spec).map(|e| e.parachain_id);

        let polkadot_cli = crate::cli::RelayChainCli::new(
            &parachain_config,
            [crate::cli::RelayChainCli::executable_name()].iter().chain(cli.relaychain_args.iter()),
        );

        let parachain_id = cumulus_primitives_core::ParaId::from(
            cli.parachain_id.or(parachain_id_extension).unwrap_or(super::KUSAMA_PARACHAIN_ID),
        );

        let parachain_account = polkadot_parachain::primitives::AccountIdConversion::<
            polkadot_primitives::v2::AccountId,
        >::into_account(&parachain_id);

        let state_version = Cli::native_runtime_version(chain_spec).state_version();
        let block: zeitgeist_runtime::Block =
            cumulus_client_service::genesis::generate_genesis_block(chain_spec, state_version)
                .map_err(|e| format!("{:?}", e))?;
        let genesis_state = format!("0x{:?}", HexDisplay::from(&block.header().encode()));

        let tokio_handle = parachain_config.tokio_handle.clone();
        let polkadot_config =
            SubstrateCli::create_configuration(&polkadot_cli, &polkadot_cli, tokio_handle)
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
fn none_command(cli: &Cli) -> sc_cli::Result<()> {
    let runner = cli.create_runner(&cli.run)?;
    runner.run_node_until_exit(|config| async move {
        match config.role {
            sc_cli::Role::Light => return Err("Light client not supported!".into()),
            _ => crate::service::new_full(config),
        }
        .map_err(sc_cli::Error::Service)
    })
}
