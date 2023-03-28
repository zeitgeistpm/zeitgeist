// Copyright 2022-2023 Forecasting Technologies LTD.
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

use crate::types::InitialItem;
use sp_runtime::DispatchError;
use zeitgeist_primitives::types::OutcomeReport;

/// The trait to initiate and resolve the global disputes.
pub trait GlobalDisputesPalletApi<MarketId, AccountId, Balance, BlockNumber> {
    /// Return the `AddOutcomePeriod` parameter.
    fn get_add_outcome_period() -> BlockNumber;

    /// Return the `GdVotingPeriod` parameter.
    fn get_vote_period() -> BlockNumber;

    /// Start a global dispute.
    ///
    /// # Arguments
    /// - `market_id` - The id of the market.
    /// - `initial_items` - The initial vote options (outcome, owner, amount)
    /// to add to the global dispute. One initial item consists of the vote outcome,
    ///  the owner of the outcome who is rewarded in case of a win,
    /// and the initial vote amount for this outcome.
    /// It is required to add at least two unique outcomes.
    /// In case of a duplicated outcome, the owner and amount is added to the pre-existing outcome.
    fn start_global_dispute(
        market_id: &MarketId,
        initial_item: &[InitialItem<AccountId, Balance>],
    ) -> Result<u32, DispatchError>;

    /// Determine the winner of a global dispute.
    ///
    /// # Arguments
    /// - `market_id` - The id of the market.
    ///
    /// # Returns
    ///
    /// Returns the winning outcome.
    fn determine_voting_winner(market_id: &MarketId) -> Option<OutcomeReport>;

    /// Check if a global dispute exists for the specified market.
    fn does_exist(market_id: &MarketId) -> bool;

    /// Check if global dispute is active.
    /// This call is useful to check if a global dispute is ready for a destruction.
    ///
    /// # Arguments
    /// - `market_id` - The id of the market.
    fn is_active(market_id: &MarketId) -> bool;

    /// Destroy a global dispute and allow to return all funds of the participants.
    ///
    /// # Arguments
    /// - `market_id` - The id of the market.
    fn destroy_global_dispute(market_id: &MarketId) -> Result<(), DispatchError>;
}
