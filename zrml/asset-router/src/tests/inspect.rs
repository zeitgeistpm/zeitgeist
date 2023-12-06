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
use frame_support::traits::tokens::fungibles::Inspect;

fn test_helper(asset: Assets, initial_amount: <Runtime as crate::Config>::Balance) {
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
fn routes_campaign_assets_correctly() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(AssetRouter::create(CAMPAIGN_ASSET, ALICE, true, CAMPAIGN_ASSET_MIN_BALANCE,));

        test_helper(CAMPAIGN_ASSET, CAMPAIGN_ASSET_INITIAL_AMOUNT);

        assert_eq!(AssetRouter::total_issuance(CUSTOM_ASSET), 0);
        assert_eq!(AssetRouter::total_issuance(MARKET_ASSET), 0);
        assert_eq!(AssetRouter::total_issuance(CURRENCY), 0);
    });
}

#[test]
fn routes_custom_assets_correctly() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(AssetRouter::create(CUSTOM_ASSET, ALICE, true, CUSTOM_ASSET_MIN_BALANCE,));

        test_helper(CUSTOM_ASSET, CUSTOM_ASSET_INITIAL_AMOUNT);

        assert_eq!(AssetRouter::total_issuance(CAMPAIGN_ASSET), 0);
        assert_eq!(AssetRouter::total_issuance(MARKET_ASSET), 0);
        assert_eq!(AssetRouter::total_issuance(CURRENCY), 0);
    });
}

#[test]
fn routes_market_assets_correctly() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(AssetRouter::create(MARKET_ASSET, ALICE, true, MARKET_ASSET_MIN_BALANCE,));

        test_helper(MARKET_ASSET, MARKET_ASSET_INITIAL_AMOUNT);

        assert_eq!(AssetRouter::total_issuance(CAMPAIGN_ASSET), 0);
        assert_eq!(AssetRouter::total_issuance(CUSTOM_ASSET), 0);
        assert_eq!(AssetRouter::total_issuance(CURRENCY), 0);
    });
}

#[test]
fn multicurrency_routes_currencies_correctly() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(<AssetRouter as orml_traits::MultiCurrency<AccountId>>::deposit(
            CURRENCY,
            &ALICE,
            CURRENCY_INITIAL_AMOUNT
        ));
        assert_eq!(AssetRouter::total_issuance(CURRENCY), 0);
        assert_eq!(AssetRouter::balance(CURRENCY, &ALICE), 0);
        assert_eq!(AssetRouter::reducible_balance(CURRENCY, &ALICE, false), 0);
        assert_eq!(
            AssetRouter::can_withdraw(CURRENCY, &ALICE, CURRENCY_INITIAL_AMOUNT),
            WithdrawConsequence::UnknownAsset
        );
        assert_eq!(
            AssetRouter::can_deposit(CURRENCY, &ALICE, 1, true),
            DepositConsequence::UnknownAsset
        );
    });
}
