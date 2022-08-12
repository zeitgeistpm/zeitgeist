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

#![cfg(feature = "with-raumgeist-runtime")]

use super::{AdditionalChainSpec, EndowedAccountWithBalance};
use crate::chain_spec::{generate_generic_genesis_function, telemetry_endpoints, token_properties};
use hex_literal::hex;
use sc_service::ChainType;
use sp_core::crypto::UncheckedInto;
use raumgeist_runtime::parameters::SS58Prefix;

use zeitgeist_primitives::{types::{AccountId, Balance}, constants::{BASE, ztg::{LIQUIDITY_MINING, LIQUIDITY_MINING_PTD}}};

#[cfg(feature = "parachain")]
use {
    super::{Extensions, DEFAULT_COLLATOR_INFLATION_INFO},
    raumgeist_runtime::{CollatorDeposit, EligibilityValue, MinCollatorStk, PolkadotXcmConfig},
};

cfg_if::cfg_if! {
    if #[cfg(feature = "parachain")] {
        const DEFAULT_STAKING_AMOUNT_RAUMGEIST: u128 = MinCollatorStk::get();
        const DEFAULT_COLLATOR_BALANCE_RAUMGEIST: Option<u128> =
            DEFAULT_STAKING_AMOUNT_RAUMGEIST.checked_add(CollatorDeposit::get());
        const DEFAULT_INITIAL_CROWDLOAN_FUNDS_RAUMGEIST: u128 = 0;
        pub type RaumgeistChainSpec = sc_service::GenericChainSpec<raumgeist_runtime::GenesisConfig, Extensions>;
    } else {
        pub type RaumgeistChainSpec = sc_service::GenericChainSpec<raumgeist_runtime::GenesisConfig>;
    }
}

const DEFAULT_SUDO_BALANCE: Balance  = 100 * BASE;

fn endowed_accounts_staging_raumgeist() -> Vec<EndowedAccountWithBalance> {
    vec![
        // dDzt4vaprRfHqGBat44bWD4i36WMDXjsGXmCHoxMom2eQgQCd
        #[cfg(feature = "parachain")]
        EndowedAccountWithBalance(
            hex!["ec9a6c37972582ce411546f96f806cfc2bb0670f60c30cbc3ad4276834b0253c"].into(),
            DEFAULT_COLLATOR_BALANCE_RAUMGEIST.unwrap(),
        ),
        // dDy7WSPy4pvWBKsUta8MdWxduWFTpJtv9zgBiVGtqWmMh6bi6
        #[cfg(feature = "parachain")]
        EndowedAccountWithBalance(
            hex!["42a1ef95149913305fb05b6ac325ab9ed4b68c8d7aa60e3ea4daf4237dd9fc09"].into(),
            DEFAULT_COLLATOR_BALANCE_RAUMGEIST.unwrap(),
        ),
        // dE36Y98QpX8hEkLANntbtUvt7figSPGxSrDxU4sscuX989CTJ
        #[cfg(feature = "parachain")]
        EndowedAccountWithBalance(
            hex!["b4b3541a95c83a71de977a6f1e7e66e594a4d47c48b030802c90ba589c8bba16"].into(),
            DEFAULT_COLLATOR_BALANCE_RAUMGEIST.unwrap(),
        ),
        // TODO - MUST BE REPLACED!!! Do not use this key.
        EndowedAccountWithBalance(
            root_key_staging_raumgeist(),
            DEFAULT_SUDO_BALANCE,
        ),
    ]
}

#[cfg(feature = "parachain")]
fn additional_chain_spec_staging_raumgeist(
    parachain_id: cumulus_primitives_core::ParaId,
) -> AdditionalChainSpec {
    AdditionalChainSpec {
        candidates: vec![
            (
                hex!["ec9a6c37972582ce411546f96f806cfc2bb0670f60c30cbc3ad4276834b0253c"].into(),
                hex!["725d4d2948ae3a703f7a4911daa6d3022b45dc54fe1998ea88cb33a6f2bd805a"]
                    .unchecked_into(),
                DEFAULT_STAKING_AMOUNT_RAUMGEIST,
            ),
            (
                hex!["42a1ef95149913305fb05b6ac325ab9ed4b68c8d7aa60e3ea4daf4237dd9fc09"].into(),
                hex!["2cb04566bb52665950acf535c6b03312b00d896a3e33534e09dc948e16c06042"]
                    .unchecked_into(),
                DEFAULT_STAKING_AMOUNT_RAUMGEIST,
            ),
            (
                hex!["b4b3541a95c83a71de977a6f1e7e66e594a4d47c48b030802c90ba589c8bba16"].into(),
                hex!["e23846832242a083b94df7640257a243fe1c5a730890b254600d953ddd65011c"]
                    .unchecked_into(),
                DEFAULT_STAKING_AMOUNT_RAUMGEIST,
            ),
        ],
        crowdloan_fund_pot: DEFAULT_INITIAL_CROWDLOAN_FUNDS_RAUMGEIST,
        inflation_info: DEFAULT_COLLATOR_INFLATION_INFO,
        nominations: vec![],
        parachain_id,
    }
}

#[cfg(not(feature = "parachain"))]
fn additional_chain_spec_staging_raumgeist() -> AdditionalChainSpec {
    AdditionalChainSpec {
        initial_authorities: vec![(
            // 5FCSJzvmeUW1hBo3ASnLzSxpUdn5QUDt1Eqobj1meiQB7mLu
            hex!["4c47b6615262606f47a4e7ae413c534a1851c9b40fecf4b47423e930eca6c554"]
                .unchecked_into(),
            // 5HGProUwcyCDMJDxjBBKbv8u7ehr5uoTBS3bckYHPcZMTifW
            hex!["9239e2cbc4dfd2e6a35660daad84d2a9bc318b29de5aa152cd13e066d15d431a"]
                .unchecked_into(),
        )],
    }
}

#[inline]
pub(super) fn get_wasm() -> Result<&'static [u8], String> {
    raumgeist_runtime::WASM_BINARY.ok_or_else(|| "WASM binary is not available".to_string())
}

#[inline]
fn root_key_staging_raumgeist() -> AccountId {
    hex!["e6c622c6f2eaba444b68955501e535247c192b35e7b3e44e4c1dc24a514b4965"].into()
}


generate_generic_genesis_function!(
    raumgeist_runtime,
    sudo: raumgeist_runtime::SudoConfig { key: Some(root_key_staging_raumgeist()) },
);

pub fn raumgeist_staging_config(
    #[cfg(feature = "parachain")] parachain_id: cumulus_primitives_core::ParaId,
) -> Result<RaumgeistChainSpec, String> {
    let wasm = get_wasm()?;

    Ok(RaumgeistChainSpec::from_genesis(
        "Raumgeist Staging",
        "raumgeist_staging",
        ChainType::Live,
        move || {
            generic_genesis(
                additional_chain_spec_staging_raumgeist(
                    #[cfg(feature = "parachain")]
                    parachain_id,
                ),
                endowed_accounts_staging_raumgeist(),
                wasm,
            )
        },
        vec![],
        telemetry_endpoints(),
        Some("raumgeist"),
        None,
        Some(token_properties("ZTG", SS58Prefix::get())),
        #[cfg(feature = "parachain")]
        crate::chain_spec::Extensions {
            relay_chain: "polkadot".into(),
            parachain_id: parachain_id.into(),
        },
        #[cfg(not(feature = "parachain"))]
        Default::default(),
    ))
}
