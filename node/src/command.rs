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

use super::{
    cli::{Cli, Subcommand},
    service::{new_chain_ops, new_full, IdentifyVariant},
};
use sc_cli::SubstrateCli;
#[cfg(feature = "runtime-benchmarks")]
use {
    super::command_helper::{inherent_benchmark_data, BenchmarkExtrinsicBuilder},
    frame_benchmarking_cli::BenchmarkCmd,
    std::sync::Arc,
};
#[cfg(feature = "with-battery-station-runtime")]
use {
    super::service::BatteryStationExecutor,
    battery_station_runtime::RuntimeApi as BatteryStationRuntimeApi,
};
#[cfg(feature = "with-raumgeist-runtime")]
use {super::service::RaumgeistExecutor, raumgeist_runtime::RuntimeApi as RaumgeistRuntimeApi};
#[cfg(feature = "with-zeitgeist-runtime")]
use {super::service::ZeitgeistExecutor, zeitgeist_runtime::RuntimeApi as ZeitgeistRuntimeApi};
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
        #[cfg(not(feature = "runtime-benchmarks"))]
        Some(Subcommand::Benchmark(_)) => Err("Runtime benchmarking wasn't enabled when building \
                                               the node. You can enable it with `--features \
                                               runtime-benchmarks`."
            .into()),
        #[cfg(feature = "runtime-benchmarks")]
        Some(Subcommand::Benchmark(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            let chain_spec = &runner.config().chain_spec;
            let id = chain_spec.id().to_string().clone();

            match cmd {
                // This switch needs to be in the client, since the client decides
                // which sub-commands it wants to support.
                BenchmarkCmd::Pallet(cmd) => match chain_spec {
                    #[cfg(feature = "with-raumgeist-runtime")]
                    spec if spec.is_raumgeist() => runner.sync_run(|config| {
                        cmd.run::<raumgeist_runtime::Block, RaumgeistExecutor>(config)
                    }),
                    #[cfg(feature = "with-zeitgeist-runtime")]
                    spec if spec.is_zeitgeist() => runner.sync_run(|config| {
                        cmd.run::<zeitgeist_runtime::Block, ZeitgeistExecutor>(config)
                    }),
                    #[cfg(feature = "with-battery-station-runtime")]
                    _ => runner.sync_run(|config| {
                        cmd.run::<battery_station_runtime::Block, BatteryStationExecutor>(config)
                    }),
                    #[cfg(not(feature = "with-battery-station-runtime"))]
                    _ => panic!("{}", crate::BATTERY_STATION_RUNTIME_NOT_AVAILABLE),
                },
                BenchmarkCmd::Block(cmd) => match chain_spec {
                    #[cfg(feature = "with-raumgeist-runtime")]
                    spec if spec.is_raumgeist() => runner.sync_run(|config| {
                        let params = crate::service::new_partial::<
                            raumgeist_runtime::RuntimeApi,
                            RaumgeistExecutor,
                        >(&config)?;
                        cmd.run(params.client)
                    }),
                    #[cfg(feature = "with-zeitgeist-runtime")]
                    spec if spec.is_zeitgeist() => runner.sync_run(|config| {
                        let params = crate::service::new_partial::<
                            zeitgeist_runtime::RuntimeApi,
                            ZeitgeistExecutor,
                        >(&config)?;
                        cmd.run(params.client)
                    }),
                    #[cfg(feature = "with-battery-station-runtime")]
                    _ => runner.sync_run(|config| {
                        let params = crate::service::new_partial::<
                            battery_station_runtime::RuntimeApi,
                            BatteryStationExecutor,
                        >(&config)?;
                        cmd.run(params.client)
                    }),
                    #[cfg(not(feature = "with-battery-station-runtime"))]
                    _ => panic!("{}", crate::BATTERY_STATION_RUNTIME_NOT_AVAILABLE),
                },
                BenchmarkCmd::Storage(cmd) => match chain_spec {
                    #[cfg(feature = "with-raumgeist-runtime")]
                    spec if spec.is_raumgeist() => runner.sync_run(|config| {
                        let params = crate::service::new_partial::<
                            raumgeist_runtime::RuntimeApi,
                            RaumgeistExecutor,
                        >(&config)?;

                        let db = params.backend.expose_db();
                        let storage = params.backend.expose_storage();

                        cmd.run(config, params.client, db, storage)
                    }),
                    #[cfg(feature = "with-zeitgeist-runtime")]
                    spec if spec.is_zeitgeist() => runner.sync_run(|config| {
                        let params = crate::service::new_partial::<
                            zeitgeist_runtime::RuntimeApi,
                            ZeitgeistExecutor,
                        >(&config)?;

                        let db = params.backend.expose_db();
                        let storage = params.backend.expose_storage();

                        cmd.run(config, params.client, db, storage)
                    }),
                    #[cfg(feature = "with-battery-station-runtime")]
                    _ => runner.sync_run(|config| {
                        let params = crate::service::new_partial::<
                            battery_station_runtime::RuntimeApi,
                            BatteryStationExecutor,
                        >(&config)?;

                        let db = params.backend.expose_db();
                        let storage = params.backend.expose_storage();

                        cmd.run(config, params.client, db, storage)
                    }),
                    #[cfg(not(feature = "with-battery-station-runtime"))]
                    _ => panic!("{}", crate::BATTERY_STATION_RUNTIME_NOT_AVAILABLE),
                },

                BenchmarkCmd::Overhead(cmd) => {
                    if cfg!(feature = "parachain") {
                        return Err("Overhead is only supported in standalone chain".into());
                    }
                    match chain_spec {
                        #[cfg(feature = "with-raumgeist-runtime")]
                        spec if spec.is_raumgeist() => runner.sync_run(|config| {
                            let params = crate::service::new_partial::<
                                raumgeist_runtime::RuntimeApi,
                                RaumgeistExecutor,
                            >(&config)?;

                            let ext_builder =
                                BenchmarkExtrinsicBuilder::new(params.client.clone(), id);
                            cmd.run(
                                config,
                                params.client,
                                inherent_benchmark_data()?,
                                Arc::new(ext_builder),
                            )
                        }),
                        #[cfg(feature = "with-zeitgeist-runtime")]
                        spec if spec.is_zeitgeist() => runner.sync_run(|config| {
                            let params = crate::service::new_partial::<
                                zeitgeist_runtime::RuntimeApi,
                                ZeitgeistExecutor,
                            >(&config)?;

                            let ext_builder =
                                BenchmarkExtrinsicBuilder::new(params.client.clone(), id);
                            cmd.run(
                                config,
                                params.client,
                                inherent_benchmark_data()?,
                                Arc::new(ext_builder),
                            )
                        }),
                        #[cfg(feature = "with-battery-station-runtime")]
                        _ => runner.sync_run(|config| {
                            let params = crate::service::new_partial::<
                                battery_station_runtime::RuntimeApi,
                                BatteryStationExecutor,
                            >(&config)?;

                            let ext_builder =
                                BenchmarkExtrinsicBuilder::new(params.client.clone(), id);
                            cmd.run(
                                config,
                                params.client,
                                inherent_benchmark_data()?,
                                Arc::new(ext_builder),
                            )
                        }),
                        #[cfg(not(feature = "with-battery-station-runtime"))]
                        _ => panic!("{}", crate::BATTERY_STATION_RUNTIME_NOT_AVAILABLE),
                    }
                }
            }
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

            let buf = match chain_spec {
                #[cfg(feature = "with-raumgeist-runtime")]
                spec if spec.is_raumgeist() => {
                    let block: raumgeist_runtime::Block =
                        cumulus_client_service::genesis::generate_genesis_block(
                            chain_spec,
                            state_version,
                        )?;
                    let raw_header = block.header().encode();

                    if params.raw {
                        raw_header
                    } else {
                        format!("0x{:?}", HexDisplay::from(&block.header().encode())).into_bytes()
                    }
                }
                #[cfg(feature = "with-zeitgeist-runtime")]
                spec if spec.is_zeitgeist() => {
                    let block: zeitgeist_runtime::Block =
                        cumulus_client_service::genesis::generate_genesis_block(
                            chain_spec,
                            state_version,
                        )?;
                    let raw_header = block.header().encode();

                    if params.raw {
                        raw_header
                    } else {
                        format!("0x{:?}", HexDisplay::from(&block.header().encode())).into_bytes()
                    }
                }
                #[cfg(feature = "with-battery-station-runtime")]
                _ => {
                    let block: battery_station_runtime::Block =
                        cumulus_client_service::genesis::generate_genesis_block(
                            chain_spec,
                            state_version,
                        )?;
                    let raw_header = block.header().encode();

                    if params.raw {
                        raw_header
                    } else {
                        format!("0x{:?}", HexDisplay::from(&block.header().encode())).into_bytes()
                    }
                }
                #[cfg(not(feature = "with-battery-station-runtime"))]
                _ => panic!("{}", crate::BATTERY_STATION_RUNTIME_NOT_AVAILABLE),
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
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|mut config| {
                let (client, backend, _, task_manager) = new_chain_ops(&mut config)?;

                Ok((cmd.run(client, backend, None), task_manager))
            })
        }
        #[cfg(feature = "try-runtime")]
        Some(Subcommand::TryRuntime(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            let chain_spec = &runner.config().chain_spec;

            match chain_spec {
                #[cfg(feature = "with-raumgeist-runtime")]
                spec if spec.is_raumgeist() => {
                    runner.async_run(|config| {
                        // we don't need any of the components of new_partial, just a runtime, or a task
                        // manager to do `async_run`.
                        let registry = config.prometheus_config.as_ref().map(|cfg| &cfg.registry);
                        let task_manager =
                            sc_service::TaskManager::new(config.tokio_handle.clone(), registry)
                                .map_err(|e| {
                                    sc_cli::Error::Service(sc_service::Error::Prometheus(e))
                                })?;
                        return Ok((
                            cmd.run::<raumgeist_runtime::Block, RaumgeistExecutor>(config),
                            task_manager,
                        ));
                    })
                }
                #[cfg(feature = "with-zeitgeist-runtime")]
                spec if spec.is_zeitgeist() => {
                    runner.async_run(|config| {
                        // we don't need any of the components of new_partial, just a runtime, or a task
                        // manager to do `async_run`.
                        let registry = config.prometheus_config.as_ref().map(|cfg| &cfg.registry);
                        let task_manager =
                            sc_service::TaskManager::new(config.tokio_handle.clone(), registry)
                                .map_err(|e| {
                                    sc_cli::Error::Service(sc_service::Error::Prometheus(e))
                                })?;
                        return Ok((
                            cmd.run::<zeitgeist_runtime::Block, ZeitgeistExecutor>(config),
                            task_manager,
                        ));
                    })
                }
                #[cfg(feature = "with-battery-station-runtime")]
                _ => runner.async_run(|config| {
                    let registry = config.prometheus_config.as_ref().map(|cfg| &cfg.registry);
                    let task_manager =
                        sc_service::TaskManager::new(config.tokio_handle.clone(), registry)
                            .map_err(|e| {
                                sc_cli::Error::Service(sc_service::Error::Prometheus(e))
                            })?;
                    return Ok((
                        cmd.run::<battery_station_runtime::Block, BatteryStationExecutor>(config),
                        task_manager,
                    ));
                }),
                #[cfg(not(feature = "with-battery-station-runtime"))]
                _ => Err("Invalid chain spec"),
            }
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

        match &parachain_config.chain_spec {
            #[cfg(feature = "with-raumgeist-runtime")]
            spec if spec.is_raumgeist() => new_full::<RaumgeistRuntimeApi, RaumgeistExecutor>(
                parachain_config,
                parachain_id,
                polkadot_config,
            )
            .await
            .map(|r| r.0)
            .map_err(Into::into),
            #[cfg(feature = "with-zeitgeist-runtime")]
            spec if spec.is_zeitgeist() => new_full::<ZeitgeistRuntimeApi, ZeitgeistExecutor>(
                parachain_config,
                parachain_id,
                polkadot_config,
            )
            .await
            .map(|r| r.0)
            .map_err(Into::into),
            #[cfg(feature = "with-battery-station-runtime")]
            _ => new_full::<BatteryStationRuntimeApi, BatteryStationExecutor>(
                parachain_config,
                parachain_id,
                polkadot_config,
            )
            .await
            .map(|r| r.0)
            .map_err(Into::into),
            #[cfg(not(feature = "with-battery-station-runtime"))]
            _ => panic!("{}", crate::BATTERY_STATION_RUNTIME_NOT_AVAILABLE),
        }
    })
}

#[cfg(not(feature = "parachain"))]
fn none_command(cli: &Cli) -> sc_cli::Result<()> {
    let runner = cli.create_runner(&cli.run)?;
    runner.run_node_until_exit(|config| async move {
        if let sc_cli::Role::Light = config.role {
            return Err("Light client not supported!".into());
        }

        match &config.chain_spec {
            #[cfg(feature = "with-raumgeist-runtime")]
            spec if spec.is_raumgeist() => {
                new_full::<RaumgeistRuntimeApi, RaumgeistExecutor>(config)
                    .map_err(sc_cli::Error::Service)
            }
            #[cfg(feature = "with-zeitgeist-runtime")]
            spec if spec.is_zeitgeist() => {
                new_full::<ZeitgeistRuntimeApi, ZeitgeistExecutor>(config)
                    .map_err(sc_cli::Error::Service)
            }
            #[cfg(feature = "with-battery-station-runtime")]
            _ => new_full::<BatteryStationRuntimeApi, BatteryStationExecutor>(config)
                .map_err(sc_cli::Error::Service),
            #[cfg(all(
                not(feature = "with-battery-station-runtime"),
                feature = "with-zeitgeist-runtime",
            ))]
            _ => new_full::<ZeitgeistRuntimeApi, ZeitgeistExecutor>(config)
                .map_err(sc_cli::Error::Service),
            #[cfg(all(
                not(feature = "with-battery-station-runtime"),
                not(feature = "with-zeitgeist-runtime"),
                feature = "with-raumgeist-runtime",
            ))]
            _ => new_full::<RaumgeistRuntimeApi, RaumgeistExecutor>(config)
                .map_err(sc_cli::Error::Service),
            #[cfg(all(
                not(feature = "with-battery-station-runtime"),
                not(feature = "with-raumgeist-runtime"),
                not(feature = "with-zeitgeist-runtime"),
            ))]
            _ => panic!("{}", crate::BATTERY_STATION_RUNTIME_NOT_AVAILABLE),
        }
    })
}
