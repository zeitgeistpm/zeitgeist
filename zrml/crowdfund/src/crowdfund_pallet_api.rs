// Copyright 2022 Forecasting Technologies LTD.
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

extern crate alloc;

use alloc::vec::Vec;
use sp_runtime::DispatchResult;
use zeitgeist_primitives::types::OutcomeReport;

/// The trait to initiate and resolve the global disputes.
pub trait CrowdfundPalletApi<MarketId, AccountId, Balance> {
    fn start_crowdfund(
        market_id: &MarketId,
    ) -> DispatchResult;

    fn stop_crowdfund(
        market_id: &MarketId,
    ) -> DispatchResult;

    /// Query the all looser balances combined
    ///
    /// # Arguments
    /// - `market_id` - The id of the market.
    ///
    /// # Returns
    ///
    /// Returns the looser balance.
    fn get_looser_stake(market_id: &MarketId) -> Balance;

    fn get_party_account(market_id: &MarketId) -> AccountId;
}
