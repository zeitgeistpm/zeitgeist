// Copyright 2024 Forecasting Technologies LTD.
//
// Copyright (C) Parity Technologies (UK) Ltd.
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

// TODO(#1325): Replace with crate "rococo-emulated-chain" from Cumulus starting from polkadot-v1.4.0

use crate::integration_tests::xcm::setup::{accounts, accounts::get_from_seed, roc};
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use polkadot_primitives::{AccountId, AssignmentId, BlockNumber, ValidatorId};
use polkadot_runtime_parachains::configuration::HostConfiguration;
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_consensus_beefy::ecdsa_crypto::AuthorityId as BeefyId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::{sr25519, storage::Storage};
use sp_runtime::BuildStorage;
use xcm_emulator::helpers::get_account_id_from_seed;

const ENDOWMENT: u128 = roc(1_000_000);

fn session_keys(
    grandpa: GrandpaId,
    babe: BabeId,
    im_online: ImOnlineId,
    para_validator: ValidatorId,
    para_assignment: AssignmentId,
    authority_discovery: AuthorityDiscoveryId,
    beefy: BeefyId,
) -> rococo_runtime::SessionKeys {
    rococo_runtime::SessionKeys {
        grandpa,
        babe,
        im_online,
        para_validator,
        para_assignment,
        authority_discovery,
        beefy,
    }
}

fn get_host_config() -> HostConfiguration<BlockNumber> {
    HostConfiguration {
        max_upward_queue_count: 10,
        max_upward_queue_size: 51200,
        max_upward_message_size: 51200,
        max_upward_message_num_per_candidate: 10,
        max_downward_message_size: 51200,
        hrmp_sender_deposit: 0,
        hrmp_recipient_deposit: 0,
        hrmp_channel_max_capacity: 1000,
        hrmp_channel_max_message_size: 102400,
        hrmp_channel_max_total_size: 102400,
        hrmp_max_parachain_outbound_channels: 30,
        hrmp_max_parachain_inbound_channels: 30,
        ..Default::default()
    }
}

mod validators {
    use super::*;

    #[allow(clippy::type_complexity)]
    pub fn initial_authorities() -> Vec<(
        AccountId,
        AccountId,
        GrandpaId,
        BabeId,
        ImOnlineId,
        ValidatorId,
        AssignmentId,
        AuthorityDiscoveryId,
        BeefyId,
    )> {
        let seed = "Alice";
        vec![(
            get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", seed)),
            get_account_id_from_seed::<sr25519::Public>(seed),
            get_from_seed::<GrandpaId>(seed),
            get_from_seed::<BabeId>(seed),
            get_from_seed::<ImOnlineId>(seed),
            get_from_seed::<ValidatorId>(seed),
            get_from_seed::<AssignmentId>(seed),
            get_from_seed::<AuthorityDiscoveryId>(seed),
            get_from_seed::<BeefyId>(seed),
        )]
    }
}

pub(crate) fn genesis() -> Storage {
    let genesis_config = rococo_runtime::RuntimeGenesisConfig {
        system: rococo_runtime::SystemConfig {
            code: rococo_runtime::WASM_BINARY.unwrap().to_vec(),
            ..Default::default()
        },
        balances: rococo_runtime::BalancesConfig {
            balances: accounts::init_balances().iter().map(|k| (k.clone(), ENDOWMENT)).collect(),
        },
        session: rococo_runtime::SessionConfig {
            keys: validators::initial_authorities()
                .iter()
                .map(|x| {
                    (
                        x.0.clone(),
                        x.0.clone(),
                        session_keys(
                            x.2.clone(),
                            x.3.clone(),
                            x.4.clone(),
                            x.5.clone(),
                            x.6.clone(),
                            x.7.clone(),
                            x.8.clone(),
                        ),
                    )
                })
                .collect::<Vec<_>>(),
        },
        babe: rococo_runtime::BabeConfig {
            authorities: Default::default(),
            epoch_config: Some(rococo_runtime::BABE_GENESIS_EPOCH_CONFIG),
            ..Default::default()
        },
        sudo: rococo_runtime::SudoConfig {
            key: Some(get_account_id_from_seed::<sr25519::Public>("Alice")),
        },
        configuration: rococo_runtime::ConfigurationConfig { config: get_host_config() },
        registrar: rococo_runtime::RegistrarConfig {
            next_free_para_id: polkadot_primitives::LOWEST_PUBLIC_ID,
            ..Default::default()
        },
        ..Default::default()
    };

    genesis_config.build_storage().unwrap()
}
