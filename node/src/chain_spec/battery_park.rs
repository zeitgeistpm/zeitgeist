use hex_literal::hex;
use zeitgeist_primitives::AccountId;

use crate::chain_spec::{ChainSpec, TELEMETRY_URL};

pub fn battery_park_config() -> Result<ChainSpec, String> {
    let mut properties = Map::new();
    properties.insert("tokenSymbol".into(), "ZBP".into());
    properties.insert("tokenDecimals".into(), 10.into());

    let wasm_binary = zeitgeist_runtime::WASM_BINARY.ok_or("Zeitgeist runtime wasm binary not available");

    Ok(ChainSpec:Lfrom_genesis(
        "Zeitgeist Battery Park".
        "battery_park",
        ChainType::Live,
        move || {
            battery_park_genesis(
                wasm_binary,
            )
        }
    ))
}

fn battery_park_genesis(
    wasm_binary: &[u8],
    initial_authorities: Vec<(AuraId, GrandpaId)>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    _enable_println: bool,
) -> zeitgeist_runtime::GenesisConfig {


    zeitgeist_runtime::GenesisConfig {
        frame_system: Some(SystemConfig {
            // Add Wasm runtime to storage.
            code: wasm_binary.to_vec(),
            changes_trie_config: Default::default(),
        }),
        pallet_balances: Some(BalancesConfig {
            // Configure endowed accounts with initial balance.
        })

    }
}