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
use sp_runtime::{DispatchError, DispatchResult};

/// The trait for handling of crowdfunds.
pub trait CrowdfundPalletApi<AccountId, Balance, FundItem, NegativeImbalance> {
    /// Create a new crowdfund.
    /// 
    /// # Returns
    /// - `FundIndex` - The id of the crowdfund.
    fn open_crowdfund() -> Result<FundIndex, DispatchError>;

    /// Get an iterator over all items of a crowdfund.
    /// 
    /// # Arguments
    /// - `fund_index` - The id of the crowdfund.
    /// 
    /// # Returns
    /// - `PrefixIterator` - The iterator over all items of the crowdfund.
    fn iter_items(
        fund_index: FundIndex,
    ) -> frame_support::storage::PrefixIterator<(FundItem, FundItemInfo<Balance>)>;

    /// Prepare for all related backers to potentially refund their stake.
    /// 
    /// # Arguments
    /// - `fund_index` - The id of the crowdfund.
    /// - `item` - The item to refund.
    /// - `fee` - The overall fee to charge from the fund item
    ///  before the backer refunds are possible.
    /// 
    /// # Returns
    /// - `NegativeImbalance` - The imbalance that contains the charged fees.
    fn prepare_refund(
        fund_index: FundIndex,
        item: &FundItem,
        fee: sp_runtime::Percent,
    ) -> Result<NegativeImbalance, DispatchError>;

    /// Close a crowdfund.
    ///
    /// # Arguments
    /// - `fund_index` - The id of the crowdfund.
    fn close_crowdfund(fund_index: FundIndex) -> DispatchResult;
}
