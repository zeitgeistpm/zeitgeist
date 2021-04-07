use crate::chain_spec::{ChainSpec, TELEMETRY_URL};
use hex_literal::hex;
use jsonrpc_core::serde_json::Map;
use sc_service::{config::TelemetryEndpoints, ChainType};
#[cfg(not(feature = "parachain"))]
use sp_core::crypto::UncheckedInto;
use zeitgeist_primitives::{AccountId, BASE};

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
            battery_park_genesis(
                vec![
                    // 5D2L4ghyiYE8p2z7VNJo9JYwRuc8uzPWtMBqdVyvjRcsnw4P
                    hex!["2a6c61a907556e4c673880b5767dd4be08339ee7f2a58d5137d0c19ca9570a5c"].into(),
                ],
                #[cfg(feature = "parachain")]
                id,
                #[cfg(not(feature = "parachain"))]
                vec![(
                    // 5FCSJzvmeUW1hBo3ASnLzSxpUdn5QUDt1Eqobj1meiQB7mLu
                    hex!["8a9a54bdf73fb4a757f5ab81fabe2f173922fdb92bb8b6e8bedf8b17fa38f500"]
                        .unchecked_into(),
                    // 5HGProUwcyCDMJDxjBBKbv8u7ehr5uoTBS3bckYHPcZMTifW
                    hex!["e61786c6426b55a034f9c4b78dc57d4183927cef8e64b2e496225ed6fca41758"]
                        .unchecked_into(),
                )],
                hex!["2a6c61a907556e4c673880b5767dd4be08339ee7f2a58d5137d0c19ca9570a5c"].into(),
                wasm_binary,
            )
        },
        vec![],
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

fn battery_park_genesis(
    endowed_accounts: Vec<AccountId>,
    #[cfg(feature = "parachain")] id: cumulus_primitives_core::ParaId,
    #[cfg(not(feature = "parachain"))] initial_authorities: Vec<(
        sp_consensus_aura::sr25519::AuthorityId,
        sp_finality_grandpa::AuthorityId,
    )>,
    root_key: AccountId,
    wasm_binary: &[u8],
) -> zeitgeist_runtime::GenesisConfig {
    let initial_balance = 10_000 * BASE;

    zeitgeist_runtime::GenesisConfig {
        frame_system: zeitgeist_runtime::SystemConfig {
            // Add Wasm runtime to storage.
            code: wasm_binary.to_vec(),
            changes_trie_config: Default::default(),
        },
        orml_tokens: zeitgeist_runtime::TokensConfig::default(),
        #[cfg(not(feature = "parachain"))]
        pallet_aura: zeitgeist_runtime::AuraConfig {
            authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
        },
        pallet_balances: zeitgeist_runtime::BalancesConfig {
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, initial_balance))
                .collect(),
        },
        #[cfg(not(feature = "parachain"))]
        pallet_grandpa: zeitgeist_runtime::GrandpaConfig {
            authorities: initial_authorities
                .iter()
                .map(|x| (x.1.clone(), 1))
                .collect(),
        },
        pallet_sudo: zeitgeist_runtime::SudoConfig {
            // Assign the network admin rights.
            key: root_key,
        },
        #[cfg(feature = "parachain")]
        parachain_info: zeitgeist_runtime::ParachainInfoConfig { parachain_id: id },
    }
}
