#[cfg(feature = "pallet-sudo")]
use crate::chain_spec::root_key_staging_testnet;
use crate::chain_spec::{
    additional_chain_spec_staging_testnet, endowed_accounts_staging_testnet, generic_genesis,
    telemetry_endpoints, token_properties, zeitgeist_wasm, ChainSpec,
};
use sc_service::ChainType;

pub fn battery_station_staging_config(
    #[cfg(feature = "parachain")] parachain_id: cumulus_primitives_core::ParaId,
) -> Result<ChainSpec, String> {
    let zeitgeist_wasm = zeitgeist_wasm()?;

    Ok(ChainSpec::from_genesis(
        "Battery Station Staging",
        "battery_station_staging",
        ChainType::Live,
        move || {
            generic_genesis(
                additional_chain_spec_staging_testnet(
                    #[cfg(feature = "parachain")]
                    parachain_id,
                ),
                endowed_accounts_staging_testnet(),
                #[cfg(feature = "pallet-sudo")]
                root_key_staging_testnet(),
                zeitgeist_wasm,
            )
        },
        vec![],
        telemetry_endpoints(),
        Some("battery_station"),
        None,
        Some(token_properties("ZBS")),
        #[cfg(feature = "parachain")]
        crate::chain_spec::Extensions {
            relay_chain: "rococo".into(),
            parachain_id: parachain_id.into(),
        },
        #[cfg(not(feature = "parachain"))]
        Default::default(),
    ))
}
