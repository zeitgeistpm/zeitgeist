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

use super::{
    generate_generic_genesis_function, telemetry_endpoints, token_properties, AdditionalChainSpec,
    EndowedAccountWithBalance,
};
use hex_literal::hex;
use raumgeist_runtime::parameters::SS58Prefix;
use sc_service::ChainType;
use sp_core::crypto::UncheckedInto;

use zeitgeist_primitives::{
    constants::{
        ztg::{LIQUIDITY_MINING, LIQUIDITY_MINING_PTD},
        BASE,
    },
    types::{AccountId, Balance},
};

#[cfg(feature = "parachain")]
use {
    super::Extensions,
    raumgeist_runtime::{CollatorDeposit, EligibilityValue, MinCollatorStk, PolkadotXcmConfig},
    zeitgeist_primitives::constants::ztg::TOTAL_INITIAL_ZTG,
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

const DEFAULT_SUDO_BALANCE: Balance = 100 * BASE;

fn endowed_accounts_staging_raumgeist() -> Vec<EndowedAccountWithBalance> {
    vec![
        // dE4NNpcWPCk8TH3GM9eJV1jauEmHC3rQcxMdnTtrc3NgDGUNo
        #[cfg(feature = "parachain")]
        EndowedAccountWithBalance(
            hex!["ec9a6c37972582ce411546f96f806cfc2bb0670f60c30cbc3ad4276834b0253c"].into(),
            DEFAULT_COLLATOR_BALANCE_RAUMGEIST.unwrap(),
        ),
        // dDzXWuvDPSRXMQFAq2cJdr9NtEjtB8bohFhE3Ap9yM9s7rUQf
        #[cfg(feature = "parachain")]
        EndowedAccountWithBalance(
            hex!["42a1ef95149913305fb05b6ac325ab9ed4b68c8d7aa60e3ea4daf4237dd9fc09"].into(),
            DEFAULT_COLLATOR_BALANCE_RAUMGEIST.unwrap(),
        ),
        // dE375YCauT8vxvXwzBGaeCfPsKTXuuBpJaqCsBqRhoySNdmtE
        #[cfg(feature = "parachain")]
        EndowedAccountWithBalance(
            hex!["b4b3541a95c83a71de977a6f1e7e66e594a4d47c48b030802c90ba589c8bba16"].into(),
            DEFAULT_COLLATOR_BALANCE_RAUMGEIST.unwrap(),
        ),
        EndowedAccountWithBalance(root_key_staging_raumgeist(), DEFAULT_SUDO_BALANCE),
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
        inflation_info: inflation_config(Perbill::from_percent(5), TOTAL_INITIAL_ZTG * BASE),
        nominations: vec![],
        parachain_id,
    }
}

#[cfg(not(feature = "parachain"))]
fn additional_chain_spec_staging_raumgeist() -> AdditionalChainSpec {
    AdditionalChainSpec {
        initial_authorities: vec![(
            // Aura
            hex!["5ce5033dba3f6f730f11c20d00c34c4d3fbe23eb81471255bfde689f25dc966e"]
                .unchecked_into(),
            // Grandpa
            hex!["ffd00bcb47e83ed435ce55264cf89969041a5108fdfb3198c79dfe0b75f66600"]
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

#[cfg(feature = "parachain")]
super::generate_inflation_config_function!(raumgeist_runtime);

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
        Some(token_properties("RGT", SS58Prefix::get())),
        #[cfg(feature = "parachain")]
        crate::chain_spec::Extensions {
            relay_chain: "polkadot".into(),
            parachain_id: parachain_id.into(),
        },
        #[cfg(not(feature = "parachain"))]
        Default::default(),
    ))
}
