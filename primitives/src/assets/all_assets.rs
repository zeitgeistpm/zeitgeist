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

use super::*;

/// The `Asset` enum represents all types of assets available in the Zeitgeist
/// system.
///
/// This complete enumeration is intended to abstract the common interaction
/// with tokens away. For example, the developer is not forced to be aware
/// about which exact implementation will handle the desired asset class to
/// instruct operations such as `transfer` or `freeze`, instead it is
/// sufficient to call a crate that manages the routing.
/// While it is not recommended to use this enum in storage, it should not pose
/// a problem as long as all other asset types use the same scale encoding for
/// a matching asset variant in this enum.
///
/// **Deprecated:** Market and Pool assets are "lazy" migrated to
/// pallet-assets.
/// Do not create any new market or pool assets using the deprecated variants
/// in this enum.
///
/// # Types
///
/// * `MI`: Market Id
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
#[derive(
    Clone,
    Copy,
    Debug,
    Decode,
    Default,
    Eq,
    Encode,
    MaxEncodedLen,
    Ord,
    PartialEq,
    PartialOrd,
    TypeInfo,
)]
pub enum Asset<MI: MaxEncodedLen + HasCompact> {
    #[codec(index = 0)]
    CategoricalOutcome(MI, CategoryIndex),

    #[codec(index = 1)]
    ScalarOutcome(MI, ScalarPosition),

    #[codec(index = 3)]
    PoolShare(PoolId),

    #[codec(index = 4)]
    #[default]
    Ztg,

    #[codec(index = 5)]
    ForeignAsset(u32),

    #[codec(index = 6)]
    ParimutuelShare(MI, CategoryIndex),

    // "New" outcomes will replace the previous outcome types after the lazy
    // migration completed
    #[codec(index = 7)]
    NewCategoricalOutcome(#[codec(compact)] MI, #[codec(compact)] CategoryIndex),

    #[codec(index = 9)]
    NewScalarOutcome(#[codec(compact)] MI, ScalarPosition),

    #[codec(index = 10)]
    NewParimutuelShare(#[codec(compact)] MI, #[codec(compact)] CategoryIndex),

    #[codec(index = 11)]
    NewPoolShare(#[codec(compact)] PoolId),

    #[codec(index = 12)]
    CampaignAssetClass(#[codec(compact)] CampaignAssetId),

    #[codec(index = 13)]
    CustomAssetClass(#[codec(compact)] CustomAssetId),
}

impl<MI: HasCompact + MaxEncodedLen> From<MarketAssetClass<MI>> for Asset<MI> {
    fn from(value: MarketAssetClass<MI>) -> Self {
        match value {
            MarketAssetClass::<MI>::OldCategoricalOutcome(market_id, cat_id) => {
                Self::CategoricalOutcome(market_id, cat_id)
            }
            MarketAssetClass::<MI>::OldScalarOutcome(market_id, scalar_pos) => {
                Self::ScalarOutcome(market_id, scalar_pos)
            }
            MarketAssetClass::<MI>::OldParimutuelShare(market_id, cat_id) => {
                Self::ParimutuelShare(market_id, cat_id)
            }
            MarketAssetClass::<MI>::OldPoolShare(pool_id) => Self::PoolShare(pool_id),
            MarketAssetClass::<MI>::CategoricalOutcome(market_id, cat_id) => {
                Self::NewCategoricalOutcome(market_id, cat_id)
            }
            MarketAssetClass::<MI>::ScalarOutcome(market_id, scalar_pos) => {
                Self::NewScalarOutcome(market_id, scalar_pos)
            }
            MarketAssetClass::<MI>::ParimutuelShare(market_id, cat_id) => {
                Self::NewParimutuelShare(market_id, cat_id)
            }
            MarketAssetClass::<MI>::PoolShare(pool_id) => Self::NewPoolShare(pool_id),
        }
    }
}

impl<MI: HasCompact + MaxEncodedLen> From<CampaignAssetClass> for Asset<MI> {
    fn from(value: CampaignAssetClass) -> Self {
        Self::CampaignAssetClass(value.0)
    }
}

impl<MI: HasCompact + MaxEncodedLen> From<CustomAssetClass> for Asset<MI> {
    fn from(value: CustomAssetClass) -> Self {
        Self::CustomAssetClass(value.0)
    }
}

impl<MI: HasCompact + MaxEncodedLen> From<CurrencyClass<MI>> for Asset<MI> {
    fn from(value: CurrencyClass<MI>) -> Self {
        match value {
            CurrencyClass::<MI>::OldCategoricalOutcome(market_id, cat_id) => {
                Self::CategoricalOutcome(market_id, cat_id)
            }
            CurrencyClass::<MI>::OldScalarOutcome(market_id, scalar_pos) => {
                Self::ScalarOutcome(market_id, scalar_pos)
            }
            CurrencyClass::<MI>::OldParimutuelShare(market_id, cat_id) => {
                Self::ParimutuelShare(market_id, cat_id)
            }
            CurrencyClass::<MI>::OldPoolShare(pool_id) => Self::PoolShare(pool_id),
            CurrencyClass::<MI>::ForeignAsset(asset_id) => Self::ForeignAsset(asset_id),
        }
    }
}
