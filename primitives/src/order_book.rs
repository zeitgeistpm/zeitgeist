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

use crate::{
    math::fixed::{FixedDiv, FixedMulDiv},
    types::Asset,
};
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
            self.maker_amount.bdiv_floor(self.taker_amount)
        } else {
            self.taker_amount.bdiv_floor(self.maker_amount)
        }
    }

    /// Return the (partial) amounts the taker and maker need to provide
    /// to fill a `sub_maker_amount` (lower or equal to `maker_amount`) of an order.
    ///
    /// If the `sub_maker_amount` is higher than or equal to the `maker_amount`,
    /// the full fill amounts of the order are returned.
    /// If the `sub_maker_amount` is lower than the `maker_amount`,
    /// the corresponding amounts to partially fill the order are returned.
    ///
    /// The `taker_fill` is the amount in `maker_asset` the maker fills to the taker.
    /// The `maker_fill` is the amount in `taker_asset` the taker fills to the maker.
    ///
    /// Returns `(taker_fill, maker_fill)`.
    pub fn taker_and_maker_fill_from_maker_amount(
        &self,
        sub_maker_amount: Balance,
    ) -> Result<(Balance, Balance), DispatchError> {
        // concious switch to match maker and taker amounts to correct fill amounts
        // the taker gets the maker amount (`taker_fill``)
        // and the maker gets the taker amount (`maker_fill``)
        let (taker_fill, maker_fill) = if sub_maker_amount < self.maker_amount {
            let sub_taker_amount =
                sub_maker_amount.bmul_bdiv_floor(self.taker_amount, self.maker_amount)?;
            (sub_maker_amount, sub_taker_amount)
        } else {
            (self.maker_amount, self.taker_amount)
        };
        Ok((taker_fill, maker_fill))
    }

    /// Return the (partial) amounts the taker and maker need to provide
    /// to fill a `sub_taker_amount` (lower or equal to `taker_amount`) of an order.
    ///
    /// If the `sub_taker_amount` is higher than or equal to the `taker_amount`,
    /// the full fill amounts of the order are returned.
    /// If the `sub_taker_amount` is lower than the `taker_amount`,
    /// the corresponding amounts to partially fill the order are returned.
    ///
    /// The `taker_fill` is the amount in `maker_asset` the maker fills to the taker.
    /// The `maker_fill` is the amount in `taker_asset` the taker fills to the maker.
    ///
    /// Returns `(taker_fill, maker_fill)`.
    pub fn taker_and_maker_fill_from_taker_amount(
        &self,
        sub_taker_amount: Balance,
    ) -> Result<(Balance, Balance), DispatchError> {
        // concious switch to match maker and taker amounts to correct fill amounts
        // the taker gets the maker amount (`taker_fill``)
        // and the maker gets the taker amount (`maker_fill``)
        let (taker_fill, maker_fill) = if sub_taker_amount < self.taker_amount {
            let sub_maker_amount =
                sub_taker_amount.bmul_bdiv_floor(self.maker_amount, self.taker_amount)?;
            (sub_maker_amount, sub_taker_amount)
        } else {
            (self.maker_amount, self.taker_amount)
        };
        Ok((taker_fill, maker_fill))
    }
}
