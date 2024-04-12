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

use sp_runtime::Perbill;

/// Trait for distributing fees collected from trading to external recipients like the treasury.
pub trait DistributeFees {
    type Asset;
    type AccountId;
    type Balance;
    type MarketId;

    /// Deduct and distribute the swap fees of the pool from the specified amount and returns the
    /// deducted fees.
    ///
    /// # Arguments
    ///
    /// - `market_id`: The market on which the fees are taken.
    /// - `asset`: The asset the fee is paid in.
    /// - `account`: The account which pays the fees.
    /// - `amount`: The gross amount from which fees are deducted.
    ///
    /// Note that this function is infallible. If distribution is impossible or fails midway, it
    /// should return the balance of the already successfully deducted fees.
    fn distribute(
        market_id: Self::MarketId,
        asset: Self::Asset,
        account: &Self::AccountId,
        amount: Self::Balance,
    ) -> Self::Balance;

    /// Returns the percentage of the fee that is distributed.
    ///
    /// # Arguments
    ///
    /// - `market_id`: The market on which the fees belong to.
    fn fee_percentage(market_id: Self::MarketId) -> Perbill;
}
