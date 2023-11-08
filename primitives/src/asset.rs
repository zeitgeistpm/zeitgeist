// Copyright 2022-2023 Forecasting Technologies LTD.
// Copyright 2021-2022 Zeitgeist PM LLC.
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

use crate::types::{CampaignAssetId, CategoryIndex, CustomAssetId, PoolId};
use parity_scale_codec::{Compact, CompactAs, Decode, Encode, HasCompact, MaxEncodedLen};
use scale_info::TypeInfo;

/// The `Asset` enum represents all types of assets available in the Zeitgeist
/// system.
///
/// Never use this enum in storage. Use this enum only to interact with assets
/// via an asset manager (transfer, freeze, etc.). Use other explicit and
/// distinct asset classes instead if storage of asset information is needed.
///
/// **Deprecated:** Market and Pool assets are "lazy" migrated to pallet-assets
/// Do not create any new market or pool assets using this enumeration.
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
    #[codec(index = 2)]
    CombinatorialOutcome,
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

    #[codec(index = 8)]
    NewCombinatorialOutcome,

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
            MarketAssetClass::<MI>::OldCategoricalOutcome(marketid, catid) => {
                Self::CategoricalOutcome(marketid, catid)
            }
            MarketAssetClass::<MI>::OldCombinatorialOutcome => Self::CombinatorialOutcome,
            MarketAssetClass::<MI>::OldScalarOutcome(marketid, scalarpos) => {
                Self::ScalarOutcome(marketid, scalarpos)
            }
            MarketAssetClass::<MI>::OldParimutuelShare(marketid, catid) => {
                Self::ParimutuelShare(marketid, catid)
            }
            MarketAssetClass::<MI>::OldPoolShare(poolid) => Self::PoolShare(poolid),
            MarketAssetClass::<MI>::CategoricalOutcome(marketid, catid) => {
                Self::NewCategoricalOutcome(marketid, catid)
            }
            MarketAssetClass::<MI>::CombinatorialOutcome => Self::NewCombinatorialOutcome,
            MarketAssetClass::<MI>::ScalarOutcome(marketid, scalarpos) => {
                Self::NewScalarOutcome(marketid, scalarpos)
            }
            MarketAssetClass::<MI>::ParimutuelShare(marketid, catid) => {
                Self::NewParimutuelShare(marketid, catid)
            }
            MarketAssetClass::<MI>::PoolShare(poolid) => Self::NewPoolShare(poolid),
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
            CurrencyClass::<MI>::OldCategoricalOutcome(marketid, catid) => {
                Self::CategoricalOutcome(marketid, catid)
            }
            CurrencyClass::<MI>::OldCombinatorialOutcome => Self::CombinatorialOutcome,
            CurrencyClass::<MI>::OldScalarOutcome(marketid, scalarpos) => {
                Self::ScalarOutcome(marketid, scalarpos)
            }
            CurrencyClass::<MI>::OldParimutuelShare(marketid, catid) => {
                Self::ParimutuelShare(marketid, catid)
            }
            CurrencyClass::<MI>::OldPoolShare(poolid) => Self::PoolShare(poolid),
            CurrencyClass::<MI>::ForeignAsset(assetid) => Self::ForeignAsset(assetid),
        }
    }
}

/// The `MarketAsset` enum represents all types of assets available in the
/// Prediction Market protocol
///
/// # Types
///
/// * `MI`: Market Id
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
#[derive(Clone, Copy, Debug, Decode, Default, Eq, Encode, MaxEncodedLen, PartialEq, TypeInfo)]
pub enum MarketAssetClass<MI: HasCompact + MaxEncodedLen> {
    // All "Old" variants will be removed once the lazy migration from
    // orml-tokens to pallet-assets is complete
    #[codec(index = 0)]
    OldCategoricalOutcome(MI, CategoryIndex),

    #[codec(index = 2)]
    OldCombinatorialOutcome,

    #[codec(index = 1)]
    OldScalarOutcome(MI, ScalarPosition),

    #[codec(index = 6)]
    OldParimutuelShare(MI, CategoryIndex),

    #[codec(index = 3)]
    OldPoolShare(PoolId),

    #[codec(index = 7)]
    CategoricalOutcome(#[codec(compact)] MI, #[codec(compact)] CategoryIndex),

    #[codec(index = 8)]
    #[default]
    CombinatorialOutcome,

    #[codec(index = 9)]
    ScalarOutcome(#[codec(compact)] MI, ScalarPosition),

    #[codec(index = 10)]
    ParimutuelShare(#[codec(compact)] MI, #[codec(compact)] CategoryIndex),

    #[codec(index = 11)]
    PoolShare(#[codec(compact)] PoolId),
}

impl<MI: HasCompact + MaxEncodedLen> TryFrom<Asset<MI>> for MarketAssetClass<MI> {
    type Error = ();

    fn try_from(value: Asset<MI>) -> Result<Self, Self::Error> {
        match value {
            Asset::<MI>::NewCategoricalOutcome(marketid, catid) => {
                Ok(Self::CategoricalOutcome(marketid, catid))
            }
            Asset::<MI>::NewCombinatorialOutcome => Ok(Self::CombinatorialOutcome),
            Asset::<MI>::NewScalarOutcome(marketid, scalarpos) => {
                Ok(Self::ScalarOutcome(marketid, scalarpos))
            }
            Asset::<MI>::NewParimutuelShare(marketid, catid) => {
                Ok(Self::ParimutuelShare(marketid, catid))
            }
            Asset::<MI>::NewPoolShare(poolid) => Ok(Self::PoolShare(poolid)),
            Asset::<MI>::CategoricalOutcome(marketid, catid) => {
                Ok(Self::OldCategoricalOutcome(marketid, catid))
            }
            Asset::<MI>::CombinatorialOutcome => Ok(Self::OldCombinatorialOutcome),
            Asset::<MI>::ScalarOutcome(marketid, scalarpos) => {
                Ok(Self::OldScalarOutcome(marketid, scalarpos))
            }
            Asset::<MI>::ParimutuelShare(marketid, catid) => {
                Ok(Self::OldParimutuelShare(marketid, catid))
            }
            Asset::<MI>::PoolShare(poolid) => Ok(Self::OldPoolShare(poolid)),
            _ => Err(()),
        }
    }
}

/// The `CustomAsset` tuple struct represents all custom assets.
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
#[derive(
    Clone, CompactAs, Copy, Debug, Decode, Default, Eq, Encode, MaxEncodedLen, PartialEq, TypeInfo,
)]
pub struct CampaignAssetClass(#[codec(compact)] CampaignAssetId);

impl From<Compact<CampaignAssetId>> for CampaignAssetClass {
    fn from(value: Compact<CampaignAssetId>) -> CampaignAssetClass {
        CampaignAssetClass(value.into())
    }
}

impl From<CampaignAssetClass> for Compact<CampaignAssetId> {
    fn from(value: CampaignAssetClass) -> Compact<CampaignAssetId> {
        value.0.into()
    }
}

impl<MI: HasCompact + MaxEncodedLen> TryFrom<Asset<MI>> for CampaignAssetClass {
    type Error = ();

    fn try_from(value: Asset<MI>) -> Result<Self, Self::Error> {
        match value {
            Asset::<MI>::CampaignAssetClass(id) => Ok(Self(id)),
            _ => Err(()),
        }
    }
}

/// The `CustomAsset` tuple struct represents all custom assets.
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
#[derive(
    Clone, CompactAs, Copy, Debug, Decode, Default, Eq, Encode, MaxEncodedLen, PartialEq, TypeInfo,
)]
pub struct CustomAssetClass(#[codec(compact)] CustomAssetId);

impl From<Compact<CampaignAssetId>> for CustomAssetClass {
    fn from(value: Compact<CampaignAssetId>) -> CustomAssetClass {
        CustomAssetClass(value.into())
    }
}

impl From<CustomAssetClass> for Compact<CampaignAssetId> {
    fn from(value: CustomAssetClass) -> Compact<CampaignAssetId> {
        value.0.into()
    }
}

impl<MI: HasCompact + MaxEncodedLen> TryFrom<Asset<MI>> for CustomAssetClass {
    type Error = ();

    fn try_from(value: Asset<MI>) -> Result<Self, Self::Error> {
        match value {
            Asset::<MI>::CustomAssetClass(id) => Ok(Self(id)),
            _ => Err(()),
        }
    }
}

/// The `CurrencyClass` enum represents all non-ztg CurrencyClass
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
#[derive(Clone, Copy, Debug, Decode, Eq, Encode, MaxEncodedLen, PartialEq, TypeInfo)]
pub enum CurrencyClass<MI> {
    // All "Old" variants will be removed once the lazy migration from
    // orml-tokens to pallet-assets is complete
    #[codec(index = 0)]
    OldCategoricalOutcome(MI, CategoryIndex),

    #[codec(index = 2)]
    OldCombinatorialOutcome,

    #[codec(index = 1)]
    OldScalarOutcome(MI, ScalarPosition),

    #[codec(index = 6)]
    OldParimutuelShare(MI, CategoryIndex),

    #[codec(index = 3)]
    OldPoolShare(PoolId),

    // Type can not be compacted as it is already used uncompacted in the storage
    #[codec(index = 5)]
    ForeignAsset(u32),
}

impl<MI> Default for CurrencyClass<MI> {
    fn default() -> Self {
        Self::ForeignAsset(u32::default())
    }
}

impl<MI: HasCompact + MaxEncodedLen> TryFrom<Asset<MI>> for CurrencyClass<MI> {
    type Error = ();

    fn try_from(value: Asset<MI>) -> Result<Self, Self::Error> {
        match value {
            Asset::<MI>::ForeignAsset(id) => Ok(Self::ForeignAsset(id)),
            _ => Err(()),
        }
    }
}

/// In a scalar market, users can either choose a `Long` position,
/// meaning that they think the outcome will be closer to the upper bound
/// or a `Short` position meaning that they think the outcome will be closer
/// to the lower bound.
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
#[derive(
    Clone, Copy, Debug, Decode, Eq, Encode, MaxEncodedLen, Ord, PartialEq, PartialOrd, TypeInfo,
)]
pub enum ScalarPosition {
    Long,
    Short,
}

mod tests {
    // TODO
    // Verify encode and decode index hack
    // Verify conversion from Assets enum to any other asset type
    // Verify conversion from any other asset type to assets enum
    // Verify asset type conversions
}
