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

fn test_helper(asset: Assets, initial_amount: <Runtime as crate::Config>::Balance) {
    assert_ok!(<AssetRouter as orml_traits::MultiCurrency<AccountId>>::deposit(
        asset,
        &ALICE,
        initial_amount
    ));
    assert!(AssetRouter::asset_exists(asset));
    assert_ok!(AssetRouter::start_destroy(asset, None));
    assert_eq!(AssetRouter::destroy_accounts(asset, 100), Ok(1));
    assert_eq!(AssetRouter::destroy_approvals(asset, 100), Ok(1));
    assert_ok!(AssetRouter::finish_destroy(asset));
    assert!(!AssetRouter::asset_exists(asset));
}

#[test]
fn routes_campaign_assets_correctly() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(AssetRouter::create(CAMPAIGN_ASSET, ALICE, true, CAMPAIGN_ASSET_MIN_BALANCE,));
        assert_ok!(
            pallet_assets::Call::<Runtime, CampaignAssetsInstance>::approve_transfer {
                id: CAMPAIGN_ASSET_INTERNAL.into(),
                delegate: BOB,
                amount: 1
            }
            .dispatch_bypass_filter(Signed(ALICE).into())
        );

        test_helper(CAMPAIGN_ASSET, CAMPAIGN_ASSET_INITIAL_AMOUNT);
    });
}

#[test]
fn routes_custom_assets_correctly() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(AssetRouter::create(CUSTOM_ASSET, ALICE, true, CUSTOM_ASSET_MIN_BALANCE,));
        assert_ok!(
            pallet_assets::Call::<Runtime, CustomAssetsInstance>::approve_transfer {
                id: CUSTOM_ASSET_INTERNAL.into(),
                delegate: BOB,
                amount: 1
            }
            .dispatch_bypass_filter(Signed(ALICE).into())
        );

        test_helper(CUSTOM_ASSET, CUSTOM_ASSET_INITIAL_AMOUNT);
    });
}

#[test]
fn routes_market_assets_correctly() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(AssetRouter::create(MARKET_ASSET, ALICE, true, MARKET_ASSET_MIN_BALANCE,));
        assert_ok!(
            pallet_assets::Call::<Runtime, MarketAssetsInstance>::approve_transfer {
                id: MARKET_ASSET_INTERNAL,
                delegate: BOB,
                amount: 1
            }
            .dispatch_bypass_filter(Signed(ALICE).into())
        );

        test_helper(MARKET_ASSET, MARKET_ASSET_INITIAL_AMOUNT);
    });
}

#[test]
fn routes_currencies_correctly() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(<AssetRouter as orml_traits::MultiCurrency<AccountId>>::deposit(
            CURRENCY,
            &ALICE,
            CURRENCY_INITIAL_AMOUNT
        ));
        assert_noop!(AssetRouter::start_destroy(CURRENCY, None), Error::<Runtime>::Unsupported);
        assert_noop!(AssetRouter::destroy_accounts(CURRENCY, 100), Error::<Runtime>::Unsupported);
        assert_noop!(AssetRouter::destroy_approvals(CURRENCY, 100), Error::<Runtime>::Unsupported);
        assert_noop!(AssetRouter::finish_destroy(CURRENCY), Error::<Runtime>::Unsupported);
    });
}
