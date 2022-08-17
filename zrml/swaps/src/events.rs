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
//
// This file incorporates work covered by the license above but
// published without copyright notice by Balancer Labs
// (<https://balancer.finance>, contact@balancer.finance) in the
// balancer-core repository
// <https://github.com/balancer-labs/balancer-core>.

use alloc::vec::Vec;
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use zeitgeist_primitives::types::PoolId;

#[derive(Clone, Debug, Decode, Default, Encode, Eq, Ord, PartialEq, PartialOrd, TypeInfo)]
pub struct CommonPoolEventParams<AI> {
    pub pool_id: PoolId,
    pub who: AI,
}

/// Common parameters of Balancer all-asset events.
#[derive(Clone, Debug, Decode, Default, Encode, Eq, Ord, PartialEq, PartialOrd, TypeInfo)]
pub struct PoolAssetsEvent<AI, AS, B> {
    pub assets: Vec<AS>,
    pub bounds: Vec<B>,
    pub cpep: CommonPoolEventParams<AI>,
    pub transferred: Vec<B>,
    pub pool_amount: B,
}

/// Common parameters of Balancer single-asset events.
#[derive(Clone, Debug, Decode, Default, Encode, Eq, Ord, PartialEq, PartialOrd, TypeInfo)]
pub struct PoolAssetEvent<AI, AS, B> {
    pub asset: AS,
    pub bound: B,
    pub cpep: CommonPoolEventParams<AI>,
    pub transferred: B,
    pub pool_amount: B,
}

#[derive(Clone, Debug, Decode, Default, Encode, Eq, Ord, PartialEq, PartialOrd, TypeInfo)]
pub struct SwapEvent<AI, AS, B> {
    pub asset_amount_in: B,
    pub asset_amount_out: B,
    pub asset_bound: Option<B>,
    pub asset_in: AS,
    pub asset_out: AS,
    pub cpep: CommonPoolEventParams<AI>,
    pub max_price: Option<B>,
}
