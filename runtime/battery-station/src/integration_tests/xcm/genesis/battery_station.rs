// Copyright 2024 Forecasting Technologies LTD.
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

use crate::{
    integration_tests::xcm::setup::{
        accounts, ztg, BTC_ID, FOREIGN_PARENT_ID, FOREIGN_SIBLING_ID, FOREIGN_ZTG_ID,
    },
    parachain_params::MinCandidateStk,
    parameters::ZeitgeistTreasuryAccount,
    Asset,
};
use nimbus_primitives::NimbusId;
use sp_core::storage::Storage;
use sp_runtime::BuildStorage;

const ENDOWMENT: u128 = ztg(1_000_000);
const SAFE_XCM_VERSION: u32 = 2;

pub(crate) fn genesis(parachain_id: u32) -> Storage {
    let genesis_config = crate::RuntimeGenesisConfig {
        author_mapping: crate::AuthorMappingConfig {
            mappings: vec![(
                accounts::get_from_seed::<NimbusId>(accounts::ALICE),
                accounts::alice(),
            )],
        },
        balances: crate::BalancesConfig {
            balances: accounts::init_balances().iter().map(|k| (k.clone(), ENDOWMENT)).collect(),
        },
        parachain_info: crate::ParachainInfoConfig {
            parachain_id: parachain_id.into(),
            ..Default::default()
        },
        parachain_staking: crate::ParachainStakingConfig {
            candidates: vec![(accounts::alice(), MinCandidateStk::get())],
            ..Default::default()
        },
        polkadot_xcm: crate::PolkadotXcmConfig {
            safe_xcm_version: Some(SAFE_XCM_VERSION),
            ..Default::default()
        },
        system: crate::SystemConfig {
            code: crate::WASM_BINARY.unwrap().to_vec(),
            ..Default::default()
        },
        tokens: crate::TokensConfig {
            balances: accounts::init_balances()
                .iter()
                .chain([(ZeitgeistTreasuryAccount::get())].iter())
                .flat_map(|k| {
                    vec![
                        (k.clone(), Asset::from(FOREIGN_PARENT_ID).try_into().unwrap(), ENDOWMENT),
                        (k.clone(), Asset::from(FOREIGN_SIBLING_ID).try_into().unwrap(), ENDOWMENT),
                        (k.clone(), Asset::from(FOREIGN_ZTG_ID).try_into().unwrap(), ENDOWMENT),
                        (k.clone(), Asset::from(BTC_ID).try_into().unwrap(), ENDOWMENT),
                    ]
                })
                .collect::<Vec<_>>(),
        },
        ..Default::default()
    };

    genesis_config.build_storage().unwrap()
}
