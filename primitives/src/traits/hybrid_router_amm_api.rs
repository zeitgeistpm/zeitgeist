// Copyright 2024 Forecasting Technologies LTD.
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

/// Trait for handling the AMM part of the hybrid router.
pub trait HybridRouterAmmApi {
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

    fn pool_exists(market_id: Self::MarketId) -> bool;

    fn get_spot_price(
        market_id: Self::MarketId,
        asset: Self::Asset,
    ) -> Result<Self::Balance, DispatchError>;

    /// Returns the amount a user has to buy to move the price of `asset` to `until`; zero if the
    /// current spot price is above `until`.
    ///
    /// # Arguments
    ///
    /// - `market_id`: The market for which to get the buy amount.
    /// - `asset`: The asset to calculate the buy amount for.
    /// - `until`: At most until this maximum price.
    fn calculate_buy_amount_until(
        market_id: Self::MarketId,
        asset: Self::Asset,
        until: Self::Balance,
    ) -> Result<Self::Balance, DispatchError>;

    fn buy(
        who: &Self::AccountId,
        market_id: Self::MarketId,
        asset_out: Self::Asset,
        amount_in: Self::Balance,
        min_amount_out: Self::Balance,
    ) -> DispatchResult;

    /// Returns the amount a user has to sell to move the price of `asset` to `until`; zero if the
    /// current spot price is below `until`.
    ///
    /// # Arguments
    ///
    /// - `market_id`: The market for which to get the sell amount.
    /// - `asset`: The asset to calculate the sell amount for.
    /// - `until`: At most until this minimum price.
    fn calculate_sell_amount_until(
        market_id: Self::MarketId,
        asset: Self::Asset,
        until: Self::Balance,
    ) -> Result<Self::Balance, DispatchError>;

    fn sell(
        who: &Self::AccountId,
        market_id: Self::MarketId,
        asset_out: Self::Asset,
        amount_in: Self::Balance,
        min_amount_out: Self::Balance,
    ) -> DispatchResult;
}
