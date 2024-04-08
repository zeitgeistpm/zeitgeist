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

#[test]
fn routes_campaign_assets_correctly() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(AssetRouter::create(CAMPAIGN_ASSET, ALICE, true, CAMPAIGN_ASSET_MIN_BALANCE));
        assert!(AssetRouter::asset_exists(CAMPAIGN_ASSET));
        assert!(!AssetRouter::asset_exists(CUSTOM_ASSET));
        assert!(!AssetRouter::asset_exists(MARKET_ASSET));
    });
}

#[test]
fn routes_custom_assets_correctly() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(AssetRouter::create(CUSTOM_ASSET, ALICE, true, CAMPAIGN_ASSET_MIN_BALANCE));
        assert!(AssetRouter::asset_exists(CUSTOM_ASSET));
        assert!(!AssetRouter::asset_exists(CAMPAIGN_ASSET));
        assert!(!AssetRouter::asset_exists(MARKET_ASSET));
    });
}

#[test]
fn routes_market_assets_correctly() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(AssetRouter::create(MARKET_ASSET, ALICE, true, CAMPAIGN_ASSET_MIN_BALANCE));
        assert!(AssetRouter::asset_exists(MARKET_ASSET));
        assert!(!AssetRouter::asset_exists(CAMPAIGN_ASSET));
        assert!(!AssetRouter::asset_exists(CUSTOM_ASSET));
    });
}

#[test]
fn routes_currencies_correctly() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            AssetRouter::create(CURRENCY, ALICE, true, CURRENCY_MIN_BALANCE),
            Error::<Runtime>::Unsupported
        );
    });
}
