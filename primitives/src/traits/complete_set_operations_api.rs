// Copyright 2023 Forecasting Technologies LTD.
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

pub trait CompleteSetOperationsApi {
    type AccountId;
    type Balance;
    type MarketId;

    fn buy_complete_set(
        who: Self::AccountId,
        market_id: Self::MarketId,
        amount: Self::Balance,
    ) -> DispatchResult;
    fn sell_complete_set(
        who: Self::AccountId,
        market_id: Self::MarketId,
        amount: Self::Balance,
    ) -> DispatchResult;
}
