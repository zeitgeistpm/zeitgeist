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

#[cfg(not(feature = "without-sudo"))]
use crate::chain_spec::root_key_staging_testnet;
use crate::chain_spec::{
    additional_chain_spec_staging_testnet, endowed_accounts_staging_testnet, generic_genesis,
    telemetry_endpoints, token_properties, zeitgeist_wasm, ChainSpec,
};
use sc_service::ChainType;

pub fn battery_station_staging_config(
    #[cfg(feature = "parachain")] parachain_id: cumulus_primitives_core::ParaId,
) -> Result<ChainSpec, String> {
    let zeitgeist_wasm = zeitgeist_wasm()?;

    Ok(ChainSpec::from_genesis(
        "Battery Station Staging",
        "battery_station_staging",
        ChainType::Live,
        move || {
            generic_genesis(
                additional_chain_spec_staging_testnet(
                    #[cfg(feature = "parachain")]
                    parachain_id,
                ),
                endowed_accounts_staging_testnet(),
                #[cfg(not(feature = "without-sudo"))]
                root_key_staging_testnet(),
                zeitgeist_wasm,
            )
        },
        vec![],
        telemetry_endpoints(),
        Some("battery_station"),
        None,
        Some(token_properties("ZBS")),
        #[cfg(feature = "parachain")]
        crate::chain_spec::Extensions {
            relay_chain: "rococo".into(),
            parachain_id: parachain_id.into(),
        },
        #[cfg(not(feature = "parachain"))]
        Default::default(),
    ))
}
