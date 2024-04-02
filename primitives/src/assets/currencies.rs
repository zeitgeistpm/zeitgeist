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

/// The `CurrencyClass` enum represents all non-ztg currencies
// used in orml-tokens
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
#[derive(
    Clone, Copy, Debug, Decode, Eq, Encode, MaxEncodedLen, Ord, PartialEq, PartialOrd, TypeInfo,
)]
pub enum CurrencyClass<MI> {
    // All Outcome and Share variants will be removed once the lazy migration from
    // orml-tokens to pallet-assets is complete
    #[codec(index = 0)]
    CategoricalOutcome(MI, CategoryIndex),

    #[codec(index = 1)]
    ScalarOutcome(MI, ScalarPosition),

    #[codec(index = 3)]
    PoolShare(PoolId),

    #[codec(index = 5)]
    ForeignAsset(u32),

    #[codec(index = 6)]
    ParimutuelShare(MI, CategoryIndex),
}

impl<MI> CurrencyClass<MI> {
    pub fn is_foreign_asset(&self) -> bool {
        matches!(self, Self::ForeignAsset(_))
    }
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
            Asset::<MI>::ForeignAsset(id) => Ok(Self::ForeignAsset(id)),
            _ => Err(()),
        }
    }
}
