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
use crate::AssetInDestruction;
use frame_support::{traits::tokens::fungibles::Inspect, BoundedVec};
use pallet_assets::ManagedDestroy;

#[test]
fn adds_assets_properly() {
    ExtBuilder::default().build().execute_with(|| {
        let campaign_asset = AssetInDestruction::new(CAMPAIGN_ASSET);
        let custom_asset = AssetInDestruction::new(CUSTOM_ASSET);

        assert_noop!(
            AssetRouter::managed_destroy(CAMPAIGN_ASSET, None),
            Error::<Runtime>::UnknownAsset
        );

        assert_ok!(AssetRouter::create(CAMPAIGN_ASSET, ALICE, true, CAMPAIGN_ASSET_MIN_BALANCE));
        assert_ok!(AssetRouter::managed_destroy(CAMPAIGN_ASSET, None));
        assert_noop!(
            AssetRouter::managed_destroy(CAMPAIGN_ASSET, None),
            Error::<Runtime>::DestructionInProgress
        );
        assert_eq!(crate::DestroyAssets::<Runtime>::get(), vec![campaign_asset]);

        assert_ok!(AssetRouter::create(CUSTOM_ASSET, ALICE, true, CUSTOM_ASSET_MIN_BALANCE));
        assert_ok!(AssetRouter::managed_destroy(CUSTOM_ASSET, None));
        let mut expected = vec![campaign_asset, custom_asset];
        expected.sort();
        assert_eq!(crate::DestroyAssets::<Runtime>::get(), expected);

        crate::IndestructibleAssets::<Runtime>::put(BoundedVec::truncate_from(vec![
            CAMPAIGN_ASSET,
            CUSTOM_ASSET,
        ]));
        crate::DestroyAssets::<Runtime>::kill();
        assert_noop!(
            AssetRouter::managed_destroy(CAMPAIGN_ASSET, None),
            Error::<Runtime>::AssetIndestructible
        );
        assert_noop!(
            AssetRouter::managed_destroy(CUSTOM_ASSET, None),
            Error::<Runtime>::AssetIndestructible
        );
    });
}

#[test]
fn adds_multi_assets_properly() {
    ExtBuilder::default().build().execute_with(|| {
        let assets = BTreeMap::from([(CAMPAIGN_ASSET, None), (CUSTOM_ASSET, None)]);
        let campaign_asset = AssetInDestruction::new(CAMPAIGN_ASSET);
        let custom_asset = AssetInDestruction::new(CUSTOM_ASSET);

        assert_noop!(
            managed_destroy_multi_transactional(assets.clone()),
            Error::<Runtime>::UnknownAsset
        );

        for (asset, _) in assets.clone() {
            assert_ok!(AssetRouter::create(asset, ALICE, true, CAMPAIGN_ASSET_MIN_BALANCE));
        }

        assert_ok!(managed_destroy_multi_transactional(assets.clone()));

        for (asset, _) in assets.clone() {
            assert_noop!(
                AssetRouter::managed_destroy(asset, None),
                Error::<Runtime>::DestructionInProgress
            );
        }

        assert_noop!(
            managed_destroy_multi_transactional(assets.clone()),
            Error::<Runtime>::DestructionInProgress
        );
        let mut expected = vec![campaign_asset, custom_asset];
        expected.sort();
        assert_eq!(crate::DestroyAssets::<Runtime>::get(), expected);

        crate::IndestructibleAssets::<Runtime>::put(BoundedVec::truncate_from(vec![
            CAMPAIGN_ASSET,
            CUSTOM_ASSET,
        ]));
        crate::DestroyAssets::<Runtime>::kill();
        assert_noop!(
            managed_destroy_multi_transactional(assets),
            Error::<Runtime>::AssetIndestructible
        );
    });
}

#[test]
fn destroys_assets_fully_works_properly() {
    ExtBuilder::default().build().execute_with(|| {
        let assets_raw = [(CAMPAIGN_ASSET, None), (CUSTOM_ASSET, None), (MARKET_ASSET, None)];
        let assets = BTreeMap::from_iter(assets_raw.to_vec());

        for (asset, _) in &assets_raw[..] {
            assert_ok!(AssetRouter::create(*asset, ALICE, true, CAMPAIGN_ASSET_MIN_BALANCE));
        }

        assert_ok!(managed_destroy_multi_transactional(assets.clone()));
        assert_eq!(crate::DestroyAssets::<Runtime>::get().len(), 3);

        let available_weight = 1_000_000_000.into();
        let remaining_weight = AssetRouter::on_idle(0, available_weight);
        assert!(!AssetRouter::asset_exists(CAMPAIGN_ASSET));
        assert!(!AssetRouter::asset_exists(CUSTOM_ASSET));
        assert!(!AssetRouter::asset_exists(MARKET_ASSET));
        assert_eq!(crate::IndestructibleAssets::<Runtime>::get(), vec![]);
        assert_eq!(crate::DestroyAssets::<Runtime>::get(), vec![]);

        let consumed_weight = available_weight - 3u64 * 3u64 * DESTROY_WEIGHT;
        assert_eq!(remaining_weight, consumed_weight);
    })
}

#[test]
fn destroys_assets_partially_properly() {
    ExtBuilder::default().build().execute_with(|| {
        let assets_raw = [(CAMPAIGN_ASSET, None), (CUSTOM_ASSET, None), (MARKET_ASSET, None)];
        let assets = BTreeMap::from_iter(assets_raw.to_vec());

        for (asset, _) in &assets_raw[..] {
            assert_ok!(AssetRouter::create(*asset, ALICE, true, CAMPAIGN_ASSET_MIN_BALANCE));
        }

        assert_ok!(managed_destroy_multi_transactional(assets.clone()));
        assert_eq!(crate::DestroyAssets::<Runtime>::get().len(), 3);

        let available_weight = DESTROY_WEIGHT * 3;
        // Make on_idle only partially delete the first asset
        let _ = AssetRouter::on_idle(0, available_weight - 2u32 * DESTROY_WEIGHT);
        assert_eq!(crate::DestroyAssets::<Runtime>::get().len(), 3);

        // Now delete each asset one by one by supplying exactly the required weight
        let _ = AssetRouter::on_idle(0, available_weight);
        assert_eq!(crate::DestroyAssets::<Runtime>::get().len(), 2);

        let _ = AssetRouter::on_idle(0, available_weight);
        assert_eq!(crate::DestroyAssets::<Runtime>::get().len(), 1);

        let _ = AssetRouter::on_idle(0, available_weight);
        assert_eq!(crate::DestroyAssets::<Runtime>::get().len(), 0);

        assert!(!AssetRouter::asset_exists(CAMPAIGN_ASSET));
        assert!(!AssetRouter::asset_exists(CUSTOM_ASSET));
        assert!(!AssetRouter::asset_exists(MARKET_ASSET));
    })
}

#[test]
fn properly_handles_indestructible_assets() {
    ExtBuilder::default().build().execute_with(|| {
        let assets_raw = vec![CAMPAIGN_ASSET, CUSTOM_ASSET, MARKET_ASSET];
        let mut destroy_assets = crate::DestroyAssets::<Runtime>::get();
        let available_weight = 1_000_000_000.into();

        for asset in assets_raw {
            destroy_assets.force_push(AssetInDestruction::new(asset));
        }

        destroy_assets.sort();

        let setup_state = || {
            assert_ok!(AssetRouter::create(
                *destroy_assets[0].asset(),
                ALICE,
                true,
                CAMPAIGN_ASSET_MIN_BALANCE
            ));
            assert_ok!(AssetRouter::create(
                *destroy_assets[2].asset(),
                ALICE,
                true,
                CAMPAIGN_ASSET_MIN_BALANCE
            ));
            assert_ok!(AssetRouter::start_destroy(*destroy_assets[0].asset(), None));
            assert_ok!(AssetRouter::start_destroy(*destroy_assets[2].asset(), None));
        };

        // [1] Asset is indestructible and not in Finalization state,
        // i.e. weight consumption bounded but unknown.
        setup_state();
        crate::DestroyAssets::<Runtime>::put(destroy_assets.clone());
        assert_eq!(crate::DestroyAssets::<Runtime>::get().len(), 3);
        let remaining_weight = AssetRouter::on_idle(0, available_weight);
        assert_eq!(crate::DestroyAssets::<Runtime>::get().len(), 1);
        assert_eq!(remaining_weight, 0.into());

        // Destroy remaining assets
        let _ = AssetRouter::on_idle(0, available_weight);
        assert_eq!(crate::DestroyAssets::<Runtime>::get().len(), 0);
        assert_eq!(crate::IndestructibleAssets::<Runtime>::get().len(), 1);

        assert!(!AssetRouter::asset_exists(CAMPAIGN_ASSET));
        assert!(!AssetRouter::asset_exists(CUSTOM_ASSET));
        assert!(!AssetRouter::asset_exists(MARKET_ASSET));

        // [2] Asset is indestructible and in Finalization state,
        // i.e. weight consumption bounded and known.
        crate::DestroyAssets::<Runtime>::kill();
        crate::IndestructibleAssets::<Runtime>::kill();
        setup_state();
        destroy_assets[1].transit_state();
        destroy_assets[1].transit_state();
        crate::DestroyAssets::<Runtime>::put(destroy_assets);
        assert_eq!(crate::DestroyAssets::<Runtime>::get().len(), 3);
        let remaining_weight = AssetRouter::on_idle(0, available_weight);
        let consumed_weight = available_weight - 2u32 * 3u32 * DESTROY_WEIGHT - DESTROY_WEIGHT;
        assert_eq!(remaining_weight, consumed_weight);
        assert_eq!(crate::DestroyAssets::<Runtime>::get().len(), 0);
        assert_eq!(crate::IndestructibleAssets::<Runtime>::get().len(), 1);

        assert!(!AssetRouter::asset_exists(CAMPAIGN_ASSET));
        assert!(!AssetRouter::asset_exists(CUSTOM_ASSET));
        assert!(!AssetRouter::asset_exists(MARKET_ASSET));
    })
}
