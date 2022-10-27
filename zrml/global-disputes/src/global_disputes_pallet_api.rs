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

use sp_runtime::DispatchResult;
use zeitgeist_primitives::types::OutcomeReport;

/// The trait to initiate and resolve the global disputes.
pub trait GlobalDisputesPalletApi<MarketId, AccountId, Balance> {
    /// Push a voting outcome for one global dispute.
    ///
    /// # Arguments
    /// - `market_id` - The id of the market.
    /// - `outcome` - The voting outcome to push.
    /// - `owner` - The owner of the outcome.
    /// - `initial_vote_balance` - The initial vote amount for the specified outcome.
    ///
    /// # Returns
    ///
    /// Returns the dispute mechanism's report if available, otherwise `None`. If `None` is
    /// returned, this means that the dispute could not be resolved.
    fn push_voting_outcome(
        market_id: &MarketId,
        outcome: OutcomeReport,
        owner: &AccountId,
        initial_vote_balance: Balance,
    ) -> DispatchResult;

    /// Get the information about a voting outcome for a global dispute.
    ///
    /// # Arguments
    /// - `market_id` - The id of the market.
    /// - `outcome` - The voting outcome to get.
    ///
    /// # Returns
    ///
    /// Returns the information stored for a particular outcome.
    /// - outcome_sum - The current sum of all locks on this outcome.
    /// - owners - The vector of owners of the outcome.
    fn get_voting_outcome_info(
        market_id: &MarketId,
        outcome: &OutcomeReport,
    ) -> Option<(Balance, Vec<AccountId>)>;

    /// Determine the winner of a global dispute.
    ///
    /// # Arguments
    /// - `market_id` - The id of the market.
    ///
    /// # Returns
    ///
    /// Returns the winning outcome.
    fn determine_voting_winner(market_id: &MarketId) -> Option<OutcomeReport>;

    /// Check if global dispute started.
    ///
    /// # Arguments
    /// - `market_id` - The id of the market.
    fn is_started(market_id: &MarketId) -> bool;

    /// Check if a global dispute has not already been started.
    ///
    /// # Arguments
    /// - `market_id` - The id of the market.
    fn is_not_started(market_id: &MarketId) -> bool {
        !Self::is_started(market_id)
    }
}
