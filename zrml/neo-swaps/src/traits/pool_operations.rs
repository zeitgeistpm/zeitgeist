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

use crate::pallet::{AssetOf, BalanceOf, Config};
use alloc::vec::Vec;
use sp_runtime::{DispatchError, DispatchResult};

/// Trait for LMSR calculations and access to pool data.
pub(crate) trait PoolOperations<T: Config> {
    /// Return an ordered vector containing the assets held in the pool.
    fn assets(&self) -> Vec<AssetOf<T>>;

    /// Return `true` if the pool holds `asset`.
    fn contains(&self, asset: &AssetOf<T>) -> bool;

    /// Return the reserve of `asset` held in the pool.
    ///
    /// Beware! The reserve need not coincide with the balance in the pool account.
    fn reserve_of(&self, asset: &AssetOf<T>) -> Result<BalanceOf<T>, DispatchError>;

    /// Perform a checked addition to the balance of `asset`.
    fn increase_reserve(
        &mut self,
        asset: &AssetOf<T>,
        increase_amount: &BalanceOf<T>,
    ) -> DispatchResult;

    /// Perform a checked subtraction from the balance of `asset`.
    fn decrease_reserve(
        &mut self,
        asset: &AssetOf<T>,
        decrease_amount: &BalanceOf<T>,
    ) -> DispatchResult;

    /// Calculate the amount received from the swap that is executed when buying (the function
    /// `y(x)` from the documentation).
    ///
    /// Note that `y(x)` does not include the amount of `asset_out` received from buying complete
    /// sets and is therefore _not_ the total amount received from the buy.
    ///
    /// # Parameters
    ///
    /// - `asset_out`: The outcome being bought.
    /// - `amount_in`: The amount of collateral paid.
    fn calculate_swap_amount_out_for_buy(
        &self,
        asset_out: AssetOf<T>,
        amount_in: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError>;

    /// Calculate the amount receives from selling an outcome to the pool.
    ///
    /// # Parameters
    ///
    /// - `asset_in`: The outcome being sold.
    /// - `amount_in`: The amount of `asset_in` sold.
    fn calculate_swap_amount_out_for_sell(
        &self,
        asset_in: AssetOf<T>,
        amount_in: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError>;

    /// Calculate the spot price of `asset`.
    fn calculate_spot_price(&self, asset: AssetOf<T>) -> Result<BalanceOf<T>, DispatchError>;

    /// Calculate a numerical threshold, which determines the maximum number of units of outcomes
    /// anyone is allowed to swap in or out of the pool, and the minimum prices required for selling
    /// to the pool.
    fn calculate_numerical_threshold(&self) -> BalanceOf<T>;

    /// Calculate the ln argument used when calculating amounts out for buys. Underflows do not
    /// raise an error and are rounded down to zero instead.
    ///
    /// # Parameters
    ///
    /// - `asset_out`: The outcome being bought.
    /// - `amount_in`: The amount of collateral paid.
    fn calculate_buy_ln_argument(
        &self,
        asset: AssetOf<T>,
        amount_in: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError>;

    /// Calculates the amount a user has to buy to move the price of `asset` to `until`. Returns
    /// zero if the current spot price is above or equal to `until`.
    ///
    /// # Parameters
    ///
    /// - `asset`: The asset to calculate the buy amount for.
    /// - `until`: The maximum price.
    fn calculate_buy_amount_until(
        &self,
        asset: AssetOf<T>,
        until: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError>;

    /// Calculates the amount a user has to sell to move the price of `asset` to `until`. Returns
    /// zero if the current spot price is below or equal to `until`.
    ///
    /// # Parameters
    ///
    /// - `asset`: The asset to calculate the sell amount for.
    /// - `until`: The minimum price.
    fn calculate_sell_amount_until(
        &self,
        asset: AssetOf<T>,
        until: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError>;
}
