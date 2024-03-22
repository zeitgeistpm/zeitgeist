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
fn from_all_assets_to_market_assets(
    old_asset: Asset<MarketId>,
    new_asset: MarketAssetClass<MarketId>,
) {
    let new_asset_converted: MarketAssetClass<MarketId> = old_asset.try_into().unwrap();
    assert_eq!(new_asset, new_asset_converted);
}

#[test_case(Asset::<MarketId>::Ztg; "ztg")]
#[test_case(Asset::<MarketId>::ForeignAsset(7); "foreign_asset")]
#[test_case(Asset::<MarketId>::CampaignAsset(7); "campaign_asset")]
#[test_case(Asset::<MarketId>::CustomAsset(7); "custom_asset")]
fn from_all_assets_to_market_assets_fails(asset: Asset<MarketId>) {
    assert!(MarketAssetClass::<MarketId>::try_from(asset).is_err());
}

#[test_case(
    MarketAssetClass::<MarketId>::CategoricalOutcome(7, 8),
    Asset::<MarketId>::CategoricalOutcome(7, 8);
    "categorical_outcome"
)]
#[test_case(
    MarketAssetClass::<MarketId>::ScalarOutcome(7, ScalarPosition::Long),
    Asset::<MarketId>::ScalarOutcome(7, ScalarPosition::Long);
    "scalar_outcome"
)]
#[test_case(
    MarketAssetClass::<MarketId>::PoolShare(7),
    Asset::<MarketId>::PoolShare(7);
    "pool_share"
)]
#[test_case(
    MarketAssetClass::<MarketId>::ParimutuelShare(7, 8),
    Asset::<MarketId>::ParimutuelShare(7, 8);
    "parimutuel_share"
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
fn from_all_assets_to_currencies(old_asset: Asset<MarketId>, new_asset: CurrencyClass<MarketId>) {
    let new_asset_converted: CurrencyClass<MarketId> = old_asset.try_into().unwrap();
    assert_eq!(new_asset, new_asset_converted);
}

#[test_case(Asset::<MarketId>::Ztg; "ztg")]
#[test_case(Asset::<MarketId>::CampaignAsset(7); "campaign_asset")]
#[test_case(Asset::<MarketId>::CustomAsset(7); "custom_asset")]
fn from_all_assets_to_currencies_fails(asset: Asset<MarketId>) {
    assert!(CurrencyClass::<MarketId>::try_from(asset).is_err());
}

#[test_case(
    CurrencyClass::<MarketId>::CategoricalOutcome(7, 8),
    Asset::<MarketId>::CategoricalOutcome(7, 8);
    "categorical_outcome"
)]
#[test_case(
    CurrencyClass::<MarketId>::ScalarOutcome(7, ScalarPosition::Long),
    Asset::<MarketId>::ScalarOutcome(7, ScalarPosition::Long);
    "scalar_outcome"
)]
#[test_case(
    CurrencyClass::<MarketId>::PoolShare(7),
    Asset::<MarketId>::PoolShare(7);
    "pool_share"
)]
#[test_case(
    CurrencyClass::<MarketId>::ParimutuelShare(7, 8),
    Asset::<MarketId>::ParimutuelShare(7, 8);
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
    let old_asset = Asset::<MarketId>::CampaignAsset(7);
    let new_asset = CampaignAssetClass(7);

    let new_asset_converted: CampaignAssetClass = old_asset.try_into().unwrap();
    assert_eq!(new_asset, new_asset_converted);
}

#[test_case(Asset::<MarketId>::CategoricalOutcome(7, 8); "categorical_outcome")]
#[test_case(Asset::<MarketId>::ScalarOutcome(7, ScalarPosition::Long); "scalar_outcome")]
#[test_case(Asset::<MarketId>::PoolShare(7); "pool_share")]
#[test_case(Asset::<MarketId>::Ztg; "ztg")]
#[test_case(Asset::<MarketId>::ForeignAsset(7); "foreign_asset")]
#[test_case(Asset::<MarketId>::ParimutuelShare(7, 8); "parimutuel_share")]
#[test_case(Asset::<MarketId>::CustomAsset(7); "custom_asset")]
fn from_all_assets_to_campaign_assets_fails(asset: Asset<MarketId>) {
    assert!(CampaignAssetClass::try_from(asset).is_err());
}

#[test]
fn from_campaign_assets_to_all_assets() {
    let old_asset = CampaignAssetClass(7);
    let new_asset = Asset::<MarketId>::CampaignAsset(7);
    let new_asset_converted: Asset<MarketId> = old_asset.into();
    assert_eq!(new_asset, new_asset_converted);
}

// Assets <> CustomAssetClass
#[test]
fn from_all_assets_to_custom_assets() {
    let old_asset = Asset::<MarketId>::CustomAsset(7);
    let new_asset = CustomAssetClass(7);

    let new_asset_converted: CustomAssetClass = old_asset.try_into().unwrap();
    assert_eq!(new_asset, new_asset_converted);
}

#[test_case(Asset::<MarketId>::CategoricalOutcome(7, 8); "categorical_outcome")]
#[test_case(Asset::<MarketId>::ScalarOutcome(7, ScalarPosition::Long); "scalar_outcome")]
#[test_case(Asset::<MarketId>::PoolShare(7); "pool_share")]
#[test_case(Asset::<MarketId>::Ztg; "ztg")]
#[test_case(Asset::<MarketId>::ForeignAsset(7); "foreign_asset")]
#[test_case(Asset::<MarketId>::ParimutuelShare(7, 8); "parimutuel_share")]
#[test_case(Asset::<MarketId>::CampaignAsset(7); "campaign_asset")]
fn from_all_assets_to_custom_assets_fails(asset: Asset<MarketId>) {
    assert!(CustomAssetClass::try_from(asset).is_err());
}

#[test]
fn from_custom_assets_to_all_assets() {
    let old_asset = CampaignAssetClass(7);
    let new_asset = Asset::<MarketId>::CampaignAsset(7);
    let new_asset_converted: Asset<MarketId> = old_asset.into();
    assert_eq!(new_asset, new_asset_converted);
}

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
fn from_all_assets_to_base_assets(old_asset: Asset<MarketId>, new_asset: BaseAssetClass) {
    let new_asset_converted: BaseAssetClass = old_asset.try_into().unwrap();
    assert_eq!(new_asset, new_asset_converted);
}

#[test_case(Asset::<MarketId>::CategoricalOutcome(7, 8); "categorical_outcome")]
#[test_case(Asset::<MarketId>::ScalarOutcome(7, ScalarPosition::Long); "scalar_outcome")]
#[test_case(Asset::<MarketId>::PoolShare(7); "pool_share")]
#[test_case(Asset::<MarketId>::ParimutuelShare(7, 8); "parimutuel_share")]
#[test_case(Asset::<MarketId>::CustomAsset(7); "custom_asset")]
fn from_all_assets_to_base_assets_fails(asset: Asset<MarketId>) {
    assert!(BaseAssetClass::try_from(asset).is_err());
}

#[test_case(
    BaseAssetClass::CampaignAsset(7),
    Asset::<MarketId>::CampaignAsset(7);
    "campaign_asset"
)]
#[test_case(
    BaseAssetClass::ForeignAsset(7),
    Asset::<MarketId>::ForeignAsset(7);
    "foreign_asset"
)]
#[test_case(
    BaseAssetClass::Ztg,
    Asset::<MarketId>::Ztg;
    "ztg"
)]
fn from_base_assets_to_all_assets(old_asset: BaseAssetClass, new_asset: Asset<MarketId>) {
    let new_asset_converted: Asset<MarketId> = old_asset.into();
    assert_eq!(new_asset, new_asset_converted);
}

// Assets <> ParimutuelAssetClass
#[test_case(
    Asset::<MarketId>::ParimutuelShare(7, 8),
    ParimutuelAssetClass::<MarketId>::Share(7, 8);
    "parimutuel_share"
)]
fn from_all_assets_to_parimutuel_assets(
    old_asset: Asset<MarketId>,
    new_asset: ParimutuelAssetClass<MarketId>,
) {
    let new_asset_converted: ParimutuelAssetClass<MarketId> = old_asset.try_into().unwrap();
    assert_eq!(new_asset, new_asset_converted);
}

#[test_case(Asset::<MarketId>::CategoricalOutcome(7, 8); "categorical_outcome")]
#[test_case(Asset::<MarketId>::ScalarOutcome(7, ScalarPosition::Long); "scalar_outcome")]
#[test_case(Asset::<MarketId>::PoolShare(7); "pool_share")]
#[test_case(Asset::<MarketId>::Ztg; "ztg")]
#[test_case(Asset::<MarketId>::ForeignAsset(7); "foreign_asset")]
#[test_case(Asset::<MarketId>::CampaignAsset(7); "campaign_asset")]
#[test_case(Asset::<MarketId>::CustomAsset(7); "custom_asset")]
fn from_all_assets_to_parimutuel_assets_fails(asset: Asset<MarketId>) {
    assert!(ParimutuelAssetClass::<MarketId>::try_from(asset).is_err());
}

#[test_case(
    ParimutuelAssetClass::<MarketId>::Share(7, 8),
    Asset::<MarketId>::ParimutuelShare(7, 8);
    "parimutuel_share"
)]
fn from_parimutuel_assets_to_all_assets(
    old_asset: ParimutuelAssetClass<MarketId>,
    new_asset: Asset<MarketId>,
) {
    let new_asset_converted: Asset<MarketId> = old_asset.into();
    assert_eq!(new_asset, new_asset_converted);
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
fn from_all_assets_to_xcm_assets(old_asset: Asset<MarketId>, new_asset: XcmAssetClass) {
    let new_asset_converted: XcmAssetClass = old_asset.try_into().unwrap();
    assert_eq!(new_asset, new_asset_converted);
}

#[test_case(Asset::<MarketId>::CategoricalOutcome(7, 8); "categorical_outcome")]
#[test_case(Asset::<MarketId>::ScalarOutcome(7, ScalarPosition::Long); "scalar_outcome")]
#[test_case(Asset::<MarketId>::PoolShare(7); "pool_share")]
#[test_case(Asset::<MarketId>::CampaignAsset(7); "campaign_asset")]
#[test_case(Asset::<MarketId>::CustomAsset(7); "custom_asset")]
fn from_all_assets_to_xcm_assets_fails(asset: Asset<MarketId>) {
    assert!(XcmAssetClass::try_from(asset).is_err());
}

#[test_case(
    XcmAssetClass::ForeignAsset(7),
    Asset::<MarketId>::ForeignAsset(7);
    "foreign_asset"
)]
#[test_case(
    XcmAssetClass::Ztg,
    Asset::<MarketId>::Ztg;
    "ztg"
)]
fn from_xcm_assets_to_all_assets(old_asset: XcmAssetClass, new_asset: Asset<MarketId>) {
    let new_asset_converted: Asset<MarketId> = old_asset.into();
    assert_eq!(new_asset, new_asset_converted);
}

// CampaignAssetId <> CampaignAssetClass
#[test]
fn from_campaign_asset_id_to_campaign_asset() {
    let campaign_asset_id = Compact(7);
    let campaign_asset = CampaignAssetClass::from(campaign_asset_id);
    let campaign_asset_id_converted = campaign_asset.into();
    assert_eq!(campaign_asset_id, campaign_asset_id_converted);
}

// CustomAssetId <> CustomAssetClass
#[test]
fn from_custom_asset_id_to_custom_asset() {
    let custom_asset_id = Compact(7);
    let custom_asset = CustomAssetClass::from(custom_asset_id);
    let custom_asset_id_converted = custom_asset.into();
    assert_eq!(custom_asset_id, custom_asset_id_converted);
}
