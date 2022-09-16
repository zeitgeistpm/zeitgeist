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
    /// # Returns
    ///
    /// Returns the dispute mechanism's report if available, otherwise `None`. If `None` is
    /// returned, this means that the dispute could not be resolved.
    fn push_voting_outcome(
        market_id: &MarketId,
        outcome: OutcomeReport,
        owner: &AccountId,
        vote_balance: Balance,
    ) -> DispatchResult;

    /// Determine the winner of a global dispute.
    ///
    /// # Returns
    ///
    /// Returns the winning outcome.
    fn determine_voting_winner(market_id: &MarketId) -> Option<OutcomeReport>;

    /// Check if global dispute started.
    fn is_started(market_id: &MarketId) -> bool;

    /// Check if a global dispute has not already been started.
    fn is_not_started(market_id: &MarketId) -> bool;
}
