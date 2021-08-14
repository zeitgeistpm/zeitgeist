use crate::chain_spec::{
    additional_chain_spec_staging, endowed_accounts_staging, generic_genesis, root_key_staging,
    telemetry_endpoints, token_properties, zeitgeist_wasm, ChainSpec,
};
use sc_service::ChainType;
use zeitgeist_primitives::constants::BASE;

pub fn battery_park_staging_config(
    #[cfg(feature = "parachain")] parachain_id: cumulus_primitives_core::ParaId,
) -> Result<ChainSpec, String> {
    let zeitgeist_wasm = zeitgeist_wasm()?;

    Ok(ChainSpec::from_genesis(
        "Battery Park Staging",
        "battery_park_staging",
        ChainType::Live,
        move || {
            generic_genesis(
                additional_chain_spec_staging(
                    #[cfg(feature = "parachain")]
                    parachain_id,
                ),
                endowed_accounts_staging(),
                10_000 * BASE,
                root_key_staging(),
                zeitgeist_wasm,
            )
        },
        vec![],
        telemetry_endpoints(),
        Some("battery_park_staging"),
        Some(token_properties()),
        #[cfg(feature = "parachain")]
        crate::chain_spec::Extensions {
            relay_chain: "rococo".into(),
            parachain_id: parachain_id.into(),
        },
        #[cfg(not(feature = "parachain"))]
        Default::default(),
    ))
}
