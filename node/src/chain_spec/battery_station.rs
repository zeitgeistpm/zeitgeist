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

use super::{AdditionalChainSpec, EndowedAccountWithBalance};
use crate::chain_spec::{generate_generic_genesis_function, telemetry_endpoints, token_properties};
use battery_station_runtime::parameters::SS58Prefix;
use hex_literal::hex;
use sc_service::ChainType;
use sp_core::crypto::UncheckedInto;
use zeitgeist_primitives::{
    constants::{
        ztg::{LIQUIDITY_MINING, LIQUIDITY_MINING_PTD},
        BASE,
    },
    types::AccountId,
};

#[cfg(feature = "parachain")]
use {
    super::{generate_inflation_config_function, Extensions},
    crate::BATTERY_STATION_PARACHAIN_ID,
    battery_station_runtime::{
        CollatorDeposit, DefaultBlocksPerRound, DefaultCollatorCommission,
        DefaultParachainBondReservePercent, EligibilityValue, PolkadotXcmConfig,
    },
    zeitgeist_primitives::constants::ztg::TOTAL_INITIAL_ZTG,
};

cfg_if::cfg_if! {
    if #[cfg(feature = "parachain")] {
        pub(super) const DEFAULT_STAKING_AMOUNT_BATTERY_STATION: u128 = 2_000 * BASE;
        const DEFAULT_COLLATOR_BALANCE_BATTERY_STATION: Option<u128> =
            DEFAULT_STAKING_AMOUNT_BATTERY_STATION.checked_add(CollatorDeposit::get());
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
        blocks_per_round: DefaultBlocksPerRound::get(),
        candidates: vec![(
            hex!["302f6d7467ae2d7e3b9b962bfc3b9d929da9fae5f1e8c977a031ddf721b0790d"].into(),
            hex!["e6ea0b63b2b5b7247a1e8280350a14c5f9e7745dec2fe3428b68aa4167d48e66"]
                .unchecked_into(),
            DEFAULT_STAKING_AMOUNT_BATTERY_STATION,
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
        parachain_id,
    }
}

#[cfg(not(feature = "parachain"))]
fn additional_chain_spec_staging_battery_station() -> AdditionalChainSpec {
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

fn endowed_accounts_staging_battery_station() -> Vec<EndowedAccountWithBalance> {
    vec![
        // 5D2L4ghyiYE8p2z7VNJo9JYwRuc8uzPWtMBqdVyvjRcsnw4P
        EndowedAccountWithBalance(
            root_key_staging_battery_station(),
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

pub(super) fn get_wasm() -> Result<&'static [u8], String> {
    battery_station_runtime::WASM_BINARY.ok_or_else(|| "WASM binary is not available".to_string())
}

generate_generic_genesis_function!(
    battery_station_runtime,
    sudo: battery_station_runtime::SudoConfig {
        key: Some(root_key_staging_battery_station()),
    },
);

#[cfg(feature = "parachain")]
generate_inflation_config_function!(battery_station_runtime);

pub fn battery_station_staging_config() -> Result<BatteryStationChainSpec, String> {
    let wasm = get_wasm()?;

    Ok(BatteryStationChainSpec::from_genesis(
        "Battery Station Staging",
        "battery_station_staging",
        ChainType::Live,
        move || {
            generic_genesis(
                additional_chain_spec_staging_battery_station(
                    #[cfg(feature = "parachain")]
                    BATTERY_STATION_PARACHAIN_ID.into(),
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
            parachain_id: BATTERY_STATION_PARACHAIN_ID,
        },
        #[cfg(not(feature = "parachain"))]
        Default::default(),
    ))
}
