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

#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]

use parity_scale_codec::{Codec, MaxEncodedLen};
use zeitgeist_primitives::types::Asset;

sp_api::decl_runtime_apis! {
    pub trait PredictionMarketsApi<MarketId, Hash> where
        MarketId: Codec + MaxEncodedLen,
        Hash: Codec,
    {
        fn market_outcome_share_id(market_id: MarketId, outcome: u16) -> Asset<MarketId>;
    }
}
