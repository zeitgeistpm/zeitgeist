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

/*
-> Asset destruction works
-> Asset destruction respects weight
-> Asset destruction moves invalid assets to other storage
*/

#[test]
fn adds_assets_properly() {
    ExtBuilder::default().build().execute_with(|| {
        assert_err!(
            AssetRouter::managed_destroy(CAMPAIGN_ASSET, None),
            Error::<Runtime>::UnknownAsset
        );

        assert_ok!(AssetRouter::create(CAMPAIGN_ASSET, ALICE, true, CAMPAIGN_ASSET_MIN_BALANCE));
        assert_ok!(AssetRouter::managed_destroy(CAMPAIGN_ASSET, None));
        assert_err!(
            AssetRouter::managed_destroy(CAMPAIGN_ASSET, None),
            Error::<Runtime>::DestructionInProgress
        );
        assert_eq!(crate::DestroyAssets::<Runtime>::get(), vec![CAMPAIGN_ASSET]);

        assert_ok!(AssetRouter::create(CUSTOM_ASSET, ALICE, true, CUSTOM_ASSET_MIN_BALANCE));
        assert_ok!(AssetRouter::managed_destroy(CUSTOM_ASSET, None));
        assert_eq!(crate::DestroyAssets::<Runtime>::get(), vec![CAMPAIGN_ASSET, CUSTOM_ASSET]);

        crate::IndestructibleAssets::<Runtime>::put(crate::DestroyAssets::<Runtime>::get());
        crate::DestroyAssets::<Runtime>::kill();
        assert_err!(
            AssetRouter::managed_destroy(CAMPAIGN_ASSET, None),
            Error::<Runtime>::DestructionInProgress
        );
        assert_err!(
            AssetRouter::managed_destroy(CUSTOM_ASSET, None),
            Error::<Runtime>::DestructionInProgress
        );
    });
}

#[test]
fn adds_multi_assets_properly() {
    ExtBuilder::default().build().execute_with(|| {
        let assets = BTreeMap::from([(CAMPAIGN_ASSET, None), (CUSTOM_ASSET, None)]);
        assert_err!(
            AssetRouter::managed_destroy_multi(assets.clone()),
            Error::<Runtime>::UnknownAsset
        );

        for (asset, _) in assets.clone() {
            assert_ok!(AssetRouter::create(asset.clone(), ALICE, true, CAMPAIGN_ASSET_MIN_BALANCE));
        }

        assert_ok!(AssetRouter::managed_destroy_multi(assets.clone()));

        for (asset, _) in assets.clone() {
            assert_err!(
                AssetRouter::managed_destroy(asset, None),
                Error::<Runtime>::DestructionInProgress
            );
        }

        assert_err!(
            AssetRouter::managed_destroy_multi(assets.clone()),
            Error::<Runtime>::DestructionInProgress
        );
        assert_eq!(crate::DestroyAssets::<Runtime>::get(), vec![CAMPAIGN_ASSET, CUSTOM_ASSET]);

        crate::IndestructibleAssets::<Runtime>::put(crate::DestroyAssets::<Runtime>::get());
        crate::DestroyAssets::<Runtime>::kill();
        assert_err!(
            AssetRouter::managed_destroy_multi(assets),
            Error::<Runtime>::DestructionInProgress
        );
    });
}

#[test]
fn destroys_assets_fully_properly() {
    ExtBuilder::default().build().execute_with(|| {
        let assets_raw = [(CAMPAIGN_ASSET, None), (CUSTOM_ASSET, None), (MARKET_ASSET, None)];
        let assets = BTreeMap::from_iter(assets_raw.to_vec());

        for (asset, _) in &assets_raw[..] {
            assert_ok!(AssetRouter::create(asset.clone(), ALICE, true, CAMPAIGN_ASSET_MIN_BALANCE));
        }

        assert_ok!(AssetRouter::managed_destroy_multi(assets.clone()));
        assert_eq!(crate::DestroyAssets::<Runtime>::get().len(), 3);

        let available_weight = 1_000_000_000.into();
        let remaining_weight = AssetRouter::on_idle(0, available_weight);
        assert!(!AssetRouter::asset_exists(CAMPAIGN_ASSET));
        assert!(!AssetRouter::asset_exists(CUSTOM_ASSET));
        assert!(!AssetRouter::asset_exists(MARKET_ASSET));
        assert_eq!(crate::IndestructibleAssets::<Runtime>::get(), vec![]);
        assert_eq!(crate::DestroyAssets::<Runtime>::get(), vec![]);

        let rw_weight = <<Runtime as frame_system::Config>::DbWeight as Get<frame_support::weights::RuntimeDbWeight>>::get().reads_writes(1,1);
        assert_eq!(remaining_weight, available_weight - rw_weight - 3u64 * 3u64 * DESTROY_WEIGHT);
    })
}
