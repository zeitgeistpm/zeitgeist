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

use crate::types::{CategoryIndex, CampaignAssetId, CustomAssetId, PoolId, SerdeWrapper};
use parity_scale_codec::{Compact, HasCompact, CompactAs, Decode, Encode, MaxEncodedLen};
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
    CategoricalOutcome(MI, CategoryIndex),
    ScalarOutcome(MI, ScalarPosition),
    CombinatorialOutcome,
    PoolShare(SerdeWrapper<PoolId>),

    #[default]
    Ztg,

    ForeignAsset(u32),

    ParimutuelShare(MI, CategoryIndex),

    CampaignAssetClass(
        #[codec(compact)] CampaignAssetId
    ),

    CustomAssetClass(
        #[codec(compact)] CustomAssetId
    ),

    NewCategoricalOutcome(
        #[codec(compact)] MI, 
        #[codec(compact)] CategoryIndex,
    ),

    NewCombinatorialOutcome,

    NewScalarOutcome(
        #[codec(compact)] MI, 
        ScalarPosition,
    ),

    NewParimutuelShare(
        #[codec(compact)] MI,
        #[codec(compact)] CategoryIndex,
    ),

    NewPoolShare(
        #[codec(compact)] PoolId,
    ),

    NewForeignAsset(
        #[codec(compact)] u32
    ),
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
    CategoricalOutcome(
        #[codec(compact)] MI, 
        #[codec(compact)] CategoryIndex,
    ),

    #[codec(index = 8)]
    #[default]
    CombinatorialOutcome,

    #[codec(index = 9)]
    ScalarOutcome(
        #[codec(compact)] MI, 
        ScalarPosition,
    ),

    #[codec(index = 10)]
    ParimutuelShare(
        #[codec(compact)] MI,
        #[codec(compact)] CategoryIndex,
    ),

    #[codec(index = 11)]
    PoolShare(
        #[codec(compact)] PoolId,
    ),
}

/// The `CustomAsset` tuple struct represents all custom assets.
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
#[derive(Clone, 
    CompactAs,
    Copy,
    Debug,
    Decode,
    Default,
    Eq,
    Encode,
    MaxEncodedLen,
    PartialEq,
    TypeInfo
)]
pub struct CampaignAssetClass(
    #[codec(compact)] CampaignAssetId,
);

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

/// The `CustomAsset` tuple struct represents all custom assets.
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
#[derive(Clone, 
    CompactAs,
    Copy,
    Debug,
    Decode,
    Default,
    Eq,
    Encode,
    MaxEncodedLen,
    PartialEq,
    TypeInfo
)]
pub struct CustomAssetClass(
    #[codec(compact)] CustomAssetId,
);

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
}
