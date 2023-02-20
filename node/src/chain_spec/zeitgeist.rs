// Copyright 2022-2023 Forecasting Technologies LTD.
// Copyright 2021-2022 Zeitgeist PM LLC.
//
// This file is part of Zeitgeist.
//
// Zeitgeist is free software: you can redistribute it and/or modify it
// under the terms of the GNU General Public License as published by the
// Free Software Foundation, either version 3 of the License, or (at
// your option) any later version.
//
// Zeitgeist is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Zeitgeist. If not, see <https://www.gnu.org/licenses/>.

#![cfg(feature = "with-zeitgeist-runtime")]

use super::{
    generate_generic_genesis_function, telemetry_endpoints, token_properties, AdditionalChainSpec,
    EndowedAccountWithBalance,
};
use hex_literal::hex;
use sc_service::ChainType;
use sp_core::crypto::UncheckedInto;
use zeitgeist_runtime::parameters::SS58Prefix;

use zeitgeist_primitives::constants::ztg::{LIQUIDITY_MINING, LIQUIDITY_MINING_PTD};

#[cfg(feature = "parachain")]
use {
    super::{generate_inflation_config_function, Extensions},
    crate::KUSAMA_PARACHAIN_ID,
    zeitgeist_primitives::constants::ztg::{STAKING_PTD, TOTAL_INITIAL_ZTG},
    zeitgeist_runtime::{
        CollatorDeposit, DefaultBlocksPerRound, DefaultCollatorCommission,
        DefaultParachainBondReservePercent, EligibilityValue, MinCollatorStk, PolkadotXcmConfig,
    },
};

cfg_if::cfg_if! {
    if #[cfg(feature = "parachain")] {
        const DEFAULT_STAKING_AMOUNT_ZEITGEIST: u128 = MinCollatorStk::get();
        const DEFAULT_COLLATOR_BALANCE_ZEITGEIST: Option<u128> =
            DEFAULT_STAKING_AMOUNT_ZEITGEIST.checked_add(CollatorDeposit::get());
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
    use zeitgeist_primitives::constants::BASE;

    AdditionalChainSpec {
        blocks_per_round: DefaultBlocksPerRound::get(),
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
        collator_commission: DefaultCollatorCommission::get(),
        inflation_info: inflation_config(
            STAKING_PTD * Perbill::from_percent(40),
            STAKING_PTD * Perbill::from_percent(70),
            STAKING_PTD,
            TOTAL_INITIAL_ZTG * BASE,
        ),
        nominations: vec![],
        parachain_bond_reserve_percent: DefaultParachainBondReservePercent::get(),
        parachain_id,
    }
}

#[cfg(not(feature = "parachain"))]
fn additional_chain_spec_staging_zeitgeist() -> AdditionalChainSpec {
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

pub(super) fn get_wasm() -> Result<&'static [u8], String> {
    zeitgeist_runtime::WASM_BINARY.ok_or_else(|| "WASM binary is not available".to_string())
}

generate_generic_genesis_function!(zeitgeist_runtime,);

#[cfg(feature = "parachain")]
generate_inflation_config_function!(zeitgeist_runtime);

pub fn zeitgeist_staging_config() -> Result<ZeitgeistChainSpec, String> {
    let wasm = get_wasm()?;

    Ok(ZeitgeistChainSpec::from_genesis(
        "Zeitgeist Staging",
        "zeitgeist_staging",
        ChainType::Live,
        move || {
            generic_genesis(
                additional_chain_spec_staging_zeitgeist(
                    #[cfg(feature = "parachain")]
                    KUSAMA_PARACHAIN_ID.into(),
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
            parachain_id: KUSAMA_PARACHAIN_ID,
        },
        #[cfg(not(feature = "parachain"))]
        Default::default(),
    ))
}
