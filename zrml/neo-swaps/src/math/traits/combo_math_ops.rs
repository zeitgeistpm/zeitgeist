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

use crate::{BalanceOf, Config};
use alloc::vec::Vec;
use sp_runtime::DispatchError;

pub(crate) trait ComboMathOps<T>
where
    T: Config,
{
    /// Calculates the amount swapped out of a pool with liquidity parameter `liquidity` when
    /// swapping in `amount_in` units of assets whose reserves in the pool are `sell` and swapping
    /// out assets whose reserves in the pool are `buy`.
    fn calculate_swap_amount_out_for_buy(
        buy: Vec<BalanceOf<T>>,
        sell: Vec<BalanceOf<T>>,
        amount_in: BalanceOf<T>,
        liquidity: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError>;

    /// Calculates the amount eventually held by the user after equalizing holdings.
    #[allow(dead_code)]
    fn calculate_equalize_amount(
        buy: Vec<BalanceOf<T>>,
        sell: Vec<BalanceOf<T>>,
        amount_buy: BalanceOf<T>,
        amount_sell: BalanceOf<T>,
        liquidity: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError>;

    /// Calculates the amount of each asset of a pool with liquidity parameter `liquidity` held
    /// in the user's wallet after equalizing positions whose reserves in the pool are `buy`, `keep`
    /// and `sell`, resp. The parameters `amount_buy` and `amount_keep` refer to the user's holdings
    /// of `buy` and `keep`.
    fn calculate_swap_amount_out_for_sell(
        buy: Vec<BalanceOf<T>>,
        keep: Vec<BalanceOf<T>>,
        sell: Vec<BalanceOf<T>>,
        amount_buy: BalanceOf<T>,
        amount_keep: BalanceOf<T>,
        liquidity: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError>;

    #[allow(dead_code)]
    fn calculate_spot_price(
        buy: Vec<BalanceOf<T>>,
        sell: Vec<BalanceOf<T>>,
        liquidity: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError>;
}
