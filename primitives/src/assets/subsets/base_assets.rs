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

/// The `BaseAssetClass` enum represents all assets that can be used as collateral in
/// prediction markets.
#[derive(
    Clone, Copy, Debug, Decode, Default, Eq, Encode, MaxEncodedLen, Ord, PartialEq, PartialOrd, TypeInfo,
    serde::Deserialize, serde::Serialize
)]
#[serde(rename_all = "camelCase")]
pub enum BaseAssetClass {
    #[codec(index = 4)]
    #[default]
    Ztg,

    #[codec(index = 5)]
    ForeignAsset(u32),

    #[codec(index = 7)]
    CampaignAsset(#[codec(compact)] CampaignAssetId),
}

impl<MI: HasCompact + MaxEncodedLen> TryFrom<Asset<MI>> for BaseAssetClass {
    type Error = ();

    fn try_from(value: Asset<MI>) -> Result<Self, Self::Error> {
        match value {
            Asset::<MI>::Ztg => Ok(Self::Ztg),
            Asset::<MI>::ForeignAsset(id) => Ok(Self::ForeignAsset(id)),
            Asset::<MI>::CampaignAsset(id) => Ok(Self::CampaignAsset(id)),
            _ => Err(()),
        }
    }
}
