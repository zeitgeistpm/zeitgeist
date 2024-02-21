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

use frame_support::{
    dispatch::{fmt::Debug, DispatchError, DispatchResult},
    pallet_prelude::{MaybeSerializeDeserialize, Member},
    Parameter,
};
use parity_scale_codec::{FullCodec, MaxEncodedLen};
use sp_runtime::traits::{AtLeast32Bit, AtLeast32BitUnsigned};

/// Trait for handling the Order Book part of the hybrid router.
pub trait HybridRouterOrderBookApi {
    type AccountId;
    type MarketId: AtLeast32Bit
        + Copy
        + Default
        + MaybeSerializeDeserialize
        + MaxEncodedLen
        + Member
        + Parameter;
    type Balance: AtLeast32BitUnsigned
        + FullCodec
        + Copy
        + MaybeSerializeDeserialize
        + Debug
        + Default
        + scale_info::TypeInfo
        + MaxEncodedLen;
    type Asset;
    type Order;
    type OrderId;

    /// Returns the order with the specified `order_id`.
    ///
    /// # Arguments
    ///
    /// - `order_id`: The id of the order to return.
    fn order(order_id: Self::OrderId) -> Result<Self::Order, DispatchError>;

    /// Fills the order with the specified `order_id` with the specified `maker_partial_fill` amount.
    ///
    /// # Arguments
    ///
    /// - `who`: The account that fills the order.
    /// - `order_id`: The id of the order to fill.
    /// - `maker_partial_fill`: The amount to fill the order with.
    fn fill_order(
        who: Self::AccountId,
        order_id: Self::OrderId,
        maker_partial_fill: Option<Self::Balance>,
    ) -> DispatchResult;

    /// Places an order on the order book.
    ///
    /// # Arguments
    ///
    /// - `who`: The account that places the order.
    /// - `market_id`: The market on which the order is placed.
    /// - `maker_asset`: The asset the maker wants to trade.
    /// - `maker_amount`: The amount the maker wants to trade.
    /// - `taker_asset`: The asset the maker wants to receive.
    /// - `taker_amount`: The amount the maker wants to receive.
    fn place_order(
        who: Self::AccountId,
        market_id: Self::MarketId,
        maker_asset: Self::Asset,
        maker_amount: Self::Balance,
        taker_asset: Self::Asset,
        taker_amount: Self::Balance,
    ) -> DispatchResult;
}
