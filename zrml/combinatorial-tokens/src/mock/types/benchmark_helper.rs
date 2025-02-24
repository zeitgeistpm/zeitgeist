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

use crate::{
    mock::{runtime::Runtime, types::MockPayout},
    BalanceOf, MarketIdOf,
};
use alloc::vec::Vec;
use sp_runtime::DispatchResult;
use zeitgeist_primitives::traits::CombinatorialTokensBenchmarkHelper;

pub struct BenchmarkHelper;

impl CombinatorialTokensBenchmarkHelper for BenchmarkHelper {
    type Balance = BalanceOf<Runtime>;
    type MarketId = MarketIdOf<Runtime>;

    /// A bit of a messy implementation as this sets the return value of the next `payout_vector`
    /// call, regardless of what `_market_id` is.
    fn setup_payout_vector(
        _market_id: Self::MarketId,
        payout: Option<Vec<Self::Balance>>,
    ) -> DispatchResult {
        MockPayout::set_return_value(payout);

        Ok(())
    }
}
