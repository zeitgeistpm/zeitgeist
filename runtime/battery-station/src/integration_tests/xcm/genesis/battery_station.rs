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

use crate::parameters::ZeitgeistTreasuryAccount;
use crate::integration_tests::xcm::setup::accounts;
use crate::xcm_config::config::battery_station::ID as parachain_id;
use crate::integration_tests::xcm::setup::ztg;
use crate::integration_tests::xcm::setup::roc;
use crate::integration_tests::xcm::setup::FOREIGN_PARENT_ID;
use sp_core::storage::Storage;
use sp_runtime::BuildStorage;
use crate::Asset;

const ENDOWMENT_ZTG: u128 = ztg(1_000_000);
const ENDOWMENT_ROC: u128 = roc(1_000_000);
const SAFE_XCM_VERSION: u32 = 2;

pub(crate) fn genesis() -> Storage {
	let genesis_config = crate::RuntimeGenesisConfig {
		system: crate::SystemConfig {
			code: crate::WASM_BINARY.unwrap().to_vec(),
			..Default::default()
		},

        /*
        asset_registry: crate::AssetRegistryConfig {

        }

        author_mapping: $runtime::AuthorMappingConfig {
            mappings: acs
                .candidates
                .iter()
                .cloned()
                .map(|(account_id, author_id, _)| (author_id, account_id))
                .collect(),
        },

        parachain_staking: $runtime::ParachainStakingConfig {
            blocks_per_round: acs.blocks_per_round,
            candidates: acs
                .candidates
                .iter()
                .cloned()
                .map(|(account, _, bond)| (account, bond))
                .collect(),
            collator_commission: acs.collator_commission,
            inflation_config: acs.inflation_info,
            delegations: acs.nominations,
            parachain_bond_reserve_percent: acs.parachain_bond_reserve_percent,
            num_selected_candidates: acs.num_selected_candidates,
        },
        */

		balances: crate::BalancesConfig {
			balances: accounts::init_balances().iter().map(|k| (k.clone(), ENDOWMENT_ZTG)).collect(),
		},

        tokens: crate::TokensConfig {
            balances: accounts::init_balances().
                iter()
                .chain(
                    vec![(ZeitgeistTreasuryAccount::get())].iter()
                )
                .map(|k| (k.clone(), Asset::from(FOREIGN_PARENT_ID).try_into().unwrap(), ENDOWMENT_ROC))
                .collect::<Vec<_>>()
        },

        parachain_info: crate::ParachainInfoConfig {
            parachain_id: parachain_id.into(),
            ..Default::default()
        },

        polkadot_xcm: crate::PolkadotXcmConfig {
            safe_xcm_version: Some(SAFE_XCM_VERSION),
            ..Default::default()
        },
        ..Default::default()
	};

	genesis_config.build_storage().unwrap()
}
