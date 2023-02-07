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

use crate::{
    xcm_config::config::{battery_station, general_key},
    AccountId, AssetRegistry, Balance, CurrencyId, ExistentialDeposit, Runtime, RuntimeOrigin,
    System,
};
use frame_support::{assert_ok, traits::GenesisBuild};
use orml_traits::asset_registry::AssetMetadata;
use xcm::{
    latest::{Junction::Parachain, Junctions::X2, MultiLocation},
    VersionedMultiLocation,
};
use zeitgeist_primitives::types::{Asset, CustomMetadata};

pub(super) struct ExtBuilder {
    balances: Vec<(AccountId, CurrencyId, Balance)>,
    parachain_id: u32,
}

impl Default for ExtBuilder {
    fn default() -> Self {
        Self { balances: vec![], parachain_id: battery_station::ID }
    }
}

impl ExtBuilder {
    pub fn set_balances(mut self, balances: Vec<(AccountId, CurrencyId, Balance)>) -> Self {
        self.balances = balances;
        self
    }

    pub fn set_parachain_id(mut self, parachain_id: u32) -> Self {
        self.parachain_id = parachain_id;
        self
    }

    pub fn build(self) -> sp_io::TestExternalities {
        let mut t = frame_system::GenesisConfig::default().build_storage::<Runtime>().unwrap();
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
            &parachain_info::GenesisConfig { parachain_id: self.parachain_id.into() },
            &mut t,
        )
        .unwrap();

        <pallet_xcm::GenesisConfig as GenesisBuild<Runtime>>::assimilate_storage(
            &pallet_xcm::GenesisConfig { safe_xcm_version: Some(2) },
            &mut t,
        )
        .unwrap();

        let mut ext = sp_io::TestExternalities::new(t);
        ext.execute_with(|| System::set_block_number(1));
        ext
    }
}

/// Accounts
pub const ALICE: [u8; 32] = [4u8; 32];
pub const BOB: [u8; 32] = [5u8; 32];

/// A PARA ID used for a sibling parachain.
/// It must be one that doesn't collide with any other in use.
pub const PARA_ID_SIBLING: u32 = 3000;

/// IDs that are used to represent tokens from other chains
pub const FOREIGN_ZTG_ID: Asset<u128> = CurrencyId::ForeignAsset(0);
pub const FOREIGN_PARENT_ID: Asset<u128> = CurrencyId::ForeignAsset(1);
pub const FOREIGN_SIBLING_ID: Asset<u128> = CurrencyId::ForeignAsset(2);

// Multilocations that are used to represent tokens from other chains
#[inline]
pub(super) fn foreign_ztg_multilocation() -> MultiLocation {
    MultiLocation::new(1, X2(Parachain(battery_station::ID), general_key(battery_station::KEY)))
}

#[inline]
pub(super) fn foreign_sibling_multilocation() -> MultiLocation {
    MultiLocation::new(1, X2(Parachain(PARA_ID_SIBLING), general_key(battery_station::KEY)))
}

#[inline]
pub(super) fn foreign_parent_multilocation() -> MultiLocation {
    MultiLocation::parent()
}

pub(super) fn register_foreign_ztg(additional_meta: Option<CustomMetadata>) {
    // Register ZTG as foreign asset.
    let meta: AssetMetadata<Balance, CustomMetadata> = AssetMetadata {
        decimals: 10,
        name: "Zeitgeist".into(),
        symbol: "ZTG".into(),
        existential_deposit: ExistentialDeposit::get(),
        location: Some(VersionedMultiLocation::V1(foreign_ztg_multilocation())),
        additional: additional_meta.unwrap_or_default(),
    };

    assert_ok!(AssetRegistry::register_asset(RuntimeOrigin::root(), meta, Some(FOREIGN_ZTG_ID)));
}

pub(super) fn register_foreign_sibling(additional_meta: Option<CustomMetadata>) {
    // Register native Sibling token as foreign asset.
    let meta: AssetMetadata<Balance, CustomMetadata> = AssetMetadata {
        decimals: 10,
        name: "Sibling".into(),
        symbol: "SBL".into(),
        existential_deposit: ExistentialDeposit::get(),
        location: Some(VersionedMultiLocation::V1(foreign_sibling_multilocation())),
        additional: additional_meta.unwrap_or_default(),
    };

    assert_ok!(AssetRegistry::register_asset(
        RuntimeOrigin::root(),
        meta,
        Some(FOREIGN_SIBLING_ID)
    ));
}

pub(super) fn register_foreign_parent(additional_meta: Option<CustomMetadata>) {
    // Register KSM as foreign asset in the sibling parachain
    let meta: AssetMetadata<Balance, CustomMetadata> = AssetMetadata {
        decimals: 12,
        name: "Kusama".into(),
        symbol: "KSM".into(),
        existential_deposit: 10_000_000_000, // 0.01
        location: Some(VersionedMultiLocation::V1(foreign_parent_multilocation())),
        additional: additional_meta.unwrap_or_default(),
    };

    assert_ok!(AssetRegistry::register_asset(RuntimeOrigin::root(), meta, Some(FOREIGN_PARENT_ID)));
}

#[inline]
pub(super) fn ztg(amount: Balance) -> Balance {
    amount * dollar(10)
}

#[inline]
pub(super) fn ksm(amount: Balance) -> Balance {
    foreign(amount, 12)
}

#[inline]
pub(super) fn foreign(amount: Balance, decimals: u32) -> Balance {
    amount * dollar(decimals)
}

#[inline]
pub(super) fn dollar(decimals: u32) -> Balance {
    10u128.saturating_pow(decimals)
}

#[inline]
pub(super) fn sibling_parachain_account() -> AccountId {
    parachain_account(PARA_ID_SIBLING)
}

#[inline]
pub(super) fn zeitgeist_parachain_account() -> AccountId {
    parachain_account(battery_station::ID)
}

#[inline]
fn parachain_account(id: u32) -> AccountId {
    use sp_runtime::traits::AccountIdConversion;

    polkadot_parachain::primitives::Sibling::from(id).into_account_truncating()
}
