// Copyright 2022-2025 Forecasting Technologies LTD.
// Copyright 2021-2022 Zeitgeist PM LLC.
// Copyright 2019-2022 PureStake Inc.
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
    benchmarking::{inherent_benchmark_data, RemarksExtrinsicBuilder, TransferKeepAliveBuilder},
    cli::{Cli, Subcommand},
    service::{new_chain_ops, new_full, IdentifyVariant},
};
#[cfg(feature = "with-battery-station-runtime")]
use battery_station_runtime::{
    ExistentialDeposit as BatteryStationED, RuntimeApi as BatteryStationRuntimeApi,
};
use frame_benchmarking_cli::{BenchmarkCmd, ExtrinsicFactory, SUBSTRATE_REFERENCE_HARDWARE};
use sc_cli::SubstrateCli;
use sp_keyring::Sr25519Keyring;
#[cfg(feature = "parachain")]
use {
    sc_cli::ChainSpec,
    sp_core::Encode,
    sp_runtime::{
        traits::{Block as BlockT, Hash as HashT, Header as HeaderT, Zero},
        StateVersion,
    },
};

#[cfg(feature = "with-zeitgeist-runtime")]
use zeitgeist_runtime::{ExistentialDeposit as ZeitgeistED, RuntimeApi as ZeitgeistRuntimeApi};
#[cfg(feature = "parachain")]
use {
    sc_client_api::client::BlockBackend, sp_core::hexdisplay::HexDisplay,
    sp_runtime::traits::AccountIdConversion, std::io::Write,
};

pub fn run() -> sc_cli::Result<()> {
    let mut cli = <Cli as SubstrateCli>::from_args();

    // Set default chain on parachain to zeitgeist and on standalone to dev
    #[cfg(feature = "parachain")]
    if cli.run.base.shared_params.chain.is_none() {
        cli.run.base.shared_params.chain = Some("zeitgeist".to_string());
    }
    #[cfg(not(feature = "parachain"))]
    if cli.run.shared_params.chain.is_none() {
        cli.run.shared_params.dev = true;
    }

    match &cli.subcommand {
        Some(Subcommand::Benchmark(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            let chain_spec = &runner.config().chain_spec;

            match cmd {
                // This switch needs to be in the client, since the client decides
                // which sub-commands it wants to support.
                BenchmarkCmd::Pallet(cmd) => {
                    if cfg!(feature = "runtime-benchmarks") {
                        match chain_spec {
                            #[cfg(feature = "with-zeitgeist-runtime")]
                            spec if spec.is_zeitgeist() => runner.sync_run(|config| {
                                cmd.run_with_spec::<sp_runtime::traits::HashingFor<zeitgeist_runtime::Block>, ()>(Some(config.chain_spec))
                            }),
                            #[cfg(feature = "with-battery-station-runtime")]
                            _ => runner.sync_run(|config| {
                                cmd.run_with_spec::<sp_runtime::traits::HashingFor<battery_station_runtime::Block>, ()>(Some(config.chain_spec))
                            }),
                            #[cfg(not(feature = "with-battery-station-runtime"))]
                            _ => panic!("{}", crate::BATTERY_STATION_RUNTIME_NOT_AVAILABLE),
                        }
                    } else {
                        Err("Runtime benchmarking wasn't enabled when building the node. \
                            You can enable it with `--features runtime-benchmarks`."
                            .into())
                    }
                }
                BenchmarkCmd::Block(cmd) => match chain_spec {
                    #[cfg(feature = "with-zeitgeist-runtime")]
                    spec if spec.is_zeitgeist() => runner.sync_run(|config| {
                        let params = crate::service::new_partial::<
                            zeitgeist_runtime::RuntimeApi,
                        >(&config)?;
                        cmd.run(params.client)
                    }),
                    #[cfg(feature = "with-battery-station-runtime")]
                    _ => runner.sync_run(|config| {
                        let params = crate::service::new_partial::<
                            battery_station_runtime::RuntimeApi,
                        >(&config)?;
                        cmd.run(params.client)
                    }),
                    #[cfg(not(feature = "with-battery-station-runtime"))]
                    _ => panic!("{}", crate::BATTERY_STATION_RUNTIME_NOT_AVAILABLE),
                },
                // The hardware requirement is currently less than Substrate's hardware
                // requirement for parachain builds. Since the parachain has to sync
                // the relay chain and adhere to tight deadlines, it requires at least
                // the hardware specs specified by the relay chain.
                BenchmarkCmd::Machine(cmd) => {
                    runner.sync_run(|config| cmd.run(&config, SUBSTRATE_REFERENCE_HARDWARE.clone()))
                }
                #[cfg(not(feature = "runtime-benchmarks"))]
                BenchmarkCmd::Storage(_) => Err("Storage benchmarking can be enabled with \
                                                 `--features runtime-benchmarks`."
                    .into()),
                #[cfg(feature = "runtime-benchmarks")]
                BenchmarkCmd::Storage(cmd) => match chain_spec {
                    #[cfg(feature = "with-zeitgeist-runtime")]
                    spec if spec.is_zeitgeist() => runner.sync_run(|config| {
                        let params = crate::service::new_partial::<
                            zeitgeist_runtime::RuntimeApi,
                        >(&config)?;

                        let db = params.backend.expose_db();
                        let storage = params.backend.expose_storage();

                        cmd.run(config, params.client, db, storage)
                    }),
                    #[cfg(feature = "with-battery-station-runtime")]
                    _ => runner.sync_run(|config| {
                        let params = crate::service::new_partial::<
                            battery_station_runtime::RuntimeApi,
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
                        #[cfg(feature = "with-zeitgeist-runtime")]
                        spec if spec.is_zeitgeist() => runner.sync_run(|config| {
                            let params = crate::service::new_partial::<
                                zeitgeist_runtime::RuntimeApi,
                            >(&config)?;

                            let ext_builder =
                                RemarksExtrinsicBuilder::new(params.client.clone(), true);
                            cmd.run(
                                config,
                                params.client,
                                inherent_benchmark_data()?,
                                Vec::new(),
                                &ext_builder,
                            )
                        }),
                        #[cfg(feature = "with-battery-station-runtime")]
                        _ => runner.sync_run(|config| {
                            let params = crate::service::new_partial::<
                                battery_station_runtime::RuntimeApi,
                            >(&config)?;

                            let ext_builder =
                                RemarksExtrinsicBuilder::new(params.client.clone(), false);
                            cmd.run(
                                config,
                                params.client,
                                inherent_benchmark_data()?,
                                Vec::new(),
                                &ext_builder,
                            )
                        }),
                        #[cfg(not(feature = "with-battery-station-runtime"))]
                        _ => panic!("{}", crate::BATTERY_STATION_RUNTIME_NOT_AVAILABLE),
                    }
                }

                BenchmarkCmd::Extrinsic(cmd) => {
                    if cfg!(feature = "parachain") {
                        return Err("Extrinsic is only supported in standalone chain".into());
                    }
                    match chain_spec {
                        #[cfg(feature = "with-zeitgeist-runtime")]
                        spec if spec.is_zeitgeist() => runner.sync_run(|config| {
                            let params = crate::service::new_partial::<
                                zeitgeist_runtime::RuntimeApi,
                            >(&config)?;
                            // Register the *Remark* and *TKA* builders.
                            let ext_factory = ExtrinsicFactory(vec![
                                Box::new(RemarksExtrinsicBuilder::new(params.client.clone(), true)),
                                Box::new(TransferKeepAliveBuilder::new(
                                    params.client.clone(),
                                    Sr25519Keyring::Alice.to_account_id(),
                                    ZeitgeistED::get(),
                                    true,
                                )),
                            ]);
                            cmd.run(
                                params.client,
                                inherent_benchmark_data()?,
                                Vec::new(),
                                &ext_factory,
                            )
                        }),
                        #[cfg(feature = "with-battery-station-runtime")]
                        _ => runner.sync_run(|config| {
                            let params = crate::service::new_partial::<
                                battery_station_runtime::RuntimeApi,
                            >(&config)?;
                            // Register the *Remark* and *TKA* builders.
                            let ext_factory = ExtrinsicFactory(vec![
                                Box::new(RemarksExtrinsicBuilder::new(
                                    params.client.clone(),
                                    false,
                                )),
                                Box::new(TransferKeepAliveBuilder::new(
                                    params.client.clone(),
                                    Sr25519Keyring::Alice.to_account_id(),
                                    BatteryStationED::get(),
                                    false,
                                )),
                            ]);
                            cmd.run(
                                params.client,
                                inherent_benchmark_data()?,
                                Vec::new(),
                                &ext_factory,
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
        Some(Subcommand::ChainInfo(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            let chain_spec = &runner.config().chain_spec;

            match chain_spec {
                #[cfg(feature = "with-zeitgeist-runtime")]
                spec if spec.is_zeitgeist() => {
                    runner.sync_run(|config| cmd.run::<zeitgeist_runtime::Block>(&config))
                }
                #[cfg(feature = "with-battery-station-runtime")]
                _ => runner.sync_run(|config| cmd.run::<battery_station_runtime::Block>(&config)),
            }
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
            use sp_runtime::generic::BlockId;

            let runner = cli.create_runner(cmd)?;

            runner.sync_run(|mut config| {
                let (client, _, _, _) = new_chain_ops(&mut config)?;
                let block_number: BlockId<zeitgeist_runtime::Block> = cmd.input.parse()?;
                let block_hash = match block_number {
                    BlockId::Hash(hash) => hash,
                    BlockId::Number(number) => match client.block_hash(number) {
                        Ok(Some(hash)) => hash,
                        Ok(None) => return Err("Unknown block".into()),
                        Err(e) => return Err(format!("Error reading block: {:?}", e).into()),
                    },
                };

                match client.block(block_hash) {
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
        Some(Subcommand::ExportGenesisHead(params)) => {
            let mut builder = sc_cli::LoggerBuilder::new("");
            builder.with_profiling(sc_tracing::TracingReceiver::Log, "");
            let _ = builder.init();
            let chain_spec =
                &crate::cli::load_spec(&params.shared_params.chain.clone().unwrap_or_default())?;
            let state_version = Cli::runtime_version(chain_spec).state_version();

            let buf = match chain_spec {
                #[cfg(feature = "with-zeitgeist-runtime")]
                spec if spec.is_zeitgeist() => {
                    let block: zeitgeist_runtime::Block =
                        generate_genesis_block(&**chain_spec, state_version)?;
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
                        generate_genesis_block(&**chain_spec, state_version)?;
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
                std::fs::write(output, &buf)?;
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

            let raw_wasm_blob = extract_genesis_wasm(
                cli.load_spec(&params.shared_params.chain.clone().unwrap_or_default())?,
            )?;
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
        Some(Subcommand::TryRuntime) => Err("The `try-runtime` subcommand has been migrated to a \
            standalone CLI (https://github.com/paritytech/try-runtime-cli). It is no longer \
            being maintained here and will be removed entirely some time after January 2024. \
            Please remove this subcommand from your runtime and use the standalone CLI."
            .into()),
        None => none_command(cli),
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
fn none_command(cli: Cli) -> sc_cli::Result<()> {
    let runner = cli.create_runner(&cli.run.normalize())?;

    runner.run_node_until_exit(|parachain_config| async move {
        let chain_spec = &parachain_config.chain_spec;
        let parachain_id_extension =
            crate::chain_spec::Extensions::try_get(&**chain_spec).map(|e| e.parachain_id);
        let polkadot_cli = crate::cli::RelayChainCli::new(
            &parachain_config,
            [crate::cli::RelayChainCli::executable_name()].iter().chain(cli.relaychain_args.iter()),
        );
        let collator_options = cli.run.collator_options();
        let parachain_id = cumulus_primitives_core::ParaId::from(
            cli.parachain_id.or(parachain_id_extension).unwrap_or(super::POLKADOT_PARACHAIN_ID),
        );
        let parachain_account =
            AccountIdConversion::<polkadot_primitives::AccountId>::into_account_truncating(
                &parachain_id,
            );
        let state_version = Cli::runtime_version(chain_spec).state_version();
        let block: zeitgeist_runtime::Block =
            generate_genesis_block(&**chain_spec, state_version).map_err(|e| format!("{:?}", e))?;
        let genesis_state = format!("0x{:?}", HexDisplay::from(&block.header().encode()));

        let tokio_handle = parachain_config.tokio_handle.clone();
        let polkadot_config =
            SubstrateCli::create_configuration(&polkadot_cli, &polkadot_cli, tokio_handle)
                .map_err(|err| format!("Relay chain argument error: {}", err))?;

        log::info!("Parachain id: {:?}", parachain_id);
        log::info!("Parachain Account: {}", parachain_account);
        log::info!("Parachain genesis state: {}", genesis_state);

        let hwbench = (!cli.no_hardware_benchmarks)
            .then_some(parachain_config.database.path().map(|database_path| {
                let _ = std::fs::create_dir_all(database_path);
                sc_sysinfo::gather_hwbench(Some(database_path), &SUBSTRATE_REFERENCE_HARDWARE)
            }))
            .flatten();

        log::info!(
            "Is collating: {}",
            if parachain_config.role.is_authority() { "yes" } else { "no" }
        );

        if !cli.run.relay_chain_rpc_urls.is_empty() && !cli.relaychain_args.is_empty() {
            log::warn!(
                "Detected relay chain node arguments together with --relay-chain-rpc-url. This \
                 command starts a minimal Polkadot node that only uses a network-related subset \
                 of all relay chain CLI options."
            );
        }

        match &parachain_config.chain_spec {
            #[cfg(feature = "with-zeitgeist-runtime")]
            spec if spec.is_zeitgeist() => new_full::<ZeitgeistRuntimeApi>(
                parachain_config,
                parachain_id,
                polkadot_config,
                false,
                cli.block_authoring_duration,
                hwbench,
                collator_options,
            )
            .await
            .map(|r| r.0)
            .map_err(Into::into),
            #[cfg(feature = "with-battery-station-runtime")]
            _ => new_full::<BatteryStationRuntimeApi>(
                parachain_config,
                parachain_id,
                polkadot_config,
                false,
                cli.block_authoring_duration,
                hwbench,
                collator_options,
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
fn none_command(cli: Cli) -> sc_cli::Result<()> {
    let runner = cli.create_runner(&cli.run)?;
    runner.run_node_until_exit(|config| async move {
        match &config.chain_spec {
            #[cfg(feature = "with-zeitgeist-runtime")]
            spec if spec.is_zeitgeist() => {
                new_full::<ZeitgeistRuntimeApi, sc_network::NetworkWorker<_, _>>(config, cli)
                    .map_err(sc_cli::Error::Service)
            }
            #[cfg(feature = "with-battery-station-runtime")]
            _ => new_full::<BatteryStationRuntimeApi, sc_network::NetworkWorker<_, _>>(config, cli)
                .map_err(sc_cli::Error::Service),
            #[cfg(all(
                not(feature = "with-battery-station-runtime"),
                feature = "with-zeitgeist-runtime"
            ))]
            _ => {
                new_full::<ZeitgeistRuntimeApi, ZeitgeistExecutor, sc_network::NetworkWorker<_, _>>(
                    config, cli,
                )
                .map_err(sc_cli::Error::Service)
            }
            #[cfg(all(
                not(feature = "with-battery-station-runtime"),
                not(feature = "with-zeitgeist-runtime")
            ))]
            _ => panic!("{}", crate::BATTERY_STATION_RUNTIME_NOT_AVAILABLE),
        }
    })
}

/// Generate the genesis block from a given ChainSpec.
#[cfg(feature = "parachain")]
pub fn generate_genesis_block<Block: BlockT>(
    chain_spec: &dyn ChainSpec,
    genesis_state_version: StateVersion,
) -> std::result::Result<Block, String> {
    let storage = chain_spec.build_storage()?;

    let child_roots = storage.children_default.iter().map(|(sk, child_content)| {
        let state_root = <<<Block as BlockT>::Header as HeaderT>::Hashing as HashT>::trie_root(
            child_content.data.clone().into_iter().collect(),
            genesis_state_version,
        );
        (sk.clone(), state_root.encode())
    });
    let state_root = <<<Block as BlockT>::Header as HeaderT>::Hashing as HashT>::trie_root(
        storage.top.clone().into_iter().chain(child_roots).collect(),
        genesis_state_version,
    );

    let extrinsics_root = <<<Block as BlockT>::Header as HeaderT>::Hashing as HashT>::trie_root(
        Vec::new(),
        genesis_state_version,
    );

    Ok(Block::new(
        <<Block as BlockT>::Header as HeaderT>::new(
            Zero::zero(),
            extrinsics_root,
            state_root,
            Default::default(),
            Default::default(),
        ),
        Default::default(),
    ))
}
