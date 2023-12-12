// Copyright 2023 Forecasting Technologies LTD.
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

use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use zeitgeist_primitives::types::Asset;

pub type OrderId = u128;

#[derive(Clone, Encode, Eq, Decode, MaxEncodedLen, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Order<AccountId, Balance, MarketId: MaxEncodedLen> {
    pub market_id: MarketId,
    pub maker: AccountId,
    pub maker_asset: Asset<MarketId>,
    pub maker_amount: Balance,
    pub taker_asset: Asset<MarketId>,
    pub taker_amount: Balance,
}
