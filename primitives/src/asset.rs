// Copyright 2022-2025 Forecasting Technologies LTD.
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
    types::{CategoryIndex, CombinatorialId, PoolId},
};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};

/// The `Asset` enum represents all types of assets available in the Zeitgeist
/// system.
#[derive(
    Clone,
    Copy,
    Debug,
    Decode,
    Default,
    Deserialize,
    Eq,
    Encode,
    MaxEncodedLen,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
    TypeInfo,
)]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub enum Asset<MarketId> {
    CategoricalOutcome(MarketId, CategoryIndex),
    ScalarOutcome(MarketId, ScalarPosition),
    CombinatorialOutcomeLegacy, // Here to avoid having to migrate all holdings on the chain.
    PoolShare(PoolId),
    #[default]
    Ztg,
    ForeignAsset(u32),
    ParimutuelShare(MarketId, CategoryIndex),
    CombinatorialToken(CombinatorialId),
}

#[cfg(feature = "runtime-benchmarks")]
impl<MarketId: MaxEncodedLen> ZeitgeistAssetEnumerator<MarketId> for Asset<MarketId> {
    fn create_asset_id(t: MarketId) -> Self {
        Asset::CategoricalOutcome(t, 0)
    }
}

impl<MarketId: MaxEncodedLen> PoolSharesId<PoolId> for Asset<MarketId> {
    fn pool_shares_id(pool_id: PoolId) -> Self {
        Self::PoolShare(pool_id)
    }
}

/// In a scalar market, users can either choose a `Long` position,
/// meaning that they think the outcome will be closer to the upper bound
/// or a `Short` position meaning that they think the outcome will be closer
/// to the lower bound.
#[derive(
    Clone,
    Copy,
    Debug,
    Decode,
    Deserialize,
    Eq,
    Encode,
    MaxEncodedLen,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
    TypeInfo,
)]
#[serde(rename_all = "camelCase")]
pub enum ScalarPosition {
    Long,
    Short,
}
