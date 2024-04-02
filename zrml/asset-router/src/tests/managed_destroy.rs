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
use crate::{
    AssetInDestruction, DestroyAssets, DestructionState, IndestructibleAssets, Weight,
    MAX_ASSET_DESTRUCTIONS_PER_BLOCK, MIN_ON_IDLE_EXTRA_COMPUTATION_WEIGHT,
};
use frame_support::{
    traits::{tokens::fungibles::Inspect, Get},
    weights::RuntimeDbWeight,
    BoundedVec,
};
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
        assert_eq!(DestroyAssets::<Runtime>::get(), vec![campaign_asset]);

        assert_ok!(AssetRouter::create(CUSTOM_ASSET, ALICE, true, CUSTOM_ASSET_MIN_BALANCE));
        assert_ok!(AssetRouter::managed_destroy(CUSTOM_ASSET, None));
        let mut expected = vec![campaign_asset, custom_asset];
        expected.sort();
        assert_eq!(DestroyAssets::<Runtime>::get(), expected);

        IndestructibleAssets::<Runtime>::put(BoundedVec::truncate_from(vec![
            CAMPAIGN_ASSET,
            CUSTOM_ASSET,
        ]));
        DestroyAssets::<Runtime>::kill();
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
        assert_eq!(DestroyAssets::<Runtime>::get(), expected);

        IndestructibleAssets::<Runtime>::put(BoundedVec::truncate_from(vec![
            CAMPAIGN_ASSET,
            CUSTOM_ASSET,
        ]));
        DestroyAssets::<Runtime>::kill();
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

        assert_ok!(
            pallet_assets::Call::<Runtime, CampaignAssetsInstance>::approve_transfer {
                id: CAMPAIGN_ASSET_INTERNAL.into(),
                delegate: BOB,
                amount: 1
            }
            .dispatch_bypass_filter(Signed(ALICE).into())
        );

        assert_ok!(managed_destroy_multi_transactional(assets.clone()));
        assert_eq!(DestroyAssets::<Runtime>::get().len(), 3);

        let available_weight = (2 * MIN_ON_IDLE_EXTRA_COMPUTATION_WEIGHT).into();
        let remaining_weight = AssetRouter::on_idle(0, available_weight);
        assert!(!AssetRouter::asset_exists(CAMPAIGN_ASSET));
        assert!(!AssetRouter::asset_exists(CUSTOM_ASSET));
        assert!(!AssetRouter::asset_exists(MARKET_ASSET));
        assert_eq!(IndestructibleAssets::<Runtime>::get(), vec![]);
        assert_eq!(DestroyAssets::<Runtime>::get(), vec![]);

        let mut consumed_weight = available_weight - 3u64 * 3u64 * DESTROY_WEIGHT;
        // Consider safety buffer for extra execution time and storage proof size
        consumed_weight = consumed_weight
            .saturating_sub(Weight::from_parts(MIN_ON_IDLE_EXTRA_COMPUTATION_WEIGHT, 45_824));
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
        assert_eq!(DestroyAssets::<Runtime>::get().len(), 3);

        let mut available_weight: Weight =
            Weight::from_all(MIN_ON_IDLE_EXTRA_COMPUTATION_WEIGHT) + 2u64 * DESTROY_WEIGHT;
        // Make on_idle only partially delete the first asset
        let _ = AssetRouter::on_idle(0, available_weight);
        assert_eq!(DestroyAssets::<Runtime>::get().len(), 3);

        // Now delete each asset one by one by supplying exactly the required weight
        available_weight = Weight::from_all(MIN_ON_IDLE_EXTRA_COMPUTATION_WEIGHT) + DESTROY_WEIGHT;
        let _ = AssetRouter::on_idle(0, available_weight);
        assert_eq!(DestroyAssets::<Runtime>::get().len(), 2);

        available_weight =
            Weight::from_all(MIN_ON_IDLE_EXTRA_COMPUTATION_WEIGHT) + 3u64 * DESTROY_WEIGHT;
        let _ = AssetRouter::on_idle(0, available_weight);
        assert_eq!(DestroyAssets::<Runtime>::get().len(), 1);

        let _ = AssetRouter::on_idle(0, available_weight);
        assert_eq!(DestroyAssets::<Runtime>::get().len(), 0);

        assert!(!AssetRouter::asset_exists(CAMPAIGN_ASSET));
        assert!(!AssetRouter::asset_exists(CUSTOM_ASSET));
        assert!(!AssetRouter::asset_exists(MARKET_ASSET));
    })
}

#[test]
fn properly_handles_indestructible_assets() {
    ExtBuilder::default().build().execute_with(|| {
        let assets_raw = vec![CAMPAIGN_ASSET, CUSTOM_ASSET, MARKET_ASSET];
        let mut destroy_assets = DestroyAssets::<Runtime>::get();
        let available_weight = (4 * MIN_ON_IDLE_EXTRA_COMPUTATION_WEIGHT).into();

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
        DestroyAssets::<Runtime>::put(destroy_assets.clone());
        assert_eq!(DestroyAssets::<Runtime>::get().len(), 3);
        let remaining_weight = AssetRouter::on_idle(0, available_weight);
        assert_eq!(DestroyAssets::<Runtime>::get().len(), 1);
        assert_eq!(remaining_weight, 0.into());

        // Destroy remaining assets
        let _ = AssetRouter::on_idle(0, available_weight);
        assert_eq!(DestroyAssets::<Runtime>::get().len(), 0);
        assert_eq!(IndestructibleAssets::<Runtime>::get().len(), 1);

        assert!(!AssetRouter::asset_exists(CAMPAIGN_ASSET));
        assert!(!AssetRouter::asset_exists(CUSTOM_ASSET));
        assert!(!AssetRouter::asset_exists(MARKET_ASSET));

        // [2] Asset is indestructible and in Finalization state,
        // i.e. weight consumption bounded and known.
        DestroyAssets::<Runtime>::kill();
        IndestructibleAssets::<Runtime>::kill();
        setup_state();
        destroy_assets[1].transit_state();
        destroy_assets[1].transit_state();
        DestroyAssets::<Runtime>::put(destroy_assets);
        assert_eq!(DestroyAssets::<Runtime>::get().len(), 3);
        let remaining_weight = AssetRouter::on_idle(0, available_weight);
        let mut consumed_weight = available_weight - 2u32 * 3u32 * DESTROY_WEIGHT - DESTROY_WEIGHT;
        // Consider safety buffer for extra execution time and storage proof size
        consumed_weight = consumed_weight
            .saturating_sub(Weight::from_parts(MIN_ON_IDLE_EXTRA_COMPUTATION_WEIGHT, 45_824));
        assert_eq!(remaining_weight, consumed_weight);
        assert_eq!(DestroyAssets::<Runtime>::get().len(), 0);
        assert_eq!(IndestructibleAssets::<Runtime>::get().len(), 1);

        assert!(!AssetRouter::asset_exists(CAMPAIGN_ASSET));
        assert!(!AssetRouter::asset_exists(CUSTOM_ASSET));
        assert!(!AssetRouter::asset_exists(MARKET_ASSET));
    })
}

#[test]
fn does_not_execute_on_insufficient_weight() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(AssetRouter::create(CAMPAIGN_ASSET, ALICE, true, CAMPAIGN_ASSET_MIN_BALANCE));
        assert_ok!(AssetRouter::managed_destroy(CAMPAIGN_ASSET, None));
        assert_eq!(DestroyAssets::<Runtime>::get().len(), 1);

        let db_weight: RuntimeDbWeight = <Runtime as frame_system::Config>::DbWeight::get();
        let mut available_weight = Weight::from_parts(MIN_ON_IDLE_EXTRA_COMPUTATION_WEIGHT, 45_824)
            + db_weight.reads(1)
            - 1u64.into();
        let mut remaining_weight = AssetRouter::on_idle(0, available_weight);
        assert_eq!(available_weight, remaining_weight);
        assert_eq!(DestroyAssets::<Runtime>::get().len(), 1);

        available_weight += Weight::from_all(1u64) + db_weight.writes(1);
        let mut remaining_weight_expected: Weight = 0u64.into();
        remaining_weight = AssetRouter::on_idle(0, available_weight);
        assert_eq!(remaining_weight_expected, remaining_weight);
        assert_eq!(DestroyAssets::<Runtime>::get().len(), 1);

        remaining_weight_expected = 1u64.into();
        available_weight += 3u64 * DESTROY_WEIGHT + remaining_weight_expected;
        remaining_weight = AssetRouter::on_idle(0, available_weight);
        assert_eq!(remaining_weight_expected, remaining_weight);
        assert_eq!(DestroyAssets::<Runtime>::get().len(), 0);
    })
}

#[test]
fn does_skip_and_remove_assets_in_invalid_state() {
    ExtBuilder::default().build().execute_with(|| {
        let mut campaign_asset = AssetInDestruction::new(CAMPAIGN_ASSET);
        campaign_asset.transit_state();
        campaign_asset.transit_state();
        assert_eq!(*campaign_asset.transit_state().unwrap(), DestructionState::Destroyed);
        let mut custom_asset = AssetInDestruction::new(CUSTOM_ASSET);
        custom_asset.transit_indestructible();

        let assets_raw = BoundedVec::truncate_from(vec![campaign_asset, custom_asset]);
        DestroyAssets::<Runtime>::put(assets_raw);
        let db_weight: RuntimeDbWeight = <Runtime as frame_system::Config>::DbWeight::get();
        let available_weight = Weight::from_parts(MIN_ON_IDLE_EXTRA_COMPUTATION_WEIGHT, 45_824)
            + db_weight.reads_writes(1, 1)
            + DESTROY_WEIGHT;

        let remaining_weight = AssetRouter::on_idle(0, available_weight);
        // No destroy routine was called
        assert_eq!(remaining_weight, DESTROY_WEIGHT);
        // Asset in invalid states got removed
        assert_eq!(DestroyAssets::<Runtime>::get().len(), 0);
    });
}

#[test]
#[should_panic(expected = "Destruction outer loop iteration guard triggered")]
fn does_trigger_on_idle_outer_loop_safety_guard() {
    ExtBuilder::default().build().execute_with(|| {
        for asset_num in 0..=MAX_ASSET_DESTRUCTIONS_PER_BLOCK {
            let asset = Assets::CampaignAsset(asset_num as u128);
            assert_ok!(AssetRouter::create(asset, ALICE, true, CAMPAIGN_ASSET_MIN_BALANCE));
            assert_ok!(AssetRouter::managed_destroy(asset, None));
        }

        let db_weight: RuntimeDbWeight = <Runtime as frame_system::Config>::DbWeight::get();
        let available_weight = Weight::from_parts(MIN_ON_IDLE_EXTRA_COMPUTATION_WEIGHT, 45_824)
            + db_weight.reads(1)
            + DESTROY_WEIGHT * 3 * (MAX_ASSET_DESTRUCTIONS_PER_BLOCK + 1) as u64;

        let _ = AssetRouter::on_idle(0, available_weight);
    });
}
