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

use zeitgeist_primitives::types::OutcomeReport;

pub trait GlobalDisputesPalletApi<MarketId, AccountId, Balance> {
    fn push_voting_outcome(
        market_id: &MarketId,
        outcome: OutcomeReport,
        owner: &AccountId,
        vote_balance: Balance,
    );

    fn get_voting_winner(market_id: &MarketId) -> Option<OutcomeReport>;

    fn is_started(market_id: &MarketId) -> bool;
}
