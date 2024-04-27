// Copyright 2024 Forecasting Technologies LTD.
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

// TODO: Replace with crate "rococo-emulated-chain" from Cumulus starting from polkadot-v1.4.0

// Substrate
use crate::Balance;
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_consensus_beefy::ecdsa_crypto::AuthorityId as BeefyId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::{sr25519, storage::Storage};

// Polkadot
use polkadot_primitives::{AssignmentId, ValidatorId};

// Cumulus

use rococo_runtime_constants::currency::UNITS as ROC;

pub const ED: Balance = rococo_runtime_constants::currency::EXISTENTIAL_DEPOSIT;
const ENDOWMENT: u128 = 1_000_000 * ROC;

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// Helper function to generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

pub fn get_host_config() -> HostConfiguration<BlockNumber> {
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

/// Helper function used in tests to build the genesis storage using given RuntimeGenesisConfig and
/// code Used in `legacy_vs_json_check` submods to verify storage building with JSON patch against
/// building with RuntimeGenesisConfig struct.
pub fn build_genesis_storage(builder: &dyn BuildStorage, code: &[u8]) -> Storage {
	let mut storage = builder.build_storage().unwrap();
	storage
		.top
		.insert(sp_core::storage::well_known_keys::CODE.to_vec(), code.into());
	storage
}

fn session_keys(
	babe: BabeId,
	grandpa: GrandpaId,
	para_validator: ValidatorId,
	para_assignment: AssignmentId,
	authority_discovery: AuthorityDiscoveryId,
	beefy: BeefyId,
) -> rococo_runtime::SessionKeys {
	rococo_runtime::SessionKeys {
		babe,
		grandpa,
		para_validator,
		para_assignment,
		authority_discovery,
		beefy,
	}
}

mod accounts {
	use super::*;
	pub const ALICE: &str = "Alice";
	pub const BOB: &str = "Bob";
	pub const CHARLIE: &str = "Charlie";
	pub const DAVE: &str = "Dave";
	pub const EVE: &str = "Eve";
	pub const FERDIE: &str = "Ferdie";
	pub const ALICE_STASH: &str = "Alice//stash";
	pub const BOB_STASH: &str = "Bob//stash";
	pub const CHARLIE_STASH: &str = "Charlie//stash";
	pub const DAVE_STASH: &str = "Dave//stash";
	pub const EVE_STASH: &str = "Eve//stash";
	pub const FERDIE_STASH: &str = "Ferdie//stash";
	pub const FERDIE_BEEFY: &str = "Ferdie//stash";

	pub fn init_balances() -> Vec<AccountId> {
		vec![
			get_account_id_from_seed::<sr25519::Public>(ALICE),
			get_account_id_from_seed::<sr25519::Public>(BOB),
			get_account_id_from_seed::<sr25519::Public>(CHARLIE),
			get_account_id_from_seed::<sr25519::Public>(DAVE),
			get_account_id_from_seed::<sr25519::Public>(EVE),
			get_account_id_from_seed::<sr25519::Public>(FERDIE),
			get_account_id_from_seed::<sr25519::Public>(ALICE_STASH),
			get_account_id_from_seed::<sr25519::Public>(BOB_STASH),
			get_account_id_from_seed::<sr25519::Public>(CHARLIE_STASH),
			get_account_id_from_seed::<sr25519::Public>(DAVE_STASH),
			get_account_id_from_seed::<sr25519::Public>(EVE_STASH),
			get_account_id_from_seed::<sr25519::Public>(FERDIE_STASH),
		]
	}
}

mod validators {
	use super::*;

	pub fn initial_authorities() -> Vec<(
		AccountId,
		AccountId,
		BabeId,
		GrandpaId,
		ValidatorId,
		AssignmentId,
		AuthorityDiscoveryId,
		BeefyId,
	)> {
		let seed = "Alice";
		vec![(
			get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", seed)),
			get_account_id_from_seed::<sr25519::Public>(seed),
			get_from_seed::<BabeId>(seed),
			get_from_seed::<GrandpaId>(seed),
			get_from_seed::<ValidatorId>(seed),
			get_from_seed::<AssignmentId>(seed),
			get_from_seed::<AuthorityDiscoveryId>(seed),
			get_from_seed::<BeefyId>(seed),
		)]
	}
}

pub(crate) fn genesis() -> Storage {
	let genesis_config = rococo_runtime::RuntimeGenesisConfig {
		system: rococo_runtime::SystemConfig::default(),
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
						),
					)
				})
				.collect::<Vec<_>>(),
		},
		babe: rococo_runtime::BabeConfig {
			authorities: Default::default(),
			epoch_config: rococo_runtime::BABE_GENESIS_EPOCH_CONFIG,
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

	build_genesis_storage(&genesis_config, rococo_runtime::WASM_BINARY.unwrap())
}