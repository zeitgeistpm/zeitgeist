// Copyright 2023-2024 Forecasting Technologies LTD.
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
use frame_support::traits::tokens::fungibles::Inspect;

fn inspect_test_helper(asset: Assets, initial_amount: <Runtime as crate::Config>::Balance) {
    assert!(AssetRouter::asset_exists(asset));

    assert_ok!(<AssetRouter as orml_traits::MultiCurrency<AccountId>>::deposit(
        asset,
        &ALICE,
        initial_amount
    ));
    assert_eq!(AssetRouter::total_issuance(asset), initial_amount);
    assert_eq!(AssetRouter::balance(asset, &ALICE), initial_amount);
    assert_eq!(AssetRouter::reducible_balance(asset, &ALICE, false), initial_amount);
    assert_eq!(
        AssetRouter::can_withdraw(asset, &ALICE, initial_amount),
        WithdrawConsequence::ReducedToZero(0)
    );
    assert_eq!(AssetRouter::can_deposit(asset, &ALICE, 1, true), DepositConsequence::Success);
}

#[test]
fn inspect_routes_campaign_assets_correctly() {
    ExtBuilder::default().build().execute_with(|| {
        use orml_traits::MultiCurrency;

        assert_ok!(AssetRouter::create(CAMPAIGN_ASSET, ALICE, true, CAMPAIGN_ASSET_MIN_BALANCE,));
        assert_eq!(
            <AssetRouter as Inspect<AccountId>>::minimum_balance(CAMPAIGN_ASSET),
            CAMPAIGN_ASSET_MIN_BALANCE
        );
        inspect_test_helper(CAMPAIGN_ASSET, CAMPAIGN_ASSET_INITIAL_AMOUNT);
        assert_eq!(<CustomAssets as Inspect<AccountId>>::total_issuance(CUSTOM_ASSET_INTERNAL), 0);
        assert_eq!(<MarketAssets as Inspect<AccountId>>::total_issuance(MARKET_ASSET_INTERNAL), 0);
        assert_eq!(<Tokens as MultiCurrency<AccountId>>::total_issuance(CURRENCY_INTERNAL), 0);
    });
}

#[test]
fn inspect_routes_custom_assets_correctly() {
    ExtBuilder::default().build().execute_with(|| {
        use orml_traits::MultiCurrency;

        assert_ok!(AssetRouter::create(CUSTOM_ASSET, ALICE, true, CUSTOM_ASSET_MIN_BALANCE,));
        assert_eq!(
            <AssetRouter as Inspect<AccountId>>::minimum_balance(CUSTOM_ASSET),
            CUSTOM_ASSET_MIN_BALANCE
        );
        inspect_test_helper(CUSTOM_ASSET, CUSTOM_ASSET_INITIAL_AMOUNT);
        assert_eq!(
            <CampaignAssets as Inspect<AccountId>>::total_issuance(CAMPAIGN_ASSET_INTERNAL),
            0
        );
        assert_eq!(<MarketAssets as Inspect<AccountId>>::total_issuance(MARKET_ASSET_INTERNAL), 0);
        assert_eq!(<Tokens as MultiCurrency<AccountId>>::total_issuance(CURRENCY_INTERNAL), 0);
    });
}

#[test]
fn inspect_routes_market_assets_correctly() {
    ExtBuilder::default().build().execute_with(|| {
        use orml_traits::MultiCurrency;

        assert_ok!(AssetRouter::create(MARKET_ASSET, ALICE, true, MARKET_ASSET_MIN_BALANCE,));
        assert_eq!(
            <AssetRouter as Inspect<AccountId>>::minimum_balance(MARKET_ASSET),
            MARKET_ASSET_MIN_BALANCE
        );
        inspect_test_helper(MARKET_ASSET, MARKET_ASSET_INITIAL_AMOUNT);
        assert_eq!(
            <CampaignAssets as Inspect<AccountId>>::total_issuance(CAMPAIGN_ASSET_INTERNAL),
            0
        );
        assert_eq!(<CustomAssets as Inspect<AccountId>>::total_issuance(CUSTOM_ASSET_INTERNAL), 0);
        assert_eq!(<Tokens as MultiCurrency<AccountId>>::total_issuance(CURRENCY_INTERNAL), 0);
    });
}

#[test]
fn inspect_routes_currencies_correctly() {
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(AssetRouter::minimum_balance(CURRENCY), CURRENCY_MIN_BALANCE);
        inspect_test_helper(CURRENCY, CURRENCY_INITIAL_AMOUNT);
        assert_eq!(
            <CampaignAssets as Inspect<AccountId>>::total_issuance(CAMPAIGN_ASSET_INTERNAL),
            0
        );
        assert_eq!(<CustomAssets as Inspect<AccountId>>::total_issuance(CUSTOM_ASSET_INTERNAL), 0);
        assert_eq!(<MarketAssets as Inspect<AccountId>>::total_issuance(MARKET_ASSET_INTERNAL), 0);
    });
}
