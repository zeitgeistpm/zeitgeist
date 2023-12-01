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

fn multi_reserveable_currency_unroutable_test_helper(
    asset: Assets,
    initial_amount: <Runtime as crate::Config>::Balance,
) {
    assert_ok!(AssetRouter::deposit(asset, &ALICE, initial_amount));
    assert!(!AssetRouter::can_reserve(asset, &ALICE, initial_amount));
    assert_err!(AssetRouter::reserve(asset, &ALICE, initial_amount), Error::<Runtime>::Unsupported);
    assert_eq!(AssetRouter::reserved_balance(asset, &ALICE), 0);
    assert_eq!(AssetRouter::slash_reserved(asset, &ALICE, 1), 1);
    assert_err!(
        AssetRouter::repatriate_reserved(asset, &ALICE, &BOB, 1, BalanceStatus::Reserved),
        Error::<Runtime>::Unsupported
    );
    assert_eq!(AssetRouter::unreserve(asset, &ALICE, 1), 1);
}

#[test]
fn multi_reserveable_currency_routes_currencies_correctly() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(AssetRouter::deposit(CURRENCY_GENERAL, &ALICE, CURRENCY_INITIAL_AMOUNT));

        assert!(AssetRouter::can_reserve(CURRENCY_GENERAL, &ALICE, CURRENCY_INITIAL_AMOUNT));
        assert!(!AssetRouter::can_reserve(CURRENCY_GENERAL, &ALICE, CURRENCY_INITIAL_AMOUNT + 1));
        assert_ok!(AssetRouter::reserve(CURRENCY_GENERAL, &ALICE, CURRENCY_INITIAL_AMOUNT));
        assert_eq!(
            AssetRouter::reserved_balance(CURRENCY_GENERAL, &ALICE),
            CURRENCY_INITIAL_AMOUNT
        );
        assert_eq!(AssetRouter::slash_reserved(CURRENCY_GENERAL, &ALICE, 1), 0);
        assert_eq!(
            AssetRouter::repatriate_reserved(
                CURRENCY_GENERAL,
                &ALICE,
                &BOB,
                CURRENCY_MIN_BALANCE,
                BalanceStatus::Reserved
            )
            .unwrap(),
            0
        );
        assert_eq!(AssetRouter::reserved_balance(CURRENCY_GENERAL, &BOB), CURRENCY_MIN_BALANCE);
        assert_eq!(
            AssetRouter::reserved_balance(CURRENCY_GENERAL, &ALICE),
            CURRENCY_INITIAL_AMOUNT - CURRENCY_MIN_BALANCE - 1
        );
        assert_eq!(AssetRouter::unreserve(CURRENCY_GENERAL, &ALICE, 1), 0);
        assert_eq!(
            AssetRouter::reserved_balance(CURRENCY_GENERAL, &ALICE),
            CURRENCY_INITIAL_AMOUNT - CURRENCY_MIN_BALANCE - 2
        );
    });
}

#[test]
fn multi_reserveable_currency_routes_campaign_assets_correctly() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(<CampaignAssets as Create<AccountId>>::create(
            CAMPAIGN_ASSET,
            ALICE,
            true,
            CAMPAIGN_ASSET_MIN_BALANCE,
        ));

        multi_reserveable_currency_unroutable_test_helper(
            CAMPAIGN_ASSET_GENERAL,
            CAMPAIGN_ASSET_INITIAL_AMOUNT,
        );
    });
}

#[test]
fn multi_reserveable_currency_routes_custom_assets_correctly() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(<CustomAssets as Create<AccountId>>::create(
            CUSTOM_ASSET,
            ALICE,
            true,
            CUSTOM_ASSET_MIN_BALANCE,
        ));

        multi_reserveable_currency_unroutable_test_helper(
            CUSTOM_ASSET_GENERAL,
            CUSTOM_ASSET_INITIAL_AMOUNT,
        );
    });
}

#[test]
fn multi_reserveable_currency_routes_market_assets_correctly() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(<MarketAssets as Create<AccountId>>::create(
            MARKET_ASSET,
            ALICE,
            true,
            MARKET_ASSET_MIN_BALANCE,
        ));

        multi_reserveable_currency_unroutable_test_helper(
            MARKET_ASSET_GENERAL,
            MARKET_ASSET_INITIAL_AMOUNT,
        );
    });
}
