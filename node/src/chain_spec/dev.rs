#![cfg(feature = "with-battery-station-runtime")]

use super::{
    battery_station::BatteryStationChainSpec, generate_generic_genesis_function,
    get_account_id_from_seed, get_from_seed, token_properties, AdditionalChainSpec,
    EndowedAccountWithBalance,
};

use sc_service::ChainType;
use sp_core::sr25519;
use zeitgeist_primitives::{
    constants::{
        ztg::{LIQUIDITY_MINING, LIQUIDITY_MINING_PTD},
        BalanceFractionalDecimals, BASE,
    },
    types::{AccountId, Balance, Signature},
};
#[cfg(feature = "parachain")]
use {
    super::{Extensions, DEFAULT_COLLATOR_INFLATION_INFO},
    battery_station_runtime::{
        CollatorDeposit, DefaultBlocksPerRound, EligibilityValue, MinCollatorStk, PolkadotXcmConfig,
    },
    sp_runtime::Perbill,
    zeitgeist_primitives::constants::{ztg, MILLISECS_PER_BLOCK},
};

const INITIAL_BALANCE: Balance = Balance::MAX >> 4;

#[cfg(not(feature = "parachain"))]
fn authority_keys_from_seed(
    s: &str,
) -> (sp_consensus_aura::sr25519::AuthorityId, sp_finality_grandpa::AuthorityId) {
    (
        get_from_seed::<sp_consensus_aura::sr25519::AuthorityId>(s),
        get_from_seed::<sp_finality_grandpa::AuthorityId>(s),
    )
}

generate_generic_genesis_function! {battery_station_runtime,
    sudo: battery_station_runtime::SudoConfig { key: Some(get_account_id_from_seed::<sr25519::Public>("Alice")) },
}

// Dev currently uses battery station runtime for the following reasons:
// 1. It is the most experimental runtime (as of writing this)
// 2. It offers the sudo pallet
pub fn dev_config(
    #[cfg(feature = "parachain")] parachain_id: cumulus_primitives_core::ParaId,
) -> Result<BatteryStationChainSpec, String> {
    let wasm = super::battery_station::get_wasm()?;

    Ok(BatteryStationChainSpec::from_genesis(
        "Development",
        "dev",
        ChainType::Local,
        move || {
            generic_genesis(
                #[cfg(feature = "parachain")]
                AdditionalChainSpec {
                    candidates: vec![(
                        get_account_id_from_seed::<sr25519::Public>("Alice"),
                        get_from_seed::<nimbus_primitives::NimbusId>("Alice"),
                        super::battery_station::DEFAULT_STAKING_AMOUNT_BATTERY_STATION,
                    )],
                    crowdloan_fund_pot: zeitgeist_primitives::constants::BASE.saturating_mul(100),
                    inflation_info: crate::chain_spec::DEFAULT_COLLATOR_INFLATION_INFO,
                    nominations: vec![],
                    parachain_id,
                },
                #[cfg(not(feature = "parachain"))]
                AdditionalChainSpec {
                    initial_authorities: vec![authority_keys_from_seed("Alice")],
                },
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Dave"),
                    get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Eve"),
                    get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
                ]
                .into_iter()
                .map(|acc| EndowedAccountWithBalance(acc, INITIAL_BALANCE))
                .collect(),
                wasm,
            )
        },
        vec![],
        None,
        None,
        None,
        Some(token_properties("DEV", battery_station_runtime::SS58Prefix::get())),
        #[cfg(feature = "parachain")]
        crate::chain_spec::Extensions {
            relay_chain: "rococo-dev".into(),
            parachain_id: parachain_id.into(),
        },
        #[cfg(not(feature = "parachain"))]
        Default::default(),
    ))
}
