#![cfg(feature = "with-battery-station-runtime")]

use crate::chain_spec::{
    generate_generic_genesis_function, telemetry_endpoints, token_properties
};

use battery_station_runtime::parameters::SS58Prefix;
use sc_service::ChainType;

use hex_literal::hex;
use jsonrpc_core::serde_json::{Map, Value};
use sc_telemetry::TelemetryEndpoints;
use sp_core::{crypto::UncheckedInto, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};
use zeitgeist_primitives::{
    constants::{
        ztg::{LIQUIDITY_MINING, LIQUIDITY_MINING_PTD},
        BalanceFractionalDecimals, BASE,
    },
    types::{AccountId, Balance, Signature},
};
use super::{get_from_seed, AdditionalChainSpec, EndowedAccountWithBalance};

#[cfg(feature = "parachain")]
use {
    super::{Extensions, DEFAULT_COLLATOR_INFLATION_INFO},
    sp_runtime::Perbill,
    zeitgeist_primitives::constants::{ztg, MILLISECS_PER_BLOCK},
    zeitgeist_runtime::{
        CollatorDeposit, DefaultBlocksPerRound, EligibilityValue, MinCollatorStk, PolkadotXcmConfig,
    },
};

cfg_if::cfg_if! {
    if #[cfg(feature = "parachain")] {
        const DEFAULT_STAKING_AMOUNT_BATTERY_STATION: u128 = 2_000 * BASE;
        const DEFAULT_COLLATOR_BALANCE_BATTERY_STATION: Option<u128> =
            DEFAULT_STAKING_AMOUNT_BATTERY_STATION.checked_add(CollatorDeposit::get());
        const DEFAULT_INITIAL_CROWDLOAN_FUNDS_BATTERY_STATION: u128 = 100 * BASE;
        pub type BatteryStationChainSpec = sc_service::GenericChainSpec<battery_station_runtime::GenesisConfig, Extensions>;
    } else {
        pub type BatteryStationChainSpec = sc_service::GenericChainSpec<battery_station_runtime::GenesisConfig>;
    }
}

const DEFAULT_INITIAL_BALANCE_BATTERY_STATION: u128 = 10_000 * BASE;

#[cfg(feature = "parachain")]
fn additional_chain_spec_staging_battery_station(
    parachain_id: cumulus_primitives_core::ParaId,
) -> AdditionalChainSpec {
    AdditionalChainSpec {
        candidates: vec![(
            hex!["302f6d7467ae2d7e3b9b962bfc3b9d929da9fae5f1e8c977a031ddf721b0790d"].into(),
            hex!["e6ea0b63b2b5b7247a1e8280350a14c5f9e7745dec2fe3428b68aa4167d48e66"]
                .unchecked_into(),
            DEFAULT_STAKING_AMOUNT_BATTERY_STATION,
        )],
        crowdloan_fund_pot: DEFAULT_INITIAL_CROWDLOAN_FUNDS_BATTERY_STATION,
        inflation_info: DEFAULT_COLLATOR_INFLATION_INFO,
        nominations: vec![],
        parachain_id,
    }
}

#[cfg(not(feature = "parachain"))]
pub(super) fn additional_chain_spec_staging_battery_station() -> AdditionalChainSpec {
    AdditionalChainSpec {
        initial_authorities: vec![(
            // 5FCSJzvmeUW1hBo3ASnLzSxpUdn5QUDt1Eqobj1meiQB7mLu
            hex!["8a9a54bdf73fb4a757f5ab81fabe2f173922fdb92bb8b6e8bedf8b17fa38f500"]
                .unchecked_into(),
            // 5HGProUwcyCDMJDxjBBKbv8u7ehr5uoTBS3bckYHPcZMTifW
            hex!["e61786c6426b55a034f9c4b78dc57d4183927cef8e64b2e496225ed6fca41758"]
                .unchecked_into(),
        )],
    }
}

#[cfg(not(feature = "parachain"))]
fn authority_keys_from_seed(
    s: &str,
) -> (sp_consensus_aura::sr25519::AuthorityId, sp_finality_grandpa::AuthorityId) {
    (
        get_from_seed::<sp_consensus_aura::sr25519::AuthorityId>(s),
        get_from_seed::<sp_finality_grandpa::AuthorityId>(s),
    )
}

fn endowed_accounts_staging_battery_station() -> Vec<EndowedAccountWithBalance> {
    vec![
        // 5D2L4ghyiYE8p2z7VNJo9JYwRuc8uzPWtMBqdVyvjRcsnw4P
        EndowedAccountWithBalance(
            hex!["2a6c61a907556e4c673880b5767dd4be08339ee7f2a58d5137d0c19ca9570a5c"].into(),
            DEFAULT_INITIAL_BALANCE_BATTERY_STATION,
        ),
        // 5EeeZVU4SiPG6ZRY7o8aDcav2p2mZMdu3ZLzbREWuHktYdhX
        EndowedAccountWithBalance(
            hex!["725bb6fd13d52b3d6830e5a9faed1f6499ca0f5e8aa285df09490646e71e831b"].into(),
            DEFAULT_INITIAL_BALANCE_BATTERY_STATION,
        ),
        // 5D9tF8w1FMSdz52bpiaQis1pCUZy5Gs6HcHS7gHxEzyq4XzU
        #[cfg(feature = "parachain")]
        EndowedAccountWithBalance(
            hex!["302f6d7467ae2d7e3b9b962bfc3b9d929da9fae5f1e8c977a031ddf721b0790d"].into(),
            DEFAULT_COLLATOR_BALANCE_BATTERY_STATION.unwrap(),
        ),
    ]
}

fn root_key_staging_battery_station() -> AccountId {
    hex!["2a6c61a907556e4c673880b5767dd4be08339ee7f2a58d5137d0c19ca9570a5c"].into()
}

#[inline]
fn get_wasm() -> Result<&'static [u8], String> {
    battery_station_runtime::WASM_BINARY.ok_or_else(|| "WASM binary is not available".to_string())
}

generate_generic_genesis_function!{battery_station_runtime,
    sudo: battery_station_runtime::SudoConfig { key: Some(root_key_staging_battery_station()) },
}

pub fn battery_station_staging_config(
    #[cfg(feature = "parachain")] parachain_id: cumulus_primitives_core::ParaId,
) -> Result<BatteryStationChainSpec, String> {
    let wasm = get_wasm()?;

    Ok(BatteryStationChainSpec::from_genesis(
        "Battery Station Staging",
        "battery_station_staging",
        ChainType::Live,
        move || {
            generic_genesis(
                additional_chain_spec_staging_battery_station(
                    #[cfg(feature = "parachain")]
                    parachain_id,
                ),
                endowed_accounts_staging_battery_station(),
                wasm,
            )
        },
        vec![],
        telemetry_endpoints(),
        Some("battery_station"),
        None,
        Some(token_properties("ZBS", SS58Prefix::get())),
        #[cfg(feature = "parachain")]
        crate::chain_spec::Extensions {
            relay_chain: "rococo".into(),
            parachain_id: parachain_id.into(),
        },
        #[cfg(not(feature = "parachain"))]
        Default::default(),
    ))
}
