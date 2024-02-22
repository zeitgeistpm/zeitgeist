// Copyright 2023-2024 Forecasting Technologies LTD.
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

use crate::{math::fixed::FixedDiv, types::Asset};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{traits::AtLeast32BitUnsigned, DispatchError, RuntimeDebug};

pub type OrderId = u128;

#[derive(Clone, Encode, Eq, Decode, MaxEncodedLen, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Order<
    AccountId,
    Balance: AtLeast32BitUnsigned + Copy,
    MarketId: MaxEncodedLen + PartialEq,
> {
    pub market_id: MarketId,
    pub maker: AccountId,
    pub maker_asset: Asset<MarketId>,
    pub maker_amount: Balance,
    pub taker_asset: Asset<MarketId>,
    pub taker_amount: Balance,
}

impl<AccountId, Balance: AtLeast32BitUnsigned + Copy, MarketId: MaxEncodedLen + PartialEq>
    Order<AccountId, Balance, MarketId>
{
    pub fn price(&self, base_asset: Asset<MarketId>) -> Result<Balance, DispatchError> {
        if self.maker_asset == base_asset {
            self.taker_amount.bdiv_floor(self.maker_amount)
        } else {
            self.maker_amount.bdiv_floor(self.taker_amount)
        }
    }
}
