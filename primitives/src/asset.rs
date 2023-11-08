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
// used in orml-tokens
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

#[cfg(test)]
mod tests {
    use super::{
        Asset, CampaignAssetClass, CurrencyClass, CustomAssetClass, MarketAssetClass,
        ScalarPosition,
    };
    use crate::types::MarketId;
    use parity_scale_codec::{Decode, Encode};

    // Verify encode and decode index hack for lazy migration
    mod scale_codec {
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
            Asset::<MarketId>::CombinatorialOutcome,
            MarketAssetClass::<MarketId>::OldCombinatorialOutcome;
            "combinatorial_outcome"
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
            Asset::<MarketId>::NewCombinatorialOutcome,
            MarketAssetClass::<MarketId>::CombinatorialOutcome;
            "new_combinatorial_outcome"
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
        fn index_matching_works_for_market_assets(
            old_asset: Asset<MarketId>,
            new_asset: MarketAssetClass<MarketId>,
        ) {
            let old_asset_encoded: Vec<u8> = old_asset.encode();
            let new_asset_decoded =
                <MarketAssetClass<MarketId> as Decode>::decode(&mut old_asset_encoded.as_slice())
                    .unwrap();
            assert_eq!(new_asset_decoded, new_asset);
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
            Asset::<MarketId>::CombinatorialOutcome,
            CurrencyClass::<MarketId>::OldCombinatorialOutcome;
            "combinatorial_outcome"
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
        fn index_matching_works_for_currencies(
            old_asset: Asset<MarketId>,
            new_asset: CurrencyClass<MarketId>,
        ) {
            let old_asset_encoded: Vec<u8> = old_asset.encode();
            let new_asset_decoded =
                <CurrencyClass<MarketId> as Decode>::decode(&mut old_asset_encoded.as_slice())
                    .unwrap();
            assert_eq!(new_asset_decoded, new_asset);
        }
    }

    // Verify conversion from Assets enum to any other asset type
    mod conversions {
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
            Asset::<MarketId>::CombinatorialOutcome,
            MarketAssetClass::<MarketId>::OldCombinatorialOutcome;
            "combinatorial_outcome"
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
            Asset::<MarketId>::NewCombinatorialOutcome,
            MarketAssetClass::<MarketId>::CombinatorialOutcome;
            "new_combinatorial_outcome"
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
            MarketAssetClass::<MarketId>::OldCombinatorialOutcome,
            Asset::<MarketId>::CombinatorialOutcome;
            "combinatorial_outcome"
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
            MarketAssetClass::<MarketId>::CombinatorialOutcome,
            Asset::<MarketId>::NewCombinatorialOutcome;
            "new_combinatorial_outcome"
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
            Asset::<MarketId>::CombinatorialOutcome,
            CurrencyClass::<MarketId>::OldCombinatorialOutcome;
            "combinatorial_outcome"
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
        fn from_all_assets_to_currencies(
            old_asset: Asset<MarketId>,
            new_asset: CurrencyClass<MarketId>,
        ) {
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
            CurrencyClass::<MarketId>::OldCombinatorialOutcome,
            Asset::<MarketId>::CombinatorialOutcome;
            "combinatorial_outcome"
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
        fn from_currencies_to_all_assets(
            old_asset: CurrencyClass<MarketId>,
            new_asset: Asset<MarketId>,
        ) {
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
    }
}
