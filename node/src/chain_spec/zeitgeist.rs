#[cfg(not(feature = "without-sudo"))]
use crate::chain_spec::root_key_staging_mainnet;
use crate::chain_spec::{
    additional_chain_spec_staging_mainnet, endowed_accounts_staging_mainnet, generic_genesis,
    telemetry_endpoints, token_properties, zeitgeist_wasm, ChainSpec,
};
use sc_service::ChainType;

pub fn zeitgeist_staging_config(
    #[cfg(feature = "parachain")] parachain_id: cumulus_primitives_core::ParaId,
) -> Result<ChainSpec, String> {
    let zeitgeist_wasm = zeitgeist_wasm()?;

    Ok(ChainSpec::from_genesis(
        "Zeitgeist Staging",
        "zeitgeist_staging",
        ChainType::Live,
        move || {
            generic_genesis(
                additional_chain_spec_staging_mainnet(
                    #[cfg(feature = "parachain")]
                    parachain_id,
                ),
                endowed_accounts_staging_mainnet(),
                #[cfg(not(feature = "without-sudo"))]
                root_key_staging_mainnet(),
                zeitgeist_wasm,
            )
        },
        vec![],
        telemetry_endpoints(),
        Some("zeitgeist"),
        None,
        Some(token_properties("ZTG")),
        #[cfg(feature = "parachain")]
        crate::chain_spec::Extensions {
            relay_chain: "kusama".into(),
            parachain_id: parachain_id.into(),
        },
        #[cfg(not(feature = "parachain"))]
        Default::default(),
    ))
}
