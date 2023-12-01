// Copyright 2023 Forecasting Technologies LTD.
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
use orml_traits::MultiCurrency;

fn multicurrency_test_helper(
    asset: Assets,
    initial_amount: <Runtime as crate::Config>::Balance,
    min_balance: <Runtime as crate::Config>::Balance,
) {
    assert_eq!(AssetRouter::minimum_balance(asset), min_balance);
    assert_ok!(AssetRouter::deposit(asset, &ALICE, initial_amount));
    assert_eq!(AssetRouter::total_issuance(asset), initial_amount);
    assert_eq!(AssetRouter::total_balance(asset, &ALICE), initial_amount);
    assert_eq!(AssetRouter::free_balance(asset, &ALICE), initial_amount);
    assert_ok!(AssetRouter::ensure_can_withdraw(asset, &ALICE, initial_amount));
    assert!(AssetRouter::ensure_can_withdraw(asset, &ALICE, initial_amount + 1).is_err());
    assert_ok!(AssetRouter::transfer(asset, &ALICE, &BOB, min_balance));
    assert_eq!(AssetRouter::free_balance(asset, &ALICE), initial_amount - min_balance);
    assert_ok!(AssetRouter::withdraw(asset, &ALICE, 1));
    assert_eq!(AssetRouter::free_balance(asset, &ALICE), initial_amount - min_balance - 1);
    assert!(AssetRouter::can_slash(asset, &ALICE, 1));
    assert_eq!(AssetRouter::slash(asset, &ALICE, 1), 0);
    assert_eq!(AssetRouter::free_balance(asset, &ALICE), initial_amount - min_balance - 2);
    assert_ok!(AssetRouter::update_balance(
        asset,
        &ALICE,
        <AssetRouter as MultiCurrencyExtended<AccountId>>::Amount::from(1u8)
            - <AssetRouter as MultiCurrencyExtended<AccountId>>::Amount::from(2u8)
    ));
    assert_eq!(AssetRouter::free_balance(asset, &ALICE), initial_amount - min_balance - 3);
}

#[test]
fn multicurrency_routes_campaign_assets_correctly() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(<CampaignAssets as Create<AccountId>>::create(
            CAMPAIGN_ASSET,
            ALICE,
            true,
            CAMPAIGN_ASSET_MIN_BALANCE,
        ));

        multicurrency_test_helper(
            CAMPAIGN_ASSET_GENERAL,
            CAMPAIGN_ASSET_INITIAL_AMOUNT,
            CAMPAIGN_ASSET_MIN_BALANCE,
        );

        assert_eq!(AssetRouter::total_issuance(CUSTOM_ASSET_GENERAL), 0);
        assert_eq!(AssetRouter::total_issuance(MARKET_ASSET_GENERAL), 0);
        assert_eq!(AssetRouter::total_issuance(CURRENCY_GENERAL), 0);
    });
}

#[test]
fn multicurrency_routes_custom_assets_correctly() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(<CustomAssets as Create<AccountId>>::create(
            CUSTOM_ASSET,
            ALICE,
            true,
            CUSTOM_ASSET_MIN_BALANCE,
        ));

        multicurrency_test_helper(
            CUSTOM_ASSET_GENERAL,
            CUSTOM_ASSET_INITIAL_AMOUNT,
            CUSTOM_ASSET_MIN_BALANCE,
        );

        assert_eq!(AssetRouter::total_issuance(CAMPAIGN_ASSET_GENERAL), 0);
        assert_eq!(AssetRouter::total_issuance(MARKET_ASSET_GENERAL), 0);
        assert_eq!(AssetRouter::total_issuance(CURRENCY_GENERAL), 0);
    });
}

#[test]
fn multicurrency_routes_market_assets_correctly() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(<MarketAssets as Create<AccountId>>::create(
            MARKET_ASSET,
            ALICE,
            true,
            MARKET_ASSET_MIN_BALANCE,
        ));

        multicurrency_test_helper(
            MARKET_ASSET_GENERAL,
            MARKET_ASSET_INITIAL_AMOUNT,
            MARKET_ASSET_MIN_BALANCE,
        );

        assert_eq!(AssetRouter::total_issuance(CAMPAIGN_ASSET_GENERAL), 0);
        assert_eq!(AssetRouter::total_issuance(CUSTOM_ASSET_GENERAL), 0);
        assert_eq!(AssetRouter::total_issuance(CURRENCY_GENERAL), 0);
    });
}

#[test]
fn multicurrency_routes_currencies_correctly() {
    ExtBuilder::default().build().execute_with(|| {
        multicurrency_test_helper(CURRENCY_GENERAL, CURRENCY_INITIAL_AMOUNT, CURRENCY_MIN_BALANCE);

        assert_eq!(AssetRouter::total_issuance(CAMPAIGN_ASSET_GENERAL), 0);
        assert_eq!(AssetRouter::total_issuance(CUSTOM_ASSET_GENERAL), 0);
        assert_eq!(AssetRouter::total_issuance(MARKET_ASSET_GENERAL), 0);
    });
}
