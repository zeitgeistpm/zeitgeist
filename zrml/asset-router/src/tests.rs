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

use super::mock::*;
use frame_support::{assert_ok, traits::tokens::fungibles::Create};
use orml_traits::MultiCurrency;
use zeitgeist_primitives::types::Assets;

fn multicurrency_test_helper<
    M: MultiCurrency<
            <Runtime as frame_system::Config>::AccountId,
            Balance = <Runtime as crate::Config>::Balance,
            CurrencyId = Assets,
        >,
>(
    asset: Assets,
    initial_amount: <Runtime as crate::Config>::Balance,
    min_balance: <Runtime as crate::Config>::Balance,
) {
    assert_eq!(M::minimum_balance(asset), min_balance);
    assert_ok!(M::deposit(asset, &ALICE, initial_amount));
    assert_eq!(M::total_issuance(asset), initial_amount);
    assert_eq!(M::total_balance(asset, &ALICE), initial_amount);
    assert_eq!(M::free_balance(asset, &ALICE), initial_amount);
    assert_ok!(M::ensure_can_withdraw(asset, &ALICE, initial_amount));
    assert!(M::ensure_can_withdraw(asset, &ALICE, initial_amount + 1).is_err());
    assert_ok!(M::transfer(asset, &ALICE, &BOB, min_balance));
    assert_eq!(M::free_balance(asset, &ALICE), initial_amount - min_balance);
    assert_ok!(M::withdraw(asset, &ALICE, 1));
    assert_eq!(M::free_balance(asset, &ALICE), initial_amount - min_balance - 1);
    assert!(M::can_slash(asset, &ALICE, 1));
    assert_eq!(M::slash(asset, &ALICE, 1), 0);
    assert_eq!(M::free_balance(asset, &ALICE), initial_amount - min_balance - 2);
}

#[test]
fn multicurrency_routes_campaign_assets_correctly() {
    ExtBuilder::default().build().execute_with(|| {
        // Create asset classes and fund ALICE
        <CampaignAssets as Create<AccountId>>::create(
            CAMPAIGN_ASSET,
            ALICE,
            true,
            CAMPAIGN_ASSET_MIN_BALANCE,
        )
        .unwrap();

        multicurrency_test_helper::<AssetRouter>(
            CAMPAIGN_ASSET_GENERAL,
            CAMPAIGN_ASSET_INITIAL_AMOUNT,
            CAMPAIGN_ASSET_MIN_BALANCE,
        );

        assert_eq!(
            <AssetRouter as MultiCurrency<AccountId>>::total_issuance(CUSTOM_ASSET_GENERAL),
            0
        );
        assert_eq!(
            <AssetRouter as MultiCurrency<AccountId>>::total_issuance(MARKET_ASSET_GENERAL),
            0
        );
        assert_eq!(<AssetRouter as MultiCurrency<AccountId>>::total_issuance(CURRENCY_GENERAL), 0);
    });
}

#[test]
fn multicurrency_routes_custom_assets_correctly() {
    ExtBuilder::default().build().execute_with(|| {
        // Create asset classes and fund ALICE
        <CustomAssets as Create<AccountId>>::create(
            CUSTOM_ASSET,
            ALICE,
            true,
            CUSTOM_ASSET_MIN_BALANCE,
        )
        .unwrap();

        multicurrency_test_helper::<AssetRouter>(
            CUSTOM_ASSET_GENERAL,
            CUSTOM_ASSET_INITIAL_AMOUNT,
            CUSTOM_ASSET_MIN_BALANCE,
        );

        assert_eq!(
            <AssetRouter as MultiCurrency<AccountId>>::total_issuance(CAMPAIGN_ASSET_GENERAL),
            0
        );
        assert_eq!(
            <AssetRouter as MultiCurrency<AccountId>>::total_issuance(MARKET_ASSET_GENERAL),
            0
        );
        assert_eq!(<AssetRouter as MultiCurrency<AccountId>>::total_issuance(CURRENCY_GENERAL), 0);
    });
}

#[test]
fn multicurrency_routes_market_assets_correctly() {
    ExtBuilder::default().build().execute_with(|| {
        // Create asset classes and fund ALICE
        <MarketAssets as Create<AccountId>>::create(
            MARKET_ASSET,
            ALICE,
            true,
            MARKET_ASSET_MIN_BALANCE,
        )
        .unwrap();

        multicurrency_test_helper::<AssetRouter>(
            MARKET_ASSET_GENERAL,
            MARKET_ASSET_INITIAL_AMOUNT,
            MARKET_ASSET_MIN_BALANCE,
        );

        assert_eq!(
            <AssetRouter as MultiCurrency<AccountId>>::total_issuance(CAMPAIGN_ASSET_GENERAL),
            0
        );
        assert_eq!(
            <AssetRouter as MultiCurrency<AccountId>>::total_issuance(CUSTOM_ASSET_GENERAL),
            0
        );
        assert_eq!(<AssetRouter as MultiCurrency<AccountId>>::total_issuance(CURRENCY_GENERAL), 0);
    });
}

#[test]
fn multicurrency_routes_currencies_correctly() {
    ExtBuilder::default().build().execute_with(|| {
        multicurrency_test_helper::<AssetRouter>(
            CURRENCY_GENERAL,
            CURRENCY_INITIAL_AMOUNT,
            CURRENCY_MIN_BALANCE,
        );

        assert_eq!(
            <AssetRouter as MultiCurrency<AccountId>>::total_issuance(CAMPAIGN_ASSET_GENERAL),
            0
        );
        assert_eq!(
            <AssetRouter as MultiCurrency<AccountId>>::total_issuance(CUSTOM_ASSET_GENERAL),
            0
        );
        assert_eq!(
            <AssetRouter as MultiCurrency<AccountId>>::total_issuance(MARKET_ASSET_GENERAL),
            0
        );
    });
}
