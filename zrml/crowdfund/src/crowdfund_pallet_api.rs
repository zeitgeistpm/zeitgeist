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

use crate::types::*;
use sp_runtime::DispatchResult;
use zeitgeist_primitives::types::OutcomeReport;

/// The trait to initiate and resolve the global disputes.
pub trait CrowdfundPalletApi<MarketId, AccountId, Balance> {
    fn start_crowdfund(market_id: &MarketId) -> DispatchResult;

    fn iter_items(
        market_id: &MarketId,
    ) -> frame_support::storage::PrefixIterator<(OutcomeReport, FundItemInfo<Balance>)>;

    fn set_item_status(
        market_id: &MarketId,
        item: &OutcomeReport,
        status: FundItemStatus,
    ) -> DispatchResult;

    fn stop_crowdfund(market_id: &MarketId) -> DispatchResult;

    /// Query the crowdfund account.
    ///
    /// # Arguments
    /// - `market_id` - The id of the market.
    ///
    /// # Returns
    ///
    /// Returns the crowdfund account.
    fn get_fund_account() -> AccountId;
}
