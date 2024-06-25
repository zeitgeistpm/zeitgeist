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

use crate::hybrid_router_api_types::{AmmSoftFail, AmmTrade, ApiError};
use sp_runtime::DispatchError;

/// A type alias for the return struct of AMM buy and sell.
type AmmTradeOf<T> = AmmTrade<<T as HybridRouterAmmApi>::Balance>;

/// A type alias for the error type of the AMM part of the hybrid router.
type ApiErrorOf = ApiError<AmmSoftFail>;

/// Trait for handling the AMM part of the hybrid router.
pub trait HybridRouterAmmApi {
    type AccountId;
    type Asset;
    type Balance;
    type MarketId;

    /// Checks if a pool exists for the given market ID.
    ///
    /// # Arguments
    ///
    /// - `market_id`: The market ID to check.
    ///
    /// # Returns
    ///
    /// Returns `true` if the pool exists, `false` otherwise.
    fn pool_exists(market_id: Self::MarketId) -> bool;

    /// Gets the spot price for the given market ID and asset.
    ///
    /// # Arguments
    ///
    /// - `market_id`: The market ID.
    /// - `asset`: The asset to get the spot price for.
    ///
    /// # Returns
    ///
    /// Returns the spot price as a `Result` containing the balance, or an error if the spot price
    /// cannot be retrieved.
    fn get_spot_price(
        market_id: Self::MarketId,
        asset: Self::Asset,
    ) -> Result<Self::Balance, DispatchError>;

    /// Calculates the amount a user has to buy to move the price of `asset` to `until`. Returns
    /// zero if the current spot price is above or equal to `until`.
    ///
    /// # Arguments
    ///
    /// - `market_id`: The market ID for which to calculate the buy amount.
    /// - `asset`: The asset to calculate the buy amount for.
    /// - `until`: The maximum price.
    ///
    /// # Returns
    ///
    /// Returns the buy amount as a `Result` containing the balance, or an error if the buy amount
    /// cannot be calculated.
    fn calculate_buy_amount_until(
        market_id: Self::MarketId,
        asset: Self::Asset,
        until: Self::Balance,
    ) -> Result<Self::Balance, DispatchError>;

    /// Executes a buy transaction.
    ///
    /// # Arguments
    ///
    /// - `who`: The account ID of the user performing the buy.
    /// - `market_id`: The market ID.
    /// - `asset_out`: The asset to receive from the buy.
    /// - `amount_in`: The base asset amount to input for the buy.
    /// - `min_amount_out`: The minimum amount to receive from the buy.
    ///
    /// # Returns
    ///
    /// Returns information about the buy trade made.
    fn buy(
        who: Self::AccountId,
        market_id: Self::MarketId,
        asset_out: Self::Asset,
        amount_in: Self::Balance,
        min_amount_out: Self::Balance,
    ) -> Result<AmmTradeOf<Self>, ApiErrorOf>;

    /// Calculates the amount a user has to sell to move the price of `asset` to `until`. Returns
    /// zero if the current spot price is below or equal to `until`.
    ///
    /// # Arguments
    ///
    /// - `market_id`: The market ID for which to calculate the sell amount.
    /// - `asset`: The asset to calculate the sell amount for.
    /// - `until`: The minimum price.
    ///
    /// # Returns
    ///
    /// Returns the sell amount as a `Result` containing the balance, or an error if the sell amount
    /// cannot be calculated.
    fn calculate_sell_amount_until(
        market_id: Self::MarketId,
        asset: Self::Asset,
        until: Self::Balance,
    ) -> Result<Self::Balance, DispatchError>;

    /// Executes a sell transaction.
    ///
    /// # Arguments
    ///
    /// - `who`: The account ID of the user performing the sell.
    /// - `market_id`: The market ID.
    /// - `asset_in`: The asset to sell.
    /// - `amount_in`: The amount to input for the sell.
    /// - `min_amount_out`: The minimum amount to receive from the sell.
    ///
    /// # Returns
    ///
    /// Returns information about the sell trade made.
    fn sell(
        who: Self::AccountId,
        market_id: Self::MarketId,
        asset_in: Self::Asset,
        amount_in: Self::Balance,
        min_amount_out: Self::Balance,
    ) -> Result<AmmTradeOf<Self>, ApiErrorOf>;
}
