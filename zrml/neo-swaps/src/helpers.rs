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

#![cfg(all(feature = "mock", test))]

use crate::{BalanceOf, Config, MIN_SPOT_PRICE};
use sp_runtime::SaturatedConversion;
use zeitgeist_primitives::math::fixed::{BaseProvider, ZeitgeistBase};

pub(crate) fn create_spot_prices<T>(asset_count: u16) -> Vec<BalanceOf<T>>
where
    T: Config,
{
    let mut result = vec![MIN_SPOT_PRICE.saturated_into(); (asset_count - 1) as usize];
    // Price distribution has no bearing on the benchmarks.
    let remaining_u128 =
        ZeitgeistBase::<u128>::get().unwrap() - (asset_count - 1) as u128 * MIN_SPOT_PRICE;
    result.push(remaining_u128.saturated_into());
    result
}
