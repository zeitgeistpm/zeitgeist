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

/// The `CustomAsset` tuple struct represents all custom assets.
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
#[derive(
    Clone, CompactAs, Copy, Debug, Decode, Default, Eq, Encode, MaxEncodedLen, PartialEq, TypeInfo,
)]
pub struct CustomAssetClass(#[codec(compact)] pub CustomAssetId);

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
