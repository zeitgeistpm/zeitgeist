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

fn multi_lockable_currency_unroutable_test_helper(asset: Assets) {
    assert_err!(
        AssetRouter::set_lock(Default::default(), asset, &ALICE, 1),
        Error::<Runtime>::Unsupported
    );
    assert_err!(
        AssetRouter::extend_lock(Default::default(), asset, &ALICE, 1),
        Error::<Runtime>::Unsupported
    );
    assert_err!(
        AssetRouter::remove_lock(Default::default(), asset, &ALICE),
        Error::<Runtime>::Unsupported
    );
}

#[test]
fn multi_lockable_currency_routes_currencies_correctly() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(AssetRouter::deposit(CURRENCY, &ALICE, CURRENCY_INITIAL_AMOUNT));
        assert_ok!(AssetRouter::set_lock(Default::default(), CURRENCY, &ALICE, 1));
        assert_eq!(
            orml_tokens::Accounts::<Runtime>::get::<
                u128,
                <Runtime as orml_tokens::Config>::CurrencyId,
            >(ALICE, Default::default())
            .frozen,
            1
        );
        assert_ok!(AssetRouter::extend_lock(Default::default(), CURRENCY, &ALICE, 2));
        assert_eq!(
            orml_tokens::Accounts::<Runtime>::get::<
                u128,
                <Runtime as orml_tokens::Config>::CurrencyId,
            >(ALICE, Default::default())
            .frozen,
            2
        );
        assert_ok!(AssetRouter::remove_lock(Default::default(), CURRENCY, &ALICE));
        assert_eq!(
            orml_tokens::Accounts::<Runtime>::get::<
                u128,
                <Runtime as orml_tokens::Config>::CurrencyId,
            >(ALICE, Default::default())
            .frozen,
            0
        );
    });
}

#[test]
fn multi_lockable_currency_routes_campaign_assets_correctly() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(AssetRouter::create(
            CAMPAIGN_ASSET,
            ALICE,
            true,
            CAMPAIGN_ASSET_MIN_BALANCE,
        ));

        multi_lockable_currency_unroutable_test_helper(CAMPAIGN_ASSET);
    });
}

#[test]
fn multi_lockable_currency_routes_custom_assets_correctly() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(AssetRouter::create(
            CUSTOM_ASSET,
            ALICE,
            true,
            CUSTOM_ASSET_MIN_BALANCE,
        ));

        multi_lockable_currency_unroutable_test_helper(CUSTOM_ASSET);
    });
}

#[test]
fn multi_lockable_currency_routes_market_assets_correctly() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(AssetRouter::create(
            MARKET_ASSET,
            ALICE,
            true,
            MARKET_ASSET_MIN_BALANCE,
        ));

        multi_lockable_currency_unroutable_test_helper(MARKET_ASSET);
    });
}
