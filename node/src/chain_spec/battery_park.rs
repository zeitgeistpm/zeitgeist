use hex_literal::hex;
use jsonrpc_core::serde_json::Map;
use sc_service::{config::TelemetryEndpoints, ChainType};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::crypto::UncheckedInto;
use sp_finality_grandpa::AuthorityId as GrandpaId;
use zeitgeist_primitives::AccountId;

use crate::chain_spec::{ChainSpec, TELEMETRY_URL};

pub fn battery_park_config() -> Result<ChainSpec, String> {
    let mut properties = Map::new();
    properties.insert("tokenSymbol".into(), "ZBP".into());
    properties.insert("tokenDecimals".into(), 10.into());

    let wasm_binary = zeitgeist_runtime::WASM_BINARY
        .ok_or("Zeitgeist runtime wasm binary not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        // Name
        "Zeitgeist Battery Park",
        // Id
        "battery_park",
        ChainType::Live,
        move || {
            battery_park_genesis(
                wasm_binary,
                // Initial PoA authorities
                vec![(
                    // 5FCSJzvmeUW1hBo3ASnLzSxpUdn5QUDt1Eqobj1meiQB7mLu
                    hex!["8a9a54bdf73fb4a757f5ab81fabe2f173922fdb92bb8b6e8bedf8b17fa38f500"]
                        .unchecked_into(),
                    // 5HGProUwcyCDMJDxjBBKbv8u7ehr5uoTBS3bckYHPcZMTifW
                    hex!["e61786c6426b55a034f9c4b78dc57d4183927cef8e64b2e496225ed6fca41758"]
                        .unchecked_into(),
                )],
                // Sudo account
                // 5D2L4ghyiYE8p2z7VNJo9JYwRuc8uzPWtMBqdVyvjRcsnw4P
                hex!["2a6c61a907556e4c673880b5767dd4be08339ee7f2a58d5137d0c19ca9570a5c"].into(),
                // Pre-funded accounts
                vec![
                    // 5D2L4ghyiYE8p2z7VNJo9JYwRuc8uzPWtMBqdVyvjRcsnw4P
                    hex!["2a6c61a907556e4c673880b5767dd4be08339ee7f2a58d5137d0c19ca9570a5c"].into(),
                ],
                true,
            )
        },
        // Bootnodes
        vec![],
        // Telemetry
        TelemetryEndpoints::new(vec![(TELEMETRY_URL.into(), 0)]).ok(),
        // Protocol ID
        Some("battery_park"),
        // Properties
        Some(properties),
        // Extensions
        Default::default(),
    ))
}

fn battery_park_genesis(
    wasm_binary: &[u8],
    initial_authorities: Vec<(AuraId, GrandpaId)>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    _enable_println: bool,
) -> zeitgeist_runtime::GenesisConfig {
    use zeitgeist_runtime::constants::currency::ZGE;
    use zeitgeist_runtime::{AuraConfig, BalancesConfig, GrandpaConfig, SudoConfig, SystemConfig};

    let initial_balance = 10_000 * ZGE;

    zeitgeist_runtime::GenesisConfig {
        frame_system: Some(SystemConfig {
            // Add Wasm runtime to storage.
            code: wasm_binary.to_vec(),
            changes_trie_config: Default::default(),
        }),
        pallet_balances: Some(BalancesConfig {
            // Configure endowed accounts with initial balance.
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, initial_balance))
                .collect(),
        }),
        pallet_aura: Some(AuraConfig {
            authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
        }),
        pallet_grandpa: Some(GrandpaConfig {
            authorities: initial_authorities
                .iter()
                .map(|x| (x.1.clone(), 1))
                .collect(),
        }),
        pallet_sudo: Some(SudoConfig {
            // Assign the network admin rights.
            key: root_key,
        }),
    }
}
