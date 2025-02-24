// Copyright 2024-2025 Forecasting Technologies LTD.
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

use alloc::vec::Vec;
use sp_runtime::DispatchResult;

/// Trait used for setting up benchmarks of zrml-combinatorial-tokens. Must not be used in
/// production.
pub trait CombinatorialTokensBenchmarkHelper {
    type Balance;
    type MarketId;

    /// Prepares the market with the specified `market_id` to have a particular `payout`.
    fn setup_payout_vector(
        market_id: Self::MarketId,
        payout: Option<Vec<Self::Balance>>,
    ) -> DispatchResult;
}
