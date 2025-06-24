// Copyright 2022-2025 Forecasting Technologies LTD.
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

use super::{
    battery_station::BatteryStationChainSpec, generate_generic_genesis_function, token_properties,
    AdditionalChainSpec, EndowedAccountWithBalance,
};
#[cfg(feature = "parachain")]
use battery_station_runtime::{EligibilityValue, PolkadotXcmConfig};
use sc_service::ChainType;
use sp_core::{sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};
use zeitgeist_primitives::types::{AccountId, Signature};

#[allow(dead_code)] // used in macros
type AccountPublic = <Signature as Verify>::Signer;

#[allow(dead_code)] // used in macros
fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
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

    Ok(BatteryStationChainSpec::builder(
        wasm,
        #[cfg(feature = "parachain")]
        crate::chain_spec::Extensions {
            relay_chain: "rococo-dev".into(),
            parachain_id: crate::BATTERY_STATION_PARACHAIN_ID,
            bad_blocks: None,
        },
        #[cfg(not(feature = "parachain"))]
        Default::default(),
    )
    .with_name("Development")
    .with_id("dev")
    .with_chain_type(ChainType::Local)
    .with_properties(token_properties("DEV", battery_station_runtime::SS58Prefix::get()))
    .with_genesis_config_preset_name(sp_genesis_builder::DEV_RUNTIME_PRESET)
    .build())
}
