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

/// The trait to initiate and resolve the global disputes.
pub trait CrowdfundPalletApi<AccountId, Balance, FundItem> {
    fn open_crowdfund() -> Result<FundIndex, DispatchError>;

    fn iter_items(
        fund_index: FundIndex,
    ) -> frame_support::storage::PrefixIterator<(FundItem, FundItemInfo<Balance>)>;

    fn set_item_status(
        fund_index: FundIndex,
        item: &FundItem,
        status: FundItemStatus,
    ) -> DispatchResult;

    /// Close a crowdfund.
    ///
    /// # Arguments
    /// - `fund_index` - The id of the crowdfund.
    fn close_crowdfund(fund_index: FundIndex) -> DispatchResult;

    /// Query the crowdfund account.
    ///
    /// # Returns
    ///
    /// Returns the crowdfund account.
    fn get_fund_account() -> AccountId;
}
