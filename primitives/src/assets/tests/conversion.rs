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

// Assets <> MarketAssetClass
#[test_case(
    Asset::<MarketId>::CategoricalOutcome(7, 7),
    MarketAssetClass::<MarketId>::OldCategoricalOutcome(7, 7);
    "categorical_outcome"
)]
#[test_case(
    Asset::<MarketId>::ScalarOutcome(7, ScalarPosition::Long),
    MarketAssetClass::<MarketId>::OldScalarOutcome(7, ScalarPosition::Long);
    "scalar_outcome"
)]
#[test_case(
    Asset::<MarketId>::PoolShare(7),
    MarketAssetClass::<MarketId>::OldPoolShare(7);
    "pool_share"
)]
#[test_case(
    Asset::<MarketId>::ParimutuelShare(7, 7),
    MarketAssetClass::<MarketId>::OldParimutuelShare(7, 7);
    "parimutuel_share"
)]
#[test_case(
    Asset::<MarketId>::NewCategoricalOutcome(7, 7),
    MarketAssetClass::<MarketId>::CategoricalOutcome(7, 7);
    "new_categorical_outcome"
)]
#[test_case(
    Asset::<MarketId>::NewScalarOutcome(7, ScalarPosition::Long),
    MarketAssetClass::<MarketId>::ScalarOutcome(7, ScalarPosition::Long);
    "new_calar_outcome"
)]
#[test_case(
    Asset::<MarketId>::NewPoolShare(7),
    MarketAssetClass::<MarketId>::PoolShare(7);
    "new_pool_share"
)]
#[test_case(
    Asset::<MarketId>::NewParimutuelShare(7, 7),
    MarketAssetClass::<MarketId>::ParimutuelShare(7, 7);
    "new_parimutuel_share"
)]
fn from_all_assets_to_market_assets(
    old_asset: Asset<MarketId>,
    new_asset: MarketAssetClass<MarketId>,
) {
    let new_asset_converted: MarketAssetClass<MarketId> = old_asset.try_into().unwrap();
    assert_eq!(new_asset, new_asset_converted);
}

#[test_case(
    MarketAssetClass::<MarketId>::OldCategoricalOutcome(7, 7),
    Asset::<MarketId>::CategoricalOutcome(7, 7);
    "categorical_outcome"
)]
#[test_case(
    MarketAssetClass::<MarketId>::OldScalarOutcome(7, ScalarPosition::Long),
    Asset::<MarketId>::ScalarOutcome(7, ScalarPosition::Long);
    "scalar_outcome"
)]
#[test_case(
    MarketAssetClass::<MarketId>::OldPoolShare(7),
    Asset::<MarketId>::PoolShare(7);
    "pool_share"
)]
#[test_case(
    MarketAssetClass::<MarketId>::OldParimutuelShare(7, 7),
    Asset::<MarketId>::ParimutuelShare(7, 7);
    "parimutuel_share"
)]
#[test_case(
    MarketAssetClass::<MarketId>::CategoricalOutcome(7, 7),
    Asset::<MarketId>::NewCategoricalOutcome(7, 7);
    "new_categorical_outcome"
)]
#[test_case(
    MarketAssetClass::<MarketId>::ScalarOutcome(7, ScalarPosition::Long),
    Asset::<MarketId>::NewScalarOutcome(7, ScalarPosition::Long);
    "new_calar_outcome"
)]
#[test_case(
    MarketAssetClass::<MarketId>::PoolShare(7),
    Asset::<MarketId>::NewPoolShare(7);
    "new_pool_share"
)]
#[test_case(
    MarketAssetClass::<MarketId>::ParimutuelShare(7, 7),
    Asset::<MarketId>::NewParimutuelShare(7, 7);
    "new_parimutuel_share"
)]
fn from_market_assets_to_all_assets(
    old_asset: MarketAssetClass<MarketId>,
    new_asset: Asset<MarketId>,
) {
    let new_asset_converted: Asset<MarketId> = old_asset.into();
    assert_eq!(new_asset, new_asset_converted);
}

// Assets <> CurrencyClass
#[test_case(
    Asset::<MarketId>::CategoricalOutcome(7, 7),
    CurrencyClass::<MarketId>::OldCategoricalOutcome(7, 7);
    "categorical_outcome"
)]
#[test_case(
    Asset::<MarketId>::ScalarOutcome(7, ScalarPosition::Long),
    CurrencyClass::<MarketId>::OldScalarOutcome(7, ScalarPosition::Long);
    "scalar_outcome"
)]
#[test_case(
    Asset::<MarketId>::PoolShare(7),
    CurrencyClass::<MarketId>::OldPoolShare(7);
    "pool_share"
)]
#[test_case(
    Asset::<MarketId>::ParimutuelShare(7, 7),
    CurrencyClass::<MarketId>::OldParimutuelShare(7, 7);
    "parimutuel_share"
)]
#[test_case(
    Asset::<MarketId>::ForeignAsset(7),
    CurrencyClass::<MarketId>::ForeignAsset(7);
    "foreign_asset"
)]
fn from_all_assets_to_currencies(old_asset: Asset<MarketId>, new_asset: CurrencyClass<MarketId>) {
    let new_asset_converted: CurrencyClass<MarketId> = old_asset.try_into().unwrap();
    assert_eq!(new_asset, new_asset_converted);
}

#[test_case(
    CurrencyClass::<MarketId>::OldCategoricalOutcome(7, 7),
    Asset::<MarketId>::CategoricalOutcome(7, 7);
    "categorical_outcome"
)]
#[test_case(
    CurrencyClass::<MarketId>::OldScalarOutcome(7, ScalarPosition::Long),
    Asset::<MarketId>::ScalarOutcome(7, ScalarPosition::Long);
    "scalar_outcome"
)]
#[test_case(
    CurrencyClass::<MarketId>::OldPoolShare(7),
    Asset::<MarketId>::PoolShare(7);
    "pool_share"
)]
#[test_case(
    CurrencyClass::<MarketId>::OldParimutuelShare(7, 7),
    Asset::<MarketId>::ParimutuelShare(7, 7);
    "parimutuel_share"
)]
#[test_case(
    CurrencyClass::<MarketId>::ForeignAsset(7),
    Asset::<MarketId>::ForeignAsset(7);
    "foreign_asset"
)]
fn from_currencies_to_all_assets(old_asset: CurrencyClass<MarketId>, new_asset: Asset<MarketId>) {
    let new_asset_converted: Asset<MarketId> = old_asset.into();
    assert_eq!(new_asset, new_asset_converted);
}

// Assets <> CampaignAssetClass
#[test]
fn from_all_assets_to_campaign_assets() {
    let old_asset = Asset::<MarketId>::CampaignAssetClass(7);
    let new_asset = CampaignAssetClass(7);

    let new_asset_converted: CampaignAssetClass = old_asset.try_into().unwrap();
    assert_eq!(new_asset, new_asset_converted);
}

#[test]
fn from_campaign_assets_to_all_assets() {
    let old_asset = CampaignAssetClass(7);
    let new_asset = Asset::<MarketId>::CampaignAssetClass(7);
    let new_asset_converted: Asset<MarketId> = old_asset.into();
    assert_eq!(new_asset, new_asset_converted);
}

// Assets <> CustomAssetClass
#[test]
fn from_all_assets_to_custom_assets() {
    let old_asset = Asset::<MarketId>::CustomAssetClass(7);
    let new_asset = CustomAssetClass(7);

    let new_asset_converted: CustomAssetClass = old_asset.try_into().unwrap();
    assert_eq!(new_asset, new_asset_converted);
}

#[test]
fn from_custom_assets_to_all_assets() {
    let old_asset = CampaignAssetClass(7);
    let new_asset = Asset::<MarketId>::CampaignAssetClass(7);
    let new_asset_converted: Asset<MarketId> = old_asset.into();
    assert_eq!(new_asset, new_asset_converted);
}
