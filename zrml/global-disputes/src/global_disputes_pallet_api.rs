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

extern crate alloc;

use alloc::vec::Vec;
use sp_runtime::{DispatchError, DispatchResult};
use zeitgeist_primitives::types::OutcomeReport;

/// The trait to initiate and resolve the global disputes.
pub trait GlobalDisputesPalletApi<MarketId, AccountId, Balance, BlockNumber> {
    /// Push a voting outcome for one global dispute.
    ///
    /// # Arguments
    /// - `market_id` - The id of the market.
    /// - `outcome` - The voting outcome to push.
    /// - `owner` - The owner of the outcome.
    /// - `initial_vote_balance` - The initial vote amount for the specified outcome.
    fn push_vote_outcome(
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

    /// Check if global dispute already exists.
    ///
    /// # Arguments
    /// - `market_id` - The id of the market.
    fn does_exist(market_id: &MarketId) -> bool;

    /// Check if global dispute is active or initialized. But not finished.
    /// This call is useful to check if a global dispute is ready for a destruction.
    ///
    /// # Arguments
    /// - `market_id` - The id of the market.
    fn is_unfinished(market_id: &MarketId) -> bool;

    /// Check if a global dispute does not exist.
    ///
    /// # Arguments
    /// - `market_id` - The id of the market.
    fn does_not_exist(market_id: &MarketId) -> bool {
        !Self::does_exist(market_id)
    }

    /// Start a global dispute.
    ///
    /// # Arguments
    /// - `market_id` - The id of the market.
    fn start_global_dispute(market_id: &MarketId) -> Result<u32, DispatchError>;

    /// Destroy a global dispute and allow to return all funds of the participants.
    ///
    /// # Arguments
    /// - `market_id` - The id of the market.
    fn destroy_global_dispute(market_id: &MarketId) -> Result<(), DispatchError>;
}
