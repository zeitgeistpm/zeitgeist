use crate::chain_spec::{generic_genesis, AdditionalChainSpec, ChainSpec, TELEMETRY_URL};
use hex_literal::hex;
use jsonrpc_core::serde_json::Map;
use sc_service::{config::TelemetryEndpoints, ChainType};
#[cfg(not(feature = "parachain"))]
use sp_core::crypto::UncheckedInto;
use zeitgeist_primitives::{constants::BASE, types::AccountId};

pub fn battery_park_config(
    #[cfg(feature = "parachain")] id: cumulus_primitives_core::ParaId,
) -> Result<ChainSpec, String> {
    let wasm_binary = zeitgeist_runtime::WASM_BINARY
        .ok_or("Development wasm binary not available".to_string())?;

    let mut properties = Map::new();
    properties.insert("tokenSymbol".into(), "ZBP".into());
    properties.insert("tokenDecimals".into(), 10.into());

    Ok(ChainSpec::from_genesis(
        "Zeitgeist Battery Park",
        "battery_park",
        ChainType::Live,
        move || {
            generic_genesis(
                #[cfg(feature = "parachain")]
                AdditionalChainSpec {
                    inflation_info: crate::chain_spec::DEFAULT_COLLATOR_INFLATION_INFO,
                    stakers: vec![],
                    parachain_id: id,
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
                ],
                10_000 * BASE,
                hex!["2a6c61a907556e4c673880b5767dd4be08339ee7f2a58d5137d0c19ca9570a5c"].into(),
                wasm_binary,
            )
        },
        vec![
            "/ip4/139.162.171.58/tcp/30333/p2p/12D3KooWPvu5rpH2FNYnAmiQ8X8XqkMiuSFTjH2jwMCSjoam7RGQ".parse().map_err(|_| "invalid bootnoode id")?
        ],
        TelemetryEndpoints::new(vec![(TELEMETRY_URL.into(), 0)]).ok(),
        Some("battery_park"),
        Some(properties),
        #[cfg(feature = "parachain")]
        crate::chain_spec::Extensions {
            relay_chain: "rococo-battery-park".into(),
            para_id: id.into(),
        },
        #[cfg(not(feature = "parachain"))]
        Default::default(),
    ))
}
