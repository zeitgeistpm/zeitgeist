// Copyright 2024 Forecasting Technologies LTD.
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
use sp_runtime::DispatchError;

pub(crate) trait ComboMathOps<T>
where
    T: Config,
{
    fn calculate_swap_amount_out_for_buy(
        _buy: Vec<BalanceOf<T>>,
        _sell: Vec<BalanceOf<T>>,
        _amount_in: BalanceOf<T>,
        _liquidity: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError>;

    fn calculate_swap_amount_out_for_sell(
        _buy: Vec<BalanceOf<T>>,
        _sell: Vec<BalanceOf<T>>,
        _amount_in: BalanceOf<T>,
        _liquidity: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError>;
}
