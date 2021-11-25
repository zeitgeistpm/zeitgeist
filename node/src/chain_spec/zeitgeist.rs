use crate::chain_spec::{
    additional_chain_spec_staging_mainnet, endowed_accounts_staging_mainnet, generic_genesis,
    root_key_staging_mainnet, telemetry_endpoints, token_properties, zeitgeist_wasm, ChainSpec,
};
use sc_service::ChainType;

// TODO: swap *_testnet with *_mainnet
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
<<<<<<< HEAD
                endowed_accounts_staging_mainnet(),
                root_key_staging_mainnet(),
=======
                endowed_accounts_staging_testnet(),
                root_key_staging_testnet(),
>>>>>>> dfaa692 (Modify generic_genesis() to allow more control over endowed accounts)
                zeitgeist_wasm,
            )
        },
        vec![],
        telemetry_endpoints(),
        Some("zeitgeist"),
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
