#![cfg(feature = "with-zeitgeist-runtime")]

use crate::chain_spec::{
    generate_generic_genesis_function, telemetry_endpoints, token_properties
};
use zeitgeist_runtime::parameters::SS58Prefix;
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
        const DEFAULT_STAKING_AMOUNT_ZEITGEIST: u128 = MinCollatorStk::get();
        const DEFAULT_COLLATOR_BALANCE_ZEITGEIST: Option<u128> =
            DEFAULT_STAKING_AMOUNT_ZEITGEIST.checked_add(CollatorDeposit::get());
        const DEFAULT_INITIAL_CROWDLOAN_FUNDS_ZEITGEIST: u128 = 0;
        pub type ZeitgeistChainSpec = sc_service::GenericChainSpec<zeitgeist_runtime::GenesisConfig, Extensions>;
    } else {
        pub type ZeitgeistChainSpec = sc_service::GenericChainSpec<zeitgeist_runtime::GenesisConfig>;
    }
}

fn endowed_accounts_staging_zeitgeist() -> Vec<EndowedAccountWithBalance> {
    vec![
        // dDzt4vaprRfHqGBat44bWD4i36WMDXjsGXmCHoxMom2eQgQCd
        #[cfg(feature = "parachain")]
        EndowedAccountWithBalance(
            hex!["524e9aac979cbb9ecdb7acd1635755c3b15696321a3345ca77f0ab0ae23f675a"].into(),
            DEFAULT_COLLATOR_BALANCE_ZEITGEIST.unwrap(),
        ),
        // dDy7WSPy4pvWBKsUta8MdWxduWFTpJtv9zgBiVGtqWmMh6bi6
        #[cfg(feature = "parachain")]
        EndowedAccountWithBalance(
            hex!["04163722a7f1f900c1ec502383d4959360e374c8808e13d47b3e553d761a6329"].into(),
            DEFAULT_COLLATOR_BALANCE_ZEITGEIST.unwrap(),
        ),
        // dE36Y98QpX8hEkLANntbtUvt7figSPGxSrDxU4sscuX989CTJ
        #[cfg(feature = "parachain")]
        EndowedAccountWithBalance(
            hex!["b449a256f73e59602eb742071a07e4d94aaae91e6872f28e161f34982a0bfc0d"].into(),
            DEFAULT_COLLATOR_BALANCE_ZEITGEIST.unwrap(),
        ),
    ]
}

#[cfg(feature = "parachain")]
fn additional_chain_spec_staging_zeitgeist(
    parachain_id: cumulus_primitives_core::ParaId,
) -> AdditionalChainSpec {
    AdditionalChainSpec {
        candidates: vec![
            (
                hex!["524e9aac979cbb9ecdb7acd1635755c3b15696321a3345ca77f0ab0ae23f675a"].into(),
                hex!["e251731d35dd19aeb7db1ffe06227d0b7da3b3eabb5ec1d79da453ac9949e80b"]
                    .unchecked_into(),
                DEFAULT_STAKING_AMOUNT_ZEITGEIST,
            ),
            (
                hex!["04163722a7f1f900c1ec502383d4959360e374c8808e13d47b3e553d761a6329"].into(),
                hex!["76d3384620053d1eb67e0f7fa8af93a8028e5cf74f22a12a5f2393b286463753"]
                    .unchecked_into(),
                DEFAULT_STAKING_AMOUNT_ZEITGEIST,
            ),
            (
                hex!["b449a256f73e59602eb742071a07e4d94aaae91e6872f28e161f34982a0bfc0d"].into(),
                hex!["14a3becfeeb700ff6a41927a2924493717aea238d9c5bea15368d61550f63e44"]
                    .unchecked_into(),
                DEFAULT_STAKING_AMOUNT_ZEITGEIST,
            ),
        ],
        crowdloan_fund_pot: DEFAULT_INITIAL_CROWDLOAN_FUNDS_ZEITGEIST,
        inflation_info: DEFAULT_COLLATOR_INFLATION_INFO,
        nominations: vec![],
        parachain_id,
    }
}

#[cfg(not(feature = "parachain"))]
fn additional_chain_spec_staging_zeitgeist() -> AdditionalChainSpec {
    super::battery_station::additional_chain_spec_staging_battery_station()
}

#[inline]
fn get_wasm() -> Result<&'static [u8], String> {
    zeitgeist_runtime::WASM_BINARY.ok_or_else(|| "WASM binary is not available".to_string())
}

generate_generic_genesis_function!(zeitgeist_runtime,);

pub fn zeitgeist_staging_config(
    #[cfg(feature = "parachain")] parachain_id: cumulus_primitives_core::ParaId,
) -> Result<ZeitgeistChainSpec, String> {
    let wasm = get_wasm()?;

    Ok(ZeitgeistChainSpec::from_genesis(
        "Zeitgeist Staging",
        "zeitgeist_staging",
        ChainType::Live,
        move || {
            generic_genesis(
                additional_chain_spec_staging_zeitgeist(
                    #[cfg(feature = "parachain")]
                    parachain_id,
                ),
                endowed_accounts_staging_zeitgeist(),
                wasm,
            )
        },
        vec![],
        telemetry_endpoints(),
        Some("zeitgeist"),
        None,
        Some(token_properties("ZTG", SS58Prefix::get())),
        #[cfg(feature = "parachain")]
        crate::chain_spec::Extensions {
            relay_chain: "kusama".into(),
            parachain_id: parachain_id.into(),
        },
        #[cfg(not(feature = "parachain"))]
        Default::default(),
    ))
}
