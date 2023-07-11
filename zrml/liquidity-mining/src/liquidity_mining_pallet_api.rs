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

use frame_support::dispatch::DispatchResult;

/// Interface to interact with the Zeitgeist Liquidity Mining pallet.
pub trait LiquidityMiningPalletApi {
    type AccountId;
    type Balance;
    type BlockNumber;
    type MarketId;

    /// Increases the number of stored pool shares of an account for a given market.
    ///
    /// It is up to the caller to synchronize the amount of shares between different pallets
    fn add_shares(account_id: Self::AccountId, market_id: Self::MarketId, shares: Self::Balance);

    /// Removes a given `market_id` period from the storage distributing incentives to all
    /// related accounts.
    fn distribute_market_incentives(market_id: &Self::MarketId) -> DispatchResult;

    /// Decreases the number of stored pool shares of an account on a given market.
    ///
    /// It is up to the caller to synchronize the amount of shares between different pallets
    fn remove_shares(
        account_id: &Self::AccountId,
        market_id: &Self::MarketId,
        shares: Self::Balance,
    );
}
