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
use test_case::test_case;

// Assets <> BaseAssetClass
#[test_case(
    Asset::<MarketId>::CampaignAsset(7),
    BaseAssetClass::CampaignAsset(7);
    "campaign_asset"
)]
#[test_case(
    Asset::<MarketId>::ForeignAsset(7),
    BaseAssetClass::ForeignAsset(7);
    "foreign_asset"
)]
#[test_case(
    Asset::<MarketId>::Ztg,
    BaseAssetClass::Ztg;
    "ztg"
)]
fn index_matching_works_for_base_assets(old_asset: Asset<MarketId>, new_asset: BaseAssetClass) {
    let old_asset_encoded: Vec<u8> = old_asset.encode();
    let new_asset_decoded =
        <BaseAssetClass as Decode>::decode(&mut old_asset_encoded.as_slice()).unwrap();
    assert_eq!(new_asset_decoded, new_asset);
}

// Assets <> CurrencyClass
#[test_case(
    Asset::<MarketId>::CategoricalOutcome(7, 8),
    CurrencyClass::<MarketId>::CategoricalOutcome(7, 8);
    "categorical_outcome"
)]
#[test_case(
    Asset::<MarketId>::ScalarOutcome(7, ScalarPosition::Long),
    CurrencyClass::<MarketId>::ScalarOutcome(7, ScalarPosition::Long);
    "scalar_outcome"
)]
#[test_case(
    Asset::<MarketId>::PoolShare(7),
    CurrencyClass::<MarketId>::PoolShare(7);
    "pool_share"
)]
#[test_case(
    Asset::<MarketId>::ParimutuelShare(7, 8),
    CurrencyClass::<MarketId>::ParimutuelShare(7, 8);
    "parimutuel_share"
)]
#[test_case(
    Asset::<MarketId>::ForeignAsset(7),
    CurrencyClass::<MarketId>::ForeignAsset(7);
    "foreign_asset"
)]
fn index_matching_works_for_currencies(
    old_asset: Asset<MarketId>,
    new_asset: CurrencyClass<MarketId>,
) {
    let old_asset_encoded: Vec<u8> = old_asset.encode();
    let new_asset_decoded =
        <CurrencyClass<MarketId> as Decode>::decode(&mut old_asset_encoded.as_slice()).unwrap();
    assert_eq!(new_asset_decoded, new_asset);
}

// Assets <> MarketAssetClass
#[test_case(
    Asset::<MarketId>::CategoricalOutcome(7, 8),
    MarketAssetClass::<MarketId>::CategoricalOutcome(7, 8);
    "categorical_outcome"
)]
#[test_case(
    Asset::<MarketId>::ScalarOutcome(7, ScalarPosition::Long),
    MarketAssetClass::<MarketId>::ScalarOutcome(7, ScalarPosition::Long);
    "scalar_outcome"
)]
#[test_case(
    Asset::<MarketId>::PoolShare(7),
    MarketAssetClass::<MarketId>::PoolShare(7);
    "pool_share"
)]
#[test_case(
    Asset::<MarketId>::ParimutuelShare(7, 8),
    MarketAssetClass::<MarketId>::ParimutuelShare(7, 8);
    "parimutuel_share"
)]
fn index_matching_works_for_market_assets(
    old_asset: Asset<MarketId>,
    new_asset: MarketAssetClass<MarketId>,
) {
    let old_asset_encoded: Vec<u8> = old_asset.encode();
    let new_asset_decoded =
        <MarketAssetClass<MarketId> as Decode>::decode(&mut old_asset_encoded.as_slice()).unwrap();
    assert_eq!(new_asset_decoded, new_asset);
}

// Assets <> ParimutuelAssetClass
#[test_case(
    Asset::<MarketId>::ParimutuelShare(7, 8),
    ParimutuelAssetClass::Share(7, 8);
    "parimutuel_share"
)]
fn index_matching_works_for_parimutuel_assets(
    old_asset: Asset<MarketId>,
    new_asset: ParimutuelAssetClass<MarketId>,
) {
    let old_asset_encoded: Vec<u8> = old_asset.encode();
    let new_asset_decoded =
        <ParimutuelAssetClass<MarketId> as Decode>::decode(&mut old_asset_encoded.as_slice())
            .unwrap();
    assert_eq!(new_asset_decoded, new_asset);
}

// Assets <> XcmAssetClass
#[test_case(
    Asset::<MarketId>::ForeignAsset(7),
    XcmAssetClass::ForeignAsset(7);
    "foreign_asset"
)]
#[test_case(
    Asset::<MarketId>::Ztg,
    XcmAssetClass::Ztg;
    "ztg"
)]
fn index_matching_works_for_xcm_assets(old_asset: Asset<MarketId>, new_asset: XcmAssetClass) {
    let old_asset_encoded: Vec<u8> = old_asset.encode();
    let new_asset_decoded =
        <XcmAssetClass as Decode>::decode(&mut old_asset_encoded.as_slice()).unwrap();
    assert_eq!(new_asset_decoded, new_asset);
}
