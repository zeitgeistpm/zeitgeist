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

#![cfg(test)]

use super::*;
use crate::*;
use frame_support::{
    assert_storage_noop,
    traits::{fungibles::Dust, tokens::fungibles::Unbalanced},
};
use orml_traits::MultiCurrency;

fn test_helper(asset: Assets, initial_amount: <Runtime as crate::Config>::Balance) {
    assert_eq!(
        <AssetRouter as MultiCurrency<AccountId>>::total_balance(asset, &ALICE),
        initial_amount
    );
    assert_ok!(AssetRouter::increase_balance(asset, &ALICE, 1, Precision::Exact));
    assert_eq!(
        <AssetRouter as MultiCurrency<AccountId>>::total_balance(asset, &ALICE),
        initial_amount + 1
    );
    assert_ok!(AssetRouter::decrease_balance(
        asset,
        &ALICE,
        1,
        Precision::Exact,
        Preservation::Expendable,
        Fortitude::Polite
    ));
    assert_eq!(
        <AssetRouter as MultiCurrency<AccountId>>::total_balance(asset, &ALICE),
        initial_amount
    );
    AssetRouter::set_total_issuance(asset, 1337);
    assert_eq!(<AssetRouter as MultiCurrency<AccountId>>::total_issuance(asset), 1337);
    assert_storage_noop!(AssetRouter::deactivate(asset, 1));
    assert_storage_noop!(AssetRouter::reactivate(asset, 1));
    assert_storage_noop!(AssetRouter::handle_raw_dust(asset, 1));
}

#[test]
fn routes_campaign_assets_correctly() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(AssetRouter::create(CAMPAIGN_ASSET, ALICE, true, CAMPAIGN_ASSET_MIN_BALANCE));
        assert_ok!(AssetRouter::deposit(CAMPAIGN_ASSET, &ALICE, CAMPAIGN_ASSET_INITIAL_AMOUNT));

        test_helper(CAMPAIGN_ASSET, CAMPAIGN_ASSET_INITIAL_AMOUNT);

        assert_eq!(<AssetRouter as MultiCurrency<AccountId>>::total_issuance(CUSTOM_ASSET), 0);
        assert_eq!(<AssetRouter as MultiCurrency<AccountId>>::total_issuance(MARKET_ASSET), 0);
        assert_eq!(<AssetRouter as MultiCurrency<AccountId>>::total_issuance(CURRENCY), 0);
    });
}

#[test]
#[should_panic]
fn campaign_assets_panic_on_write_balance() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(AssetRouter::create(CAMPAIGN_ASSET, ALICE, true, CAMPAIGN_ASSET_MIN_BALANCE));
        let _ = AssetRouter::write_balance(CAMPAIGN_ASSET, &ALICE, 42);
    });
}

#[test]
#[should_panic]
fn campaign_assets_panic_on_handle_dust() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(AssetRouter::create(CAMPAIGN_ASSET, ALICE, true, CAMPAIGN_ASSET_MIN_BALANCE));
        AssetRouter::handle_dust(Dust(CAMPAIGN_ASSET, 1));
    });
}

#[test]
fn routes_custom_assets_correctly() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(AssetRouter::create(CUSTOM_ASSET, ALICE, true, CUSTOM_ASSET_MIN_BALANCE));
        assert_ok!(AssetRouter::deposit(CUSTOM_ASSET, &ALICE, CUSTOM_ASSET_INITIAL_AMOUNT));

        test_helper(CUSTOM_ASSET, CUSTOM_ASSET_INITIAL_AMOUNT);

        assert_eq!(<AssetRouter as MultiCurrency<AccountId>>::total_issuance(CAMPAIGN_ASSET), 0);
        assert_eq!(<AssetRouter as MultiCurrency<AccountId>>::total_issuance(MARKET_ASSET), 0);
        assert_eq!(<AssetRouter as MultiCurrency<AccountId>>::total_issuance(CURRENCY), 0);
    });
}

#[test]
#[should_panic]
fn custom_assets_panic_on_write_balance() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(AssetRouter::create(CUSTOM_ASSET, ALICE, true, CUSTOM_ASSET_MIN_BALANCE));
        let _ = AssetRouter::write_balance(CUSTOM_ASSET, &ALICE, 42);
    });
}

#[test]
#[should_panic]
fn custom_assets_panic_on_handle_dust() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(AssetRouter::create(CUSTOM_ASSET, ALICE, true, CUSTOM_ASSET_MIN_BALANCE));
        AssetRouter::handle_dust(Dust(CUSTOM_ASSET, 1));
    });
}

#[test]
fn routes_market_assets_correctly() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(AssetRouter::create(MARKET_ASSET, ALICE, true, MARKET_ASSET_MIN_BALANCE));
        assert_ok!(AssetRouter::deposit(MARKET_ASSET, &ALICE, MARKET_ASSET_INITIAL_AMOUNT));

        test_helper(MARKET_ASSET, MARKET_ASSET_INITIAL_AMOUNT);

        assert_eq!(<AssetRouter as MultiCurrency<AccountId>>::total_issuance(CAMPAIGN_ASSET), 0);
        assert_eq!(<AssetRouter as MultiCurrency<AccountId>>::total_issuance(CUSTOM_ASSET), 0);
        assert_eq!(<AssetRouter as MultiCurrency<AccountId>>::total_issuance(CURRENCY), 0);
    });
}

#[test]
#[should_panic]
fn market_assets_panic_on_write_balance() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(AssetRouter::create(MARKET_ASSET, ALICE, true, MARKET_ASSET_MIN_BALANCE));
        let _ = AssetRouter::write_balance(MARKET_ASSET, &ALICE, 42);
    });
}

#[test]
#[should_panic]
fn market_assets_panic_on_handle_dust() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(AssetRouter::create(MARKET_ASSET, ALICE, true, MARKET_ASSET_MIN_BALANCE));
        AssetRouter::handle_dust(Dust(MARKET_ASSET, 1));
    });
}

#[test]
fn routes_currencies_correctly() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(AssetRouter::write_balance(CURRENCY, &ALICE, CURRENCY_INITIAL_AMOUNT));
        test_helper(CURRENCY, CURRENCY_INITIAL_AMOUNT);
        assert_storage_noop!(AssetRouter::handle_dust(Dust(CURRENCY, 1)));
        assert_ok!(AssetRouter::write_balance(CURRENCY, &ALICE, CURRENCY_MIN_BALANCE));
        assert_eq!(AssetRouter::free_balance(CURRENCY, &ALICE), CURRENCY_MIN_BALANCE);

        assert_eq!(<AssetRouter as MultiCurrency<AccountId>>::total_issuance(CAMPAIGN_ASSET), 0);
        assert_eq!(<AssetRouter as MultiCurrency<AccountId>>::total_issuance(CUSTOM_ASSET), 0);
        assert_eq!(<AssetRouter as MultiCurrency<AccountId>>::total_issuance(MARKET_ASSET), 0);
    });
}
