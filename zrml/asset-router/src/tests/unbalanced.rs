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
use frame_support::traits::tokens::fungibles::Unbalanced;
use orml_traits::MultiCurrency;

fn test_helper(
    asset: Assets,
    initial_amount: <Runtime as crate::Config>::Balance,
    min_balance: <Runtime as crate::Config>::Balance,
) {
    assert_eq!(AssetRouter::total_balance(asset, &ALICE), initial_amount);
    assert_ok!(AssetRouter::increase_balance(asset, &ALICE, 1));
    assert_eq!(AssetRouter::total_balance(asset, &ALICE), initial_amount + 1);
    assert_ok!(AssetRouter::decrease_balance(asset, &ALICE, 1));
    assert_eq!(AssetRouter::total_balance(asset, &ALICE), initial_amount);
    assert_eq!(AssetRouter::increase_balance_at_most(asset, &ALICE, 1), 1);
    assert_eq!(AssetRouter::total_balance(asset, &ALICE), initial_amount + 1);
    let to_decrease = initial_amount + 2 - min_balance;
    assert_eq!(
        AssetRouter::decrease_balance_at_most(asset, &ALICE, to_decrease),
        initial_amount + 1
    );
    assert_eq!(AssetRouter::total_balance(asset, &ALICE), 0);
    AssetRouter::set_total_issuance(asset, 1337);
    assert_eq!(AssetRouter::total_issuance(asset), 1337);
}

#[test]
fn routes_campaign_assets_correctly() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(AssetRouter::create(CAMPAIGN_ASSET, ALICE, true, CAMPAIGN_ASSET_MIN_BALANCE));
        assert_ok!(AssetRouter::deposit(CAMPAIGN_ASSET, &ALICE, CAMPAIGN_ASSET_INITIAL_AMOUNT));

        test_helper(CAMPAIGN_ASSET, CAMPAIGN_ASSET_INITIAL_AMOUNT, CAMPAIGN_ASSET_MIN_BALANCE);

        assert_eq!(AssetRouter::total_issuance(CUSTOM_ASSET), 0);
        assert_eq!(AssetRouter::total_issuance(MARKET_ASSET), 0);
        assert_eq!(AssetRouter::total_issuance(CURRENCY), 0);
    });
}

#[test]
#[should_panic]
fn campaign_assets_panic_on_set_balance() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(AssetRouter::create(CAMPAIGN_ASSET, ALICE, true, CAMPAIGN_ASSET_MIN_BALANCE));
        let _ = AssetRouter::set_balance(CAMPAIGN_ASSET, &ALICE, 42);
    });
}

#[test]
fn routes_custom_assets_correctly() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(AssetRouter::create(CUSTOM_ASSET, ALICE, true, CUSTOM_ASSET_MIN_BALANCE));
        assert_ok!(AssetRouter::deposit(CUSTOM_ASSET, &ALICE, CUSTOM_ASSET_INITIAL_AMOUNT));

        test_helper(CUSTOM_ASSET, CUSTOM_ASSET_INITIAL_AMOUNT, CUSTOM_ASSET_MIN_BALANCE);

        assert_eq!(AssetRouter::total_issuance(CAMPAIGN_ASSET), 0);
        assert_eq!(AssetRouter::total_issuance(MARKET_ASSET), 0);
        assert_eq!(AssetRouter::total_issuance(CURRENCY), 0);
    });
}

#[test]
#[should_panic]
fn custom_assets_panic_on_set_balance() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(AssetRouter::create(CUSTOM_ASSET, ALICE, true, CUSTOM_ASSET_MIN_BALANCE));
        let _ = AssetRouter::set_balance(CUSTOM_ASSET, &ALICE, 42);
    });
}

#[test]
fn routes_market_assets_correctly() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(AssetRouter::create(MARKET_ASSET, ALICE, true, MARKET_ASSET_MIN_BALANCE));
        assert_ok!(AssetRouter::deposit(MARKET_ASSET, &ALICE, MARKET_ASSET_INITIAL_AMOUNT));

        test_helper(MARKET_ASSET, MARKET_ASSET_INITIAL_AMOUNT, MARKET_ASSET_MIN_BALANCE);

        assert_eq!(AssetRouter::total_issuance(CAMPAIGN_ASSET), 0);
        assert_eq!(AssetRouter::total_issuance(CUSTOM_ASSET), 0);
        assert_eq!(AssetRouter::total_issuance(CURRENCY), 0);
    });
}

#[test]
#[should_panic]
fn market_assets_panic_on_set_balance() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(AssetRouter::create(MARKET_ASSET, ALICE, true, MARKET_ASSET_MIN_BALANCE));
        let _ = AssetRouter::set_balance(MARKET_ASSET, &ALICE, 42);
    });
}

#[test]
fn routes_currencies_correctly() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(AssetRouter::set_balance(CURRENCY, &ALICE, CURRENCY_INITIAL_AMOUNT));
        test_helper(CURRENCY, CURRENCY_INITIAL_AMOUNT, CURRENCY_MIN_BALANCE);

        assert_eq!(AssetRouter::total_issuance(CAMPAIGN_ASSET), 0);
        assert_eq!(AssetRouter::total_issuance(CUSTOM_ASSET), 0);
        assert_eq!(AssetRouter::total_issuance(MARKET_ASSET), 0);
    });
}
