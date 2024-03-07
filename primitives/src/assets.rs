// Copyright 2022-2024 Forecasting Technologies LTD.
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

#[cfg(feature = "runtime-benchmarks")]
use crate::traits::ZeitgeistAssetEnumerator;
use crate::{
    traits::PoolSharesId,
    types::{CampaignAssetId, CategoryIndex, CustomAssetId, PoolId},
};
use parity_scale_codec::{Compact, CompactAs, Decode, Encode, HasCompact, MaxEncodedLen};
use scale_info::TypeInfo;

pub use all_assets::Asset;
pub use campaign_assets::CampaignAssetClass;
pub use currencies::CurrencyClass;
pub use custom_assets::CustomAssetClass;
pub use market_assets::MarketAssetClass;
pub use subsets::{BaseAssetClass, ParimutuelAssetClass, XcmAssetClass};

mod all_assets;
mod campaign_assets;
mod currencies;
mod custom_assets;
mod market_assets;
mod subsets;
#[cfg(test)]
mod tests;

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
