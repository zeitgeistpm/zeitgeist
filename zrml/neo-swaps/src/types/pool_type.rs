// Copyright 2024-2025 Forecasting Technologies LTD.
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

use alloc::{boxed::Box, fmt::Debug};
use core::iter;
use frame_support::{CloneNoBound, PartialEqNoBound, RuntimeDebugNoBound};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{traits::Get, BoundedVec};

#[derive(
    CloneNoBound, Decode, Encode, Eq, MaxEncodedLen, PartialEqNoBound, RuntimeDebugNoBound, TypeInfo,
)]
#[scale_info(skip_type_params(MaxMarkets))]
pub(crate) enum PoolType<MarketId, MaxMarkets>
where
    MarketId: Clone + Decode + Debug + Encode + MaxEncodedLen + PartialEq + Eq + TypeInfo,
    MaxMarkets: Get<u32>,
{
    Standard(MarketId),
    Combinatorial(BoundedVec<MarketId, MaxMarkets>),
}

impl<MarketId, MaxMarkets> PoolType<MarketId, MaxMarkets>
where
    MarketId: Clone + Decode + Debug + Encode + MaxEncodedLen + PartialEq + Eq + TypeInfo,
    MaxMarkets: Get<u32>,
{
    pub fn iter_market_ids(&self) -> Box<dyn Iterator<Item = &MarketId> + '_> {
        match self {
            PoolType::Standard(market_id) => Box::new(iter::once(market_id)),
            PoolType::Combinatorial(market_ids) => Box::new(market_ids.iter()),
        }
    }
}
