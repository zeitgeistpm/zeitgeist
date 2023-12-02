// Copyright 2023 Forecasting Technologies LTD.
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

use crate::{constants::MAX_ASSETS, types::PoolStatus};
use alloc::{collections::BTreeMap, vec::Vec};
use parity_scale_codec::{Compact, Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{RuntimeDebug, SaturatedConversion};

// TODO Move  out of primitives
// TODO Use bounded
#[derive(TypeInfo, Clone, Encode, Eq, Decode, PartialEq, RuntimeDebug)]
pub struct Pool<Asset, Balance> {
    pub assets: Vec<Asset>,
    pub status: PoolStatus,
    pub swap_fee: Balance,
    pub total_weight: u128,
    pub weights: BTreeMap<Asset, u128>,
}

impl<Asset, Balance> Pool<Asset, Balance>
where
    Asset: Ord,
{
    pub fn bound(&self, asset: &Asset) -> bool {
        self.weights.get(asset).is_some()
    }
}

impl<Asset, Balance> MaxEncodedLen for Pool<Asset, Balance>
where
    Asset: MaxEncodedLen,
    Balance: MaxEncodedLen,
{
    fn max_encoded_len() -> usize {
        let assets_size = Asset::max_encoded_len().saturating_mul(MAX_ASSETS.saturated_into());
        let status_size = PoolStatus::max_encoded_len();
        let swap_fee_size = Balance::max_encoded_len();
        let total_weight_size = u128::max_encoded_len();
        let max_encoded_length_bytes = Compact::<u64>::max_encoded_len();
        let weights_size =
            1usize
                .saturating_add(MAX_ASSETS.saturated_into::<usize>().saturating_mul(
                    Asset::max_encoded_len().saturating_add(u128::max_encoded_len()),
                ))
                .saturating_add(max_encoded_length_bytes);
        assets_size
            .saturating_add(status_size)
            .saturating_add(swap_fee_size)
            .saturating_add(total_weight_size)
            .saturating_add(weights_size)
    }
}

#[derive(TypeInfo, Clone, Copy, Encode, Eq, Decode, MaxEncodedLen, PartialEq, RuntimeDebug)]
pub enum ScoringRule {
    CPMM,
    Lmsr,
    Orderbook,
    Parimutuel,
}
