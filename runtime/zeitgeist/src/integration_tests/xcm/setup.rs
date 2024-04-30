// Copyright 2022-2024 Forecasting Technologies LTD.
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
    xcm_config::config::{general_key, zeitgeist},
    AccountId, AssetRegistry, AssetRegistryStringLimit, Balance, ExistentialDeposit, RuntimeOrigin,
};
use frame_support::assert_ok;
use orml_traits::asset_registry::AssetMetadata;
use sp_core::{sr25519, Pair, Public};
use xcm::{
    latest::{Junction::Parachain, Junctions::X2, MultiLocation},
    VersionedMultiLocation,
};
use xcm_emulator::helpers::get_account_id_from_seed;
use zeitgeist_primitives::types::{CustomMetadata, XcmAsset};

pub(super) mod accounts {
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

    pub fn alice() -> AccountId {
        get_account_id_from_seed::<sr25519::Public>(ALICE)
    }

    pub fn bob() -> AccountId {
        get_account_id_from_seed::<sr25519::Public>(BOB)
    }

    /// Helper function to generate a crypto pair from seed
    pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
        TPublic::Pair::from_string(&format!("//{}", seed), None)
            .expect("static values are valid; qed")
            .public()
    }
}

/// A PARA ID used for a sibling parachain.
/// It must be one that doesn't collide with any other in use.
pub const PARA_ID_SIBLING: u32 = 3000;
pub const PARA_ID_ZEITGEIST: u32 = zeitgeist::ID;

/// IDs that are used to represent tokens from other chains
pub const FOREIGN_ZTG_ID: XcmAsset = XcmAsset::ForeignAsset(0);
pub const FOREIGN_PARENT_ID: XcmAsset = XcmAsset::ForeignAsset(1);
pub const FOREIGN_SIBLING_ID: XcmAsset = XcmAsset::ForeignAsset(2);
pub const BTC_ID: XcmAsset = XcmAsset::ForeignAsset(3);
pub const ETH_ID: XcmAsset = XcmAsset::ForeignAsset(4);

#[inline]
pub(super) const fn ztg(amount: Balance) -> Balance {
    amount * dollar(10)
}

#[inline]
pub(super) const fn dot(amount: Balance) -> Balance {
    foreign(amount, 10)
}

#[inline]
pub(super) const fn btc(amount: Balance) -> Balance {
    foreign(amount, 8)
}

#[inline]
pub(super) const fn eth(amount: Balance) -> Balance {
    foreign(amount, 18)
}

#[inline]
pub(super) const fn foreign(amount: Balance, decimals: u32) -> Balance {
    amount * dollar(decimals)
}

#[inline]
pub(super) const fn dollar(decimals: u32) -> Balance {
    10u128.saturating_pow(decimals)
}

#[inline]
pub(super) const fn adjusted_balance(foreign_base: Balance, amount: Balance) -> Balance {
    if foreign_base > ztg(1) {
        amount.saturating_div(foreign_base / ztg(1))
    } else {
        amount.saturating_mul(ztg(1) / foreign_base)
    }
}

// Multilocations that are used to represent tokens from other chains
#[inline]
pub(super) fn foreign_ztg_multilocation() -> MultiLocation {
    MultiLocation::new(1, X2(Parachain(PARA_ID_ZEITGEIST), general_key(zeitgeist::KEY)))
}

#[inline]
pub(super) fn foreign_sibling_multilocation() -> MultiLocation {
    MultiLocation::new(1, X2(Parachain(PARA_ID_SIBLING), general_key(zeitgeist::KEY)))
}

#[inline]
pub(super) fn foreign_parent_multilocation() -> MultiLocation {
    MultiLocation::parent()
}

pub(super) fn register_foreign_ztg(additional_meta: Option<CustomMetadata>) {
    // Register ZTG as foreign asset.
    let meta: AssetMetadata<Balance, CustomMetadata, AssetRegistryStringLimit> = AssetMetadata {
        decimals: 10,
        name: "Zeitgeist".as_bytes().to_vec().try_into().unwrap(),
        symbol: "ZTG".as_bytes().to_vec().try_into().unwrap(),
        existential_deposit: ExistentialDeposit::get(),
        location: Some(VersionedMultiLocation::V3(foreign_ztg_multilocation())),
        additional: additional_meta.unwrap_or_default(),
    };

    assert_ok!(AssetRegistry::register_asset(RuntimeOrigin::root(), meta, Some(FOREIGN_ZTG_ID)));
}

pub(super) fn register_btc(additional_meta: Option<CustomMetadata>) {
    let meta: AssetMetadata<Balance, CustomMetadata, AssetRegistryStringLimit> = AssetMetadata {
        decimals: 8,
        name: "Bitcoin".as_bytes().to_vec().try_into().unwrap(),
        symbol: "BTC".as_bytes().to_vec().try_into().unwrap(),
        existential_deposit: ExistentialDeposit::get(),
        location: Some(VersionedMultiLocation::V3(foreign_sibling_multilocation())),
        additional: additional_meta.unwrap_or_default(),
    };

    assert_ok!(AssetRegistry::register_asset(RuntimeOrigin::root(), meta, Some(BTC_ID)));
}

pub(super) fn register_eth(additional_meta: Option<CustomMetadata>) {
    let meta: AssetMetadata<Balance, CustomMetadata, AssetRegistryStringLimit> = AssetMetadata {
        decimals: 18,
        name: "Ethereum".as_bytes().to_vec().try_into().unwrap(),
        symbol: "ETH".as_bytes().to_vec().try_into().unwrap(),
        existential_deposit: ExistentialDeposit::get(),
        location: Some(VersionedMultiLocation::V3(foreign_sibling_multilocation())),
        additional: additional_meta.unwrap_or_default(),
    };

    assert_ok!(AssetRegistry::register_asset(RuntimeOrigin::root(), meta, Some(ETH_ID)));
}

pub(super) fn register_foreign_sibling(additional_meta: Option<CustomMetadata>) {
    // Register native Sibling token as foreign asset.
    let meta: AssetMetadata<Balance, CustomMetadata, AssetRegistryStringLimit> = AssetMetadata {
        decimals: 10,
        name: "Sibling".as_bytes().to_vec().try_into().unwrap(),
        symbol: "SBL".as_bytes().to_vec().try_into().unwrap(),
        existential_deposit: ExistentialDeposit::get(),
        location: Some(VersionedMultiLocation::V3(foreign_sibling_multilocation())),
        additional: additional_meta.unwrap_or_default(),
    };

    assert_ok!(AssetRegistry::register_asset(
        RuntimeOrigin::root(),
        meta,
        Some(FOREIGN_SIBLING_ID)
    ));
}

pub(super) fn register_foreign_parent(additional_meta: Option<CustomMetadata>) {
    // Register dot as foreign asset in the sibling parachain
    let meta: AssetMetadata<Balance, CustomMetadata, AssetRegistryStringLimit> = AssetMetadata {
        decimals: 10,
        name: "Polkadot".as_bytes().to_vec().try_into().unwrap(),
        symbol: "DOT".as_bytes().to_vec().try_into().unwrap(),
        existential_deposit: 10_000_000_000, // 1
        location: Some(VersionedMultiLocation::V3(foreign_parent_multilocation())),
        additional: additional_meta.unwrap_or_default(),
    };

    assert_ok!(AssetRegistry::register_asset(RuntimeOrigin::root(), meta, Some(FOREIGN_PARENT_ID)));
}

#[inline]
pub(super) fn sibling_parachain_account() -> AccountId {
    parachain_account(PARA_ID_SIBLING)
}

#[inline]
pub(super) fn zeitgeist_parachain_account() -> AccountId {
    parachain_account(PARA_ID_ZEITGEIST)
}

#[inline]
fn parachain_account(id: u32) -> AccountId {
    use sp_runtime::traits::AccountIdConversion;

    polkadot_parachain_primitives::primitives::Sibling::from(id).into_account_truncating()
}
