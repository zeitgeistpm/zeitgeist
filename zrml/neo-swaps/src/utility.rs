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

use sp_runtime::{
    traits::{One, Zero},
    SaturatedConversion,
};

pub(crate) trait LogCeil {
    /// Calculates the ceil of the log with base 2 of `self`.
    fn log_ceil(&self) -> Self;
}

impl LogCeil for u16 {
    fn log_ceil(&self) -> Self {
        let x = *self;

        if x.is_zero() {
            return One::one();
        }

        let bits_minus_one: u16 = u16::BITS.saturating_sub(1).saturated_into();
        let leading_zeros: u16 = x.leading_zeros().saturated_into();
        let floor_log2 = bits_minus_one.saturating_sub(leading_zeros);

        if x.is_power_of_two() { floor_log2 } else { floor_log2.saturating_add(1) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(0, 1)]
    #[test_case(1, 0)]
    #[test_case(2, 1)]
    #[test_case(3, 2)]
    #[test_case(4, 2)]
    #[test_case(5, 3)]
    #[test_case(6, 3)]
    #[test_case(7, 3)]
    #[test_case(8, 3)]
    #[test_case(9, 4)]
    #[test_case(15, 4)]
    #[test_case(16, 4)]
    #[test_case(17, 5)]
    #[test_case(1023, 10)]
    #[test_case(1024, 10)]
    #[test_case(1025, 11)]
    #[test_case(32767, 15)]
    #[test_case(32768, 15)]
    #[test_case(32769, 16)]
    #[test_case(65535, 16)]
    fn log_ceil_works(value: u16, expected: u16) {
        let actual = value.log_ceil();
        assert_eq!(actual, expected);
    }
}
