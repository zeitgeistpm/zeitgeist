// Copyright 2023-2024 Forecasting Technologies LTD.
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

use frame_support::storage::{bounded_btree_map::BoundedBTreeMap, bounded_vec::BoundedVec};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{traits::Get, RuntimeDebug};
use zeitgeist_primitives::constants::MAX_ASSETS;

pub struct MaxAssets;

impl Get<u32> for MaxAssets {
    fn get() -> u32 {
        MAX_ASSETS as u32
    }
}

#[derive(TypeInfo, Clone, Encode, Eq, Decode, MaxEncodedLen, PartialEq, RuntimeDebug)]
pub struct Pool<Asset, Balance> {
    pub assets: BoundedVec<Asset, MaxAssets>,
    pub status: PoolStatus,
    pub swap_fee: Balance,
    pub total_weight: Balance,
    pub weights: BoundedBTreeMap<Asset, Balance, MaxAssets>,
}

impl<Asset, Balance> Pool<Asset, Balance>
where
    Asset: Ord,
{
    pub fn bound(&self, asset: &Asset) -> bool {
        self.weights.get(asset).is_some()
    }
}

/// The status of a pool. Closely related to the lifecycle of a market.
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
#[derive(
    parity_scale_codec::Decode,
    parity_scale_codec::Encode,
    parity_scale_codec::MaxEncodedLen,
    scale_info::TypeInfo,
    Clone,
    Copy,
    Debug,
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
)]
pub enum PoolStatus {
    /// Shares can be normally negotiated.
    Open,
    /// No trading/adding liquidity is allowed.
    Closed,
}
