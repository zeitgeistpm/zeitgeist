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
use frame_support::traits::tokens::fungibles::Create;
use orml_traits::MultiCurrency;
use zeitgeist_primitives::types::{CampaignAssetClass, Currencies, CustomAssetClass, MarketAsset, Assets};

#[test]
fn multicurrency_routes_correctly() {
    ExtBuilder::default().build().execute_with(|| {
        type AccountId = <Runtime as frame_system::Config>::AccountId;

        /*
        minimum_balance
        total_issuance
        total_balance
        free_balance
        ensure_can_withdraw
        transfer
        deposit
        withdraw
        can_slash
        slash
        */

        // Fund an account with all assets
        <CampaignAssets as Create<AccountId>>::create(
            CampaignAssetClass::default(),
            ALICE,
            true,
            CAMPAIGN_ASSET_MIN_BALANCE,
        )
        .unwrap();
        assert_eq!(<AssetRouter as MultiCurrency<AccountId>>::minimum_balance(CAMPAIGN_ASSET_GENERAL), CAMPAIGN_ASSET_MIN_BALANCE);
        <AssetRouter as MultiCurrency<AccountId>>::deposit(CAMPAIGN_ASSET_GENERAL, &ALICE, 10).unwrap();
        assert_eq!(<AssetRouter as MultiCurrency<AccountId>>::total_issuance(CAMPAIGN_ASSET_GENERAL), 10);
        assert_eq!(<AssetRouter as MultiCurrency<AccountId>>::free_balance(CAMPAIGN_ASSET_GENERAL, &ALICE), 10);
        // TODO...


        <CustomAssets as Create<AccountId>>::create(
            CustomAssetClass::default(),
            ALICE,
            true,
            CUSTOM_ASSET_MIN_BALANCE,
        )
        .unwrap();
        assert_eq!(<AssetRouter as MultiCurrency<AccountId>>::minimum_balance(CUSTOM_ASSET_GENERAL), CUSTOM_ASSET_MIN_BALANCE);

        <MarketAssets as Create<AccountId>>::create(
            MarketAsset::default(),
            ALICE,
            true,
            MARKET_ASSET_MIN_BALANCE,
        )
        .unwrap();
        assert_eq!(<AssetRouter as MultiCurrency<AccountId>>::minimum_balance(MARKET_ASSET_GENERAL), MARKET_ASSET_MIN_BALANCE);

        /*
        AssetRouter::create(
            Currencies::default(),
            ALICE,
            true,
            CURRENCY_MIN_BALANCE,
        );
        */
    });
}
