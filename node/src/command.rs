use crate::cli::{Cli, Subcommand};
use sc_cli::SubstrateCli;
use sc_service::PartialComponents;
use zeitgeist_runtime::Block;
#[cfg(feature = "parachain")]
use {
    parity_scale_codec::Encode, sp_core::hexdisplay::HexDisplay,
    sp_runtime::traits::Block as BlockT, std::io::Write,
};

pub fn run() -> sc_cli::Result<()> {
    let cli = <Cli as SubstrateCli>::from_args();

    match &cli.subcommand {
        #[cfg(not(feature = "parachain"))]
        Some(Subcommand::Benchmark(cmd)) => {
            if cfg!(feature = "runtime-benchmarks") {
                let runner = cli.create_runner(cmd)?;

                runner.sync_run(|config| cmd.run::<Block, crate::service::Executor>(config))
            } else {
                Err(
          "Benchmarking wasn't enabled when building the node. You can enable it with `--features \
           runtime-benchmarks`."
            .into(),
        )
            }
        }
        Some(Subcommand::BuildSpec(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
        }
        Some(Subcommand::CheckBlock(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let PartialComponents {
                    client,
                    task_manager,
                    import_queue,
                    ..
                } = crate::service::new_partial(&config)?;
                Ok((cmd.run(client, import_queue), task_manager))
            })
        }
        Some(Subcommand::ExportBlocks(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let PartialComponents {
                    client,
                    task_manager,
                    ..
                } = crate::service::new_partial(&config)?;
                Ok((cmd.run(client, config.database), task_manager))
            })
        }
        #[cfg(feature = "parachain")]
        Some(Subcommand::ExportGenesisState(params)) => {
            let mut builder = sc_cli::LoggerBuilder::new("");
            builder.with_profiling(sc_tracing::TracingReceiver::Log, "");
            let _ = builder.init();

            let block: Block =
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
                extract_genesis_wasm(&cli.load_spec(&params.chain.clone().unwrap_or_default())?)?;
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
                let PartialComponents {
                    client,
                    task_manager,
                    ..
                } = crate::service::new_partial(&config)?;
                Ok((cmd.run(client, config.chain_spec), task_manager))
            })
        }
        Some(Subcommand::ImportBlocks(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.async_run(|config| {
                let PartialComponents {
                    client,
                    task_manager,
                    import_queue,
                    ..
                } = crate::service::new_partial(&config)?;
                Ok((cmd.run(client, import_queue), task_manager))
            })
        }
        #[cfg(not(feature = "parachain"))]
        Some(Subcommand::Key(cmd)) => cmd.run(&cli),
        #[cfg(feature = "parachain")]
        Some(Subcommand::PurgeChain(cmd)) => {
            let runner = cli.create_runner(cmd)?;

            runner.sync_run(|config| {
                let polkadot_cli = crate::cli::RelayChainCli::new(
                    &config,
                    [crate::cli::RelayChainCli::executable_name().to_string()]
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
                let PartialComponents {
                    client,
                    task_manager,
                    backend,
                    ..
                } = crate::service::new_partial(&config)?;
                Ok((cmd.run(client, backend), task_manager))
            })
        }
        None => none_command(&cli),
    }
}

#[cfg(feature = "parachain")]
fn extract_genesis_wasm(chain_spec: &Box<dyn sc_service::ChainSpec>) -> sc_cli::Result<Vec<u8>> {
    let mut storage = chain_spec.build_storage()?;

    storage
        .top
        .remove(sp_core::storage::well_known_keys::CODE)
        .ok_or_else(|| "Could not find wasm file in genesis state!".into())
}

#[cfg(feature = "parachain")]
fn none_command(cli: &Cli) -> sc_cli::Result<()> {
    let runner = cli.create_runner(&*cli.run)?;

    runner.run_node_until_exit(|config| async move {
        let key = sp_core::Pair::generate().0;

        let para_id =
            crate::chain_spec::Extensions::try_get(&*config.chain_spec).map(|e| e.para_id);

        let polkadot_cli = crate::cli::RelayChainCli::new(
            &config,
            [crate::cli::RelayChainCli::executable_name().to_string()]
                .iter()
                .chain(cli.relaychain_args.iter()),
        );

        let id = cumulus_primitives_core::ParaId::from(
            cli.run
                .parachain_id
                .or(para_id)
                .unwrap_or(crate::DEFAULT_PARACHAIN_ID),
        );

        let parachain_account = polkadot_parachain::primitives::AccountIdConversion::<
            polkadot_primitives::v0::AccountId,
        >::into_account(&id);

        let block: Block =
            cumulus_client_service::genesis::generate_genesis_block(&config.chain_spec)
                .map_err(|e| format!("{:?}", e))?;
        let genesis_state = format!("0x{:?}", HexDisplay::from(&block.header().encode()));

        let task_executor = config.task_executor.clone();
        let polkadot_config =
            SubstrateCli::create_configuration(&polkadot_cli, &polkadot_cli, task_executor)
                .map_err(|err| format!("Relay chain argument error: {}", err))?;
        let collator = cli.run.base.validator || cli.collator;

        log::info!("Parachain id: {:?}", id);
        log::info!("Parachain Account: {}", parachain_account);
        log::info!("Parachain genesis state: {}", genesis_state);
        log::info!("Is collating: {}", if collator { "yes" } else { "no" });

        crate::service::new_full(config, key, polkadot_config, id, collator)
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
            sc_cli::Role::Light => crate::service::new_light(config),
            _ => crate::service::new_full(config),
        }
        .map_err(sc_cli::Error::Service)
    })
}
