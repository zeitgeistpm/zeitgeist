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

/// The `MarketAsset` enum represents all types of assets available in the
/// Prediction Market protocol
///
/// # Types
///
/// * `MI`: Market Id
#[derive(
    Clone,
    Copy,
    Debug,
    Decode,
    Eq,
    Encode,
    MaxEncodedLen,
    Ord,
    PartialEq,
    PartialOrd,
    TypeInfo,
    serde::Deserialize,
    serde::Serialize,
)]
#[serde(rename_all = "camelCase")]
pub enum MarketAssetClass<MI: HasCompact + MaxEncodedLen> {
    #[codec(index = 0)]
    CategoricalOutcome(MI, CategoryIndex),

    #[codec(index = 1)]
    ScalarOutcome(MI, ScalarPosition),

    #[codec(index = 3)]
    PoolShare(PoolId),

    #[codec(index = 6)]
    ParimutuelShare(MI, CategoryIndex),
}

impl<MI: HasCompact + MaxEncodedLen> TryFrom<Asset<MI>> for MarketAssetClass<MI> {
    type Error = ();

    fn try_from(value: Asset<MI>) -> Result<Self, Self::Error> {
        match value {
            Asset::<MI>::CategoricalOutcome(market_id, cat_id) => {
                Ok(Self::CategoricalOutcome(market_id, cat_id))
            }
            Asset::<MI>::ScalarOutcome(market_id, scalar_pos) => {
                Ok(Self::ScalarOutcome(market_id, scalar_pos))
            }
            Asset::<MI>::ParimutuelShare(market_id, cat_id) => {
                Ok(Self::ParimutuelShare(market_id, cat_id))
            }
            Asset::<MI>::PoolShare(pool_id) => Ok(Self::PoolShare(pool_id)),
            _ => Err(()),
        }
    }
}
