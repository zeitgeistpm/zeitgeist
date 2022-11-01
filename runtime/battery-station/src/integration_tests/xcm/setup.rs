// Copyright 2021 Centrifuge Foundation (centrifuge.io).
// Copyright 2022 Forecasting Technologies LTD.
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

use crate::{AccountId, Balance, CurrencyId, Origin, Runtime, System, xcm_config::{asset_registry::CustomMetadata, config::zeitgeist}};
use frame_support::traits::GenesisBuild;
use orml_traits::asset_registry::AssetMetadata;
use zeitgeist_primitives::types::Asset;

/// Accounts
pub const ALICE: [u8; 32] = [4u8; 32];
pub const BOB: [u8; 32] = [5u8; 32];

/// A PARA ID used for a sibling parachain.
/// It must be one that doesn't collide with any other in use.
pub const PARA_ID_SIBLING: u32 = 3000;

/// IDs that are used to represent tokens from other chains
pub const FOREIGN_ZTG: Asset<u128> = CurrencyId::ForeignAsset(0);
pub const FOREIGN_KSM: Asset<u128> = CurrencyId::ForeignAsset(1);
pub const FOREIGN_SIBLING: Asset<u128> = CurrencyId::ForeignAsset(2);


pub struct ExtBuilder {
	balances: Vec<(AccountId, CurrencyId, Balance)>,
	parachain_id: u32,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			balances: vec![],
			parachain_id: zeitgeist::ID,
		}
	}
}

impl ExtBuilder {
	pub fn balances(mut self, balances: Vec<(AccountId, CurrencyId, Balance)>) -> Self {
		self.balances = balances;
		self
	}

	pub fn parachain_id(mut self, parachain_id: u32) -> Self {
		self.parachain_id = parachain_id;
		self
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap();
		let native_currency_id = CurrencyId::Ztg;
		pallet_balances::GenesisConfig::<Runtime> {
			balances: self
				.balances
				.clone()
				.into_iter()
				.filter(|(_, currency_id, _)| *currency_id == native_currency_id)
				.map(|(account_id, _, initial_balance)| (account_id, initial_balance))
				.collect::<Vec<_>>(),
		}
		.assimilate_storage(&mut t)
		.unwrap();

		orml_tokens::GenesisConfig::<Runtime> {
			balances: self
				.balances
				.into_iter()
				.filter(|(_, currency_id, _)| *currency_id != native_currency_id)
				.collect::<Vec<_>>(),
		}
		.assimilate_storage(&mut t)
		.unwrap();

		<parachain_info::GenesisConfig as GenesisBuild<Runtime>>::assimilate_storage(
			&parachain_info::GenesisConfig {
				parachain_id: self.parachain_id.into(),
			},
			&mut t,
		)
		.unwrap();

		<pallet_xcm::GenesisConfig as GenesisBuild<Runtime>>::assimilate_storage(
			&pallet_xcm::GenesisConfig {
				safe_xcm_version: Some(2),
			},
			&mut t,
		)
		.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}

pub fn ztg(amount: Balance) -> Balance {
	amount * dollar(10)
}

pub fn sibling(amount: Balance) -> Balance {
	foreign(amount, 10)
}

pub fn ksm(amount: Balance) -> Balance {
	foreign(amount, 12)
}

pub fn foreign(amount: Balance, decimals: u32) -> Balance {
	amount * dollar(decimals)
}

pub fn dollar(decimals: u32) -> Balance {
	10u128.saturating_pow(decimals.into())
}

pub fn sibling_account() -> AccountId {
	parachain_account(PARA_ID_SIBLING.into())
}

pub fn zeitgeist_account() -> AccountId {
	parachain_account(zeitgeist::ID)
}

fn parachain_account(id: u32) -> AccountId {
	use sp_runtime::traits::AccountIdConversion;

	polkadot_parachain::primitives::Sibling::from(id).into_account()
}