// Copyright 2024 Forecasting Technologies LTD.
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

/// The `ParimutuelAssetClass` enum represents all assets that are specific to parimutuel markets.
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
#[derive(Clone, Copy, Debug, Decode, Eq, Encode, MaxEncodedLen, PartialEq, TypeInfo)]
pub enum ParimutuelAssetClass<MI> {
    #[codec(index = 6)]
    Share(MI, CategoryIndex),
}

impl<MI: HasCompact + MaxEncodedLen> TryFrom<Asset<MI>> for ParimutuelAssetClass<MI> {
    type Error = ();

    fn try_from(value: Asset<MI>) -> Result<Self, Self::Error> {
        match value {
            Asset::<MI>::ParimutuelShare(market_id, cat_id) => Ok(Self::Share(market_id, cat_id)),
            _ => Err(()),
        }
    }
}
