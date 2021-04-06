#[cfg(feature = "parachain")]
mod cli_parachain;
#[cfg(not(feature = "parachain"))]
mod cli_standalone;

#[cfg(feature = "parachain")]
pub use cli_parachain::{Cli, RelayChainCli, Subcommand};
#[cfg(not(feature = "parachain"))]
pub use cli_standalone::{Cli, Subcommand};

pub fn load_spec(
    id: &str,
    #[cfg(feature = "parachain")] para_id: cumulus_primitives_core::ParaId,
) -> Result<Box<dyn sc_service::ChainSpec>, String> {
    Ok(match id {
        "dev" => Box::new(crate::chain_spec::dev_config(
            #[cfg(feature = "parachain")]
            para_id,
        )?),
        "" | "local" => Box::new(crate::chain_spec::local_testnet_config(
            #[cfg(feature = "parachain")]
            para_id,
        )?),
        "battery_park" => Box::new(crate::chain_spec::battery_park_config(
            #[cfg(feature = "parachain")]
            para_id,
        )?),
        path => Box::new(crate::chain_spec::ChainSpec::from_json_file(
            std::path::PathBuf::from(path),
        )?),
    })
}
