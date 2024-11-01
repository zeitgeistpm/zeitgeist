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

use sp_runtime::SaturatedConversion;

pub(crate) trait LogCeil {
    fn log_ceil(&self) -> Self;
}

impl LogCeil for u16 {
    fn log_ceil(&self) -> Self {
        let x = *self;

        let bits_minus_one = u16::MAX.saturating_sub(1);
        let leading_zeros: u16 = x.leading_zeros().saturated_into();
        let floor_log2 = bits_minus_one.saturating_sub(leading_zeros);

        if x.is_power_of_two() { floor_log2 } else { floor_log2.saturating_add(1) }
    }
}
