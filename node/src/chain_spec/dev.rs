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

#![cfg(feature = "with-battery-station-runtime")]

use super::{
    battery_station::BatteryStationChainSpec, generate_generic_genesis_function,
    get_account_id_from_seed, get_from_seed, token_properties, AdditionalChainSpec,
    EndowedAccountWithBalance,
};
#[cfg(feature = "parachain")]
use battery_station_runtime::{
    DefaultBlocksPerRound, DefaultCollatorCommission, DefaultParachainBondReservePercent,
    EligibilityValue, PolkadotXcmConfig,
};
use sc_service::ChainType;
use sp_core::sr25519;
use zeitgeist_primitives::{
    constants::ztg::{LIQUIDITY_MINING, LIQUIDITY_MINING_PTD},
    types::Balance,
};
#[cfg(feature = "parachain")]
use {
    super::battery_station::inflation_config,
    sp_runtime::Perbill,
    zeitgeist_primitives::constants::{ztg::TOTAL_INITIAL_ZTG, BASE},
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

generate_generic_genesis_function! {
    battery_station_runtime,
    sudo: battery_station_runtime::SudoConfig {
        key: Some(get_account_id_from_seed::<sr25519::Public>("Alice")),
    },
}

// Dev currently uses battery station runtime for the following reasons:
// 1. It is the most experimental runtime (as of writing this)
// 2. It offers the sudo pallet
pub fn dev_config() -> Result<BatteryStationChainSpec, String> {
    let wasm = super::battery_station::get_wasm()?;

    Ok(BatteryStationChainSpec::from_genesis(
        "Development",
        "dev",
        ChainType::Local,
        move || {
            generic_genesis(
                #[cfg(feature = "parachain")]
                AdditionalChainSpec {
                    blocks_per_round: DefaultBlocksPerRound::get(),
                    candidates: vec![(
                        get_account_id_from_seed::<sr25519::Public>("Alice"),
                        get_from_seed::<nimbus_primitives::NimbusId>("Alice"),
                        super::battery_station::DEFAULT_STAKING_AMOUNT_BATTERY_STATION,
                    )],
                    collator_commission: DefaultCollatorCommission::get(),
                    inflation_info: inflation_config(
                        Perbill::from_parts(20),
                        Perbill::from_parts(35),
                        Perbill::from_parts(50),
                        TOTAL_INITIAL_ZTG * BASE,
                    ),
                    nominations: vec![],
                    parachain_bond_reserve_percent: DefaultParachainBondReservePercent::get(),
                    parachain_id: crate::BATTERY_STATION_PARACHAIN_ID.into(),
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
            parachain_id: crate::BATTERY_STATION_PARACHAIN_ID,
        },
        #[cfg(not(feature = "parachain"))]
        Default::default(),
    ))
}
