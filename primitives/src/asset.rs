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
    types::{CategoryIndex, PoolId, SerdeWrapper},
};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

/// The `Asset` enum represents all types of assets available in the Zeitgeist
/// system.
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
pub enum Asset<MI: MaxEncodedLen> {
    CategoricalOutcome(MI, CategoryIndex),
    ScalarOutcome(MI, ScalarPosition),
    CombinatorialOutcome,
    PoolShare(SerdeWrapper<PoolId>),
    #[default]
    Ztg,
    ForeignAsset(u32),
    ParimutuelShare(MI, CategoryIndex),
}

impl<MI: MaxEncodedLen> PoolSharesId<SerdeWrapper<PoolId>> for Asset<MI> {
    fn pool_shares_id(pool_id: SerdeWrapper<PoolId>) -> Self {
        Self::PoolShare(pool_id)
    }
}

#[cfg(feature = "runtime-benchmarks")]
impl<MI: MaxEncodedLen> ZeitgeistAssetEnumerator<MI> for Asset<MI> {
    fn create_asset_id(t: MI) -> Self {
        Asset::CategoricalOutcome(t, 0)
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
