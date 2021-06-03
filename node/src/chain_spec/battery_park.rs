use crate::chain_spec::{generic_genesis, AdditionalChainSpec, ChainSpec, TELEMETRY_URL};
use hex_literal::hex;
use jsonrpc_core::serde_json::Map;
use sc_service::{config::TelemetryEndpoints, ChainType};
#[cfg(not(feature = "parachain"))]
use sp_core::crypto::UncheckedInto;
use zeitgeist_primitives::constants::BASE;

pub fn battery_park_staging_config(
    #[cfg(feature = "parachain")] parachain_id: cumulus_primitives_core::ParaId,
) -> Result<ChainSpec, String> {
    let wasm_binary = zeitgeist_runtime::WASM_BINARY
        .ok_or_else(|| "Development wasm binary not available".to_string())?;

    let mut properties = Map::new();
    properties.insert("tokenSymbol".into(), "ZBP".into());
    properties.insert("tokenDecimals".into(), 10.into());

    Ok(ChainSpec::from_genesis(
        "Zeitgeist Battery Park Staging",
        "battery_park_staging",
        ChainType::Live,
        move || {
            generic_genesis(
                #[cfg(feature = "parachain")]
                AdditionalChainSpec {
                    candidates: vec![],
                    inflation_info: crate::chain_spec::DEFAULT_COLLATOR_INFLATION_INFO,
                    nominations: vec![],
                    parachain_id,
                },
                #[cfg(not(feature = "parachain"))]
                AdditionalChainSpec {
                    initial_authorities: vec![(
                        // 5FCSJzvmeUW1hBo3ASnLzSxpUdn5QUDt1Eqobj1meiQB7mLu
                        hex!["8a9a54bdf73fb4a757f5ab81fabe2f173922fdb92bb8b6e8bedf8b17fa38f500"]
                            .unchecked_into(),
                        // 5HGProUwcyCDMJDxjBBKbv8u7ehr5uoTBS3bckYHPcZMTifW
                        hex!["e61786c6426b55a034f9c4b78dc57d4183927cef8e64b2e496225ed6fca41758"]
                            .unchecked_into(),
                    )],
                },
                vec![
                    // 5D2L4ghyiYE8p2z7VNJo9JYwRuc8uzPWtMBqdVyvjRcsnw4P
                    hex!["2a6c61a907556e4c673880b5767dd4be08339ee7f2a58d5137d0c19ca9570a5c"].into(),
                    // 5EeeZVU4SiPG6ZRY7o8aDcav2p2mZMdu3ZLzbREWuHktYdhX
                    hex!["725bb6fd13d52b3d6830e5a9faed1f6499ca0f5e8aa285df09490646e71e831b"].into(),
                ],
                10_000 * BASE,
                hex!["2a6c61a907556e4c673880b5767dd4be08339ee7f2a58d5137d0c19ca9570a5c"].into(),
                wasm_binary,
            )
        },
        vec![],
        TelemetryEndpoints::new(vec![(TELEMETRY_URL.into(), 0)]).ok(),
        Some("battery_park_staging"),
        Some(properties),
        #[cfg(feature = "parachain")]
        crate::chain_spec::Extensions {
            relay_chain: "rococo".into(),
            parachain_id: parachain_id.into(),
        },
        #[cfg(not(feature = "parachain"))]
        Default::default(),
    ))
}
