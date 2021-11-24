use crate::chain_spec::{
    additional_chain_spec_staging, endowed_accounts_staging, generic_genesis, root_key_staging,
    telemetry_endpoints, token_properties, zeitgeist_wasm, ChainSpec,
};
use sc_service::ChainType;
use zeitgeist_primitives::{constants::BASE, types::Balance};

const INITIAL_BALANCE: Balance = 10_000 * BASE;

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
                additional_chain_spec_staging(
                    #[cfg(feature = "parachain")]
                    parachain_id,
                ),
                endowed_accounts_staging(),
                INITIAL_BALANCE,
                root_key_staging(),
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
