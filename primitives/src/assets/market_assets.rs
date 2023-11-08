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

use super::*;

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
