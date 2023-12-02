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

use crate::{
    constants::MAX_ASSETS,
    types::{Asset, PoolStatus},
};
use alloc::{collections::BTreeMap, vec::Vec};
use parity_scale_codec::{Compact, Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{RuntimeDebug, SaturatedConversion};

// TODO Remove total_subsidy and make total_weight and weights non-optional, etc.
#[derive(TypeInfo, Clone, Encode, Eq, Decode, PartialEq, RuntimeDebug)]
pub struct Pool<Balance, MarketId>
where
    MarketId: MaxEncodedLen,
{
    pub assets: Vec<Asset<MarketId>>,
    pub pool_status: PoolStatus,
    pub swap_fee: Balance,
    pub total_weight: u128,
    pub weights: BTreeMap<Asset<MarketId>, u128>,
}

impl<Balance, MarketId> Pool<Balance, MarketId>
where
    MarketId: MaxEncodedLen + Ord,
{
    pub fn bound(&self, asset: &Asset<MarketId>) -> bool {
        self.weights.get(asset).is_some()
    }
}

impl<Balance, MarketId> MaxEncodedLen for Pool<Balance, MarketId>
where
    Balance: MaxEncodedLen,
    MarketId: MaxEncodedLen,
{
    fn max_encoded_len() -> usize {
        let max_encoded_length_bytes = <Compact<u64>>::max_encoded_len();
        let b_tree_map_size = 1usize
            .saturating_add(MAX_ASSETS.saturated_into::<usize>().saturating_mul(
                <Asset<MarketId>>::max_encoded_len().saturating_add(u128::max_encoded_len()),
            ))
            .saturating_add(max_encoded_length_bytes);

        <Asset<MarketId>>::max_encoded_len()
            .saturating_mul(MAX_ASSETS.saturated_into::<usize>())
            .saturating_add(max_encoded_length_bytes)
            .saturating_add(PoolStatus::max_encoded_len())
            .saturating_add(<Option<Balance>>::max_encoded_len().saturating_mul(2))
            .saturating_add(<Option<u128>>::max_encoded_len())
            .saturating_add(b_tree_map_size)
    }
}

#[derive(TypeInfo, Clone, Copy, Encode, Eq, Decode, MaxEncodedLen, PartialEq, RuntimeDebug)]
pub enum ScoringRule {
    CPMM,
    Lmsr,
    Orderbook,
    Parimutuel,
}
