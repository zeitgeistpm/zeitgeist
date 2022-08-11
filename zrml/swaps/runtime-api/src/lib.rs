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

//! Runtime API definition for the swaps pallet.

#![cfg_attr(not(feature = "std"), no_std)]

use parity_scale_codec::{Codec, MaxEncodedLen};
use sp_runtime::traits::{MaybeDisplay, MaybeFromStr};
use zeitgeist_primitives::types::{Asset, SerdeWrapper};

sp_api::decl_runtime_apis! {
    pub trait SwapsApi<PoolId, AccountId, Balance, MarketId> where
        PoolId: Codec,
        AccountId: Codec,
        Balance: Codec + MaybeDisplay + MaybeFromStr + MaxEncodedLen,
        MarketId: Codec + MaxEncodedLen,
    {
        fn pool_shares_id(pool_id: PoolId) -> Asset<SerdeWrapper<MarketId>>;
        fn pool_account_id(pool_id: PoolId) -> AccountId;
        fn get_spot_price(pool_id: PoolId, asset_in: Asset<MarketId>, asset_out: Asset<MarketId>) -> SerdeWrapper<Balance>;
    }
}
