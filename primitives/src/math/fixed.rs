// Copyright 2023 Forecasting Technologies LTD.
// Copyright 2021-2022 Zeitgeist PM LLC.
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
//
// This file incorporates work covered by the license above but
// published without copyright notice by Balancer Labs
// (<https://balancer.finance>, contact@balancer.finance) in the
// balancer-core repository
// <https://github.com/balancer-labs/balancer-core>.

use super::checked_ops_res::{CheckedAddRes, CheckedDivRes, CheckedMulRes};
use crate::constants::BASE;
use alloc::{borrow::ToOwned, format, string::ToString, vec::Vec};
use core::{convert::TryFrom, marker::PhantomData};
use fixed::{traits::Fixed, ParseFixedError};
use frame_support::dispatch::DispatchError;
use sp_arithmetic::traits::AtLeast32BitUnsigned;

pub trait BaseProvider<T> {
    fn get() -> Result<T, DispatchError>;
}

pub struct ZeitgeistBase<T>(PhantomData<T>);

// Used to avoid saturating operations.
impl<T> BaseProvider<T> for ZeitgeistBase<T>
where
    T: AtLeast32BitUnsigned,
{
    fn get() -> Result<T, DispatchError> {
        BASE.try_into()
            .map_err(|_| DispatchError::Other("ZeitgeistBase failed to convert BASE to Balance"))
    }
}

pub trait FixedMul
where
    Self: Sized,
{
    fn bmul(&self, other: Self) -> Result<Self, DispatchError>;

    fn bmul_floor(&self, other: Self) -> Result<Self, DispatchError>;

    fn bmul_ceil(&self, other: Self) -> Result<Self, DispatchError>;
}

pub trait FixedDiv
where
    Self: Sized,
{
    fn bdiv(&self, other: Self) -> Result<Self, DispatchError>;
    fn bdiv_floor(&self, other: Self) -> Result<Self, DispatchError>;
    fn bdiv_ceil(&self, other: Self) -> Result<Self, DispatchError>;
}

impl<T> FixedMul for T
where
    T: AtLeast32BitUnsigned,
{
    fn bmul(&self, other: Self) -> Result<Self, DispatchError> {
        let c0 = self.checked_mul_res(&other)?;
        let c1 = c0.checked_add_res(&ZeitgeistBase::<T>::get()?.checked_div_res(&2u8.into())?)?;
        c1.checked_div_res(&ZeitgeistBase::get()?)
    }

    fn bmul_floor(&self, other: Self) -> Result<Self, DispatchError> {
        self.bmul(other) // TODO
    }

    fn bmul_ceil(&self, other: Self) -> Result<Self, DispatchError> {
        self.bmul(other) // TODO
    }
}

impl<T> FixedDiv for T
where
    T: AtLeast32BitUnsigned,
{
    fn bdiv(&self, other: Self) -> Result<Self, DispatchError> {
        let c0 = self.checked_mul_res(&ZeitgeistBase::get()?)?;
        let c1 = c0.checked_add_res(&other.checked_div_res(&2u8.into())?)?;
        c1.checked_div_res(&other)
    }

    fn bdiv_floor(&self, other: Self) -> Result<Self, DispatchError> {
        self.bdiv(other) // TODO
    }

    fn bdiv_ceil(&self, other: Self) -> Result<Self, DispatchError> {
        self.bdiv(other) // TODO
    }
}

/// Converts a fixed point decimal number into another type.
pub trait FromFixedDecimal<N: Into<u128>>
where
    Self: Sized,
{
    /// Craft a fixed point decimal number from `N`.
    fn from_fixed_decimal(decimal: N, places: u8) -> Result<Self, ParseFixedError>;
}

/// Converts a fixed point decimal number into another type.
pub trait IntoFixedFromDecimal<F> {
    /// Converts a fixed point decimal number into another type.
    fn to_fixed_from_fixed_decimal(self, places: u8) -> Result<F, ParseFixedError>;
}

/// Converts a type into a fixed point decimal number.
pub trait FromFixedToDecimal<F>
where
    Self: Sized + TryFrom<u128>,
{
    /// Craft a fixed point decimal number from another type.
    fn from_fixed_to_fixed_decimal(fixed: F, places: u8) -> Result<Self, &'static str>;
}

/// Converts a type into a fixed point decimal number.
pub trait IntoFixedDecimal<N: TryFrom<u128>> {
    /// Converts a type into a fixed point decimal number.
    fn to_fixed_decimal(self, places: u8) -> Result<N, &'static str>;
}

impl<F: Fixed, N: Into<u128>> FromFixedDecimal<N> for F {
    /// Craft a `Fixed` type from a fixed point decimal number of type `N`
    fn from_fixed_decimal(decimal: N, places: u8) -> Result<Self, ParseFixedError> {
        let decimal_u128 = decimal.into();
        let mut decimal_string = decimal_u128.to_string();

        if decimal_string.len() <= places as usize {
            // This can never underflow (places >= len). Saturating subtraction to satisfy clippy.
            decimal_string = "0.".to_owned()
                + &"0".repeat((places as usize).saturating_sub(decimal_string.len()))
                + &decimal_string;
        } else {
            // This can never underflow (len > places). Saturating subtraction to satisfy clippy.
            decimal_string.insert(decimal_string.len().saturating_sub(places as usize), '.');
        }

        F::from_str(&decimal_string)
    }
}

impl<F, N> IntoFixedFromDecimal<F> for N
where
    F: Fixed + FromFixedDecimal<Self>,
    N: Into<u128>,
{
    /// Converts a fixed point decimal number into `Fixed` type (e.g. `Balance` -> `Fixed`).
    fn to_fixed_from_fixed_decimal(self, places: u8) -> Result<F, ParseFixedError> {
        F::from_fixed_decimal(self, places)
    }
}

impl<F: Fixed + ToString, N: TryFrom<u128>> FromFixedToDecimal<F> for N {
    fn from_fixed_to_fixed_decimal(fixed: F, decimals: u8) -> Result<N, &'static str> {
        let decimals_usize = decimals as usize;
        let s = fixed.to_string();
        let mut parts: Vec<&str> = s.split('.').collect();
        // If there's no fractional part, then `fixed` was an integer.
        if parts.len() != 2 {
            parts.push("0");
        }

        let (int_part, frac_part) = (parts[0], parts[1]);
        let mut increment = false;

        let new_frac_part = if frac_part.len() < decimals_usize {
            format!("{}{}", frac_part, "0".repeat(decimals_usize.saturating_sub(frac_part.len())))
        } else {
            // Adding rounding behavior
            let round_digit = frac_part.chars().nth(decimals_usize);
            match round_digit {
                Some(d) if d >= '5' => increment = true,
                _ => {}
            }

            frac_part.chars().take(decimals_usize).collect()
        };

        let mut fixed_decimal: u128 = format!("{}{}", int_part, new_frac_part)
            .parse::<u128>()
            .map_err(|_| "Failed to parse the fixed decimal representation into u128")?;

        if increment {
            fixed_decimal = fixed_decimal.saturating_add(1);
        }

        let result: N = fixed_decimal.try_into().map_err(|_| {
            "The parsed fixed decimal representation does not fit into the target type"
        })?;
        Ok(result)
    }
}

impl<F, N> IntoFixedDecimal<N> for F
where
    F: Fixed,
    N: FromFixedToDecimal<Self>,
{
    /// Converts a `Fixed` type into a fixed point decimal number.
    fn to_fixed_decimal(self, places: u8) -> Result<N, &'static str> {
        N::from_fixed_to_fixed_decimal(self, places)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_approx;
    use fixed::{traits::ToFixed, FixedU128};
    use test_case::test_case;
    use typenum::U80;

    #[test_case(0, 10, 0.0)]
    #[test_case(1, 10, 0.0000000001)]
    #[test_case(9, 10, 0.0000000009)]
    #[test_case(123_456_789, 10, 0.123456789)]
    #[test_case(999_999_999, 10, 0.999999999)]
    #[test_case(10_000_000_000, 10, 1.0)]
    #[test_case(10_000_000_001, 10, 1.00000000001)]
    #[test_case(20751874964, 10, 2.075_187_496_394_219)]
    #[test_case(123456789876543210, 10, 12_345_678.987_654_32)]
    #[test_case(99999999999999999999999, 10, 9999999999999.9999999999)]
    // Tests taken from Rikiddo pallet
    #[test_case(1, 10, 0.000_000_000_1)]
    #[test_case(123_456_789, 10, 0.012_345_678_9)]
    #[test_case(9_999, 2, 99.99)]
    #[test_case(736_101, 2, 7_361.01)]
    #[test_case(133_733_333_333, 8, 1_337.333_333_33)]
    #[test_case(1, 1, 0.1)]
    #[test_case(55, 11, 0.000_000_000_6)]
    #[test_case(34, 11, 0.000_000_000_3)]
    fn to_fixed_from_fixed_decimal(value: u128, decimals: u8, expected_float: f64) {
        let result: FixedU128<U80> = value.to_fixed_from_fixed_decimal(decimals).unwrap();
        assert_approx!(result, <FixedU128<U80>>::from_num(expected_float), 1);
    }

    #[test_case(0.0, 10, 0)]
    #[test_case(0.00000000004, 10, 0)]
    #[test_case(0.00000000005, 10, 1)]
    #[test_case(0.0000000001, 10, 1)]
    #[test_case(0.00000000099, 10, 10)]
    #[test_case(0.0123456789, 10, 123_456_789)]
    #[test_case(0.09999999999, 10, 1_000_000_000)]
    #[test_case(0.19999999999, 10, 2_000_000_000)]
    #[test_case(0.99999999999, 10, 10_000_000_000)]
    #[test_case(1.0, 10, 10_000_000_000)]
    #[test_case(1.00000000001, 10, 10_000_000_000)]
    #[test_case(1.67899999995, 10, 16_790_000_000)]
    #[test_case(1.89999999995, 10, 19_000_000_000)]
    #[test_case(1.99999999995, 10, 20_000_000_000)]
    #[test_case(2.075_187_496_394_219, 10, 20751874964)]
    #[test_case(12_345_678.987_654_32, 10, 123456789876543210)]
    #[test_case(99.999999999999, 10, 1_000_000_000_000)]
    #[test_case(9999999999999.9999999999, 10, 99999999999999999999999)]
    // Tests taken from Rikiddo pallet
    #[test_case(32.5, 0, 33)]
    #[test_case(32.25, 0, 32)]
    #[test_case(200.0, 8, 20_000_000_000)]
    #[test_case(200.1234, 8, 20_012_340_000)]
    #[test_case(200.1234, 2, 20_012)]
    #[test_case(200.1254, 2, 20_013)]
    #[test_case(123.456, 3, 123_456)]
    #[test_case(123.0, 0, 123)]
    // Random values
    #[test_case(0.1161, 3, 116)]
    #[test_case(0.2449, 3, 245)]
    #[test_case(0.29, 3, 290)]
    #[test_case(0.297, 3, 297)]
    #[test_case(0.3423, 3, 342)]
    #[test_case(0.4259, 3, 426)]
    #[test_case(0.4283, 3, 428)]
    #[test_case(0.4317, 3, 432)]
    #[test_case(0.4649, 3, 465)]
    #[test_case(0.4924, 3, 492)]
    #[test_case(0.5656, 3, 566)]
    #[test_case(0.7197, 3, 720)]
    #[test_case(0.9803, 3, 980)]
    #[test_case(1.0285, 3, 1029)]
    #[test_case(1.0661, 3, 1066)]
    #[test_case(1.0701, 3, 1070)]
    #[test_case(1.1505, 3, 1151)]
    #[test_case(1.1814, 3, 1181)]
    #[test_case(1.2284, 3, 1228)]
    #[test_case(1.3549, 3, 1355)]
    #[test_case(1.3781, 3, 1378)]
    #[test_case(1.3987, 3, 1399)]
    #[test_case(1.5239, 3, 1524)]
    #[test_case(1.5279, 3, 1528)]
    #[test_case(1.5636, 3, 1564)]
    #[test_case(1.5688, 3, 1569)]
    #[test_case(1.6275, 3, 1628)]
    #[test_case(1.6567, 3, 1657)]
    #[test_case(1.7245, 3, 1725)]
    #[test_case(1.7264, 3, 1726)]
    #[test_case(1.7884, 3, 1788)]
    #[test_case(1.8532, 3, 1853)]
    #[test_case(2.0569, 3, 2057)]
    #[test_case(2.0801, 3, 2080)]
    #[test_case(2.1192, 3, 2119)]
    #[test_case(2.1724, 3, 2172)]
    #[test_case(2.2966, 3, 2297)]
    #[test_case(2.3375, 3, 2338)]
    #[test_case(2.3673, 3, 2367)]
    #[test_case(2.4284, 3, 2428)]
    #[test_case(2.431, 3, 2431)]
    #[test_case(2.4724, 3, 2472)]
    #[test_case(2.5036, 3, 2504)]
    #[test_case(2.5329, 3, 2533)]
    #[test_case(2.5976, 3, 2598)]
    #[test_case(2.625, 3, 2625)]
    #[test_case(2.7198, 3, 2720)]
    #[test_case(2.7713, 3, 2771)]
    #[test_case(2.8375, 3, 2838)]
    #[test_case(2.9222, 3, 2922)]
    #[test_case(2.9501, 3, 2950)]
    #[test_case(2.9657, 3, 2966)]
    #[test_case(3.0959, 3, 3096)]
    #[test_case(3.182, 3, 3182)]
    #[test_case(3.216, 3, 3216)]
    #[test_case(3.2507, 3, 3251)]
    #[test_case(3.3119, 3, 3312)]
    #[test_case(3.338, 3, 3338)]
    #[test_case(3.473, 3, 3473)]
    #[test_case(3.5163, 3, 3516)]
    #[test_case(3.5483, 3, 3548)]
    #[test_case(3.6441, 3, 3644)]
    #[test_case(3.7228, 3, 3723)]
    #[test_case(3.7712, 3, 3771)]
    #[test_case(3.7746, 3, 3775)]
    #[test_case(3.8729, 3, 3873)]
    #[test_case(3.8854, 3, 3885)]
    #[test_case(3.935, 3, 3935)]
    #[test_case(3.9437, 3, 3944)]
    #[test_case(3.9872, 3, 3987)]
    #[test_case(4.0136, 3, 4014)]
    #[test_case(4.069, 3, 4069)]
    #[test_case(4.0889, 3, 4089)]
    #[test_case(4.2128, 3, 4213)]
    #[test_case(4.2915, 3, 4292)]
    #[test_case(4.3033, 3, 4303)]
    #[test_case(4.3513, 3, 4351)]
    #[test_case(4.3665, 3, 4367)]
    #[test_case(4.3703, 3, 4370)]
    #[test_case(4.4216, 3, 4422)]
    #[test_case(4.4768, 3, 4477)]
    #[test_case(4.5022, 3, 4502)]
    #[test_case(4.5236, 3, 4524)]
    #[test_case(4.5336, 3, 4534)]
    #[test_case(4.5371, 3, 4537)]
    #[test_case(4.5871, 3, 4587)]
    #[test_case(4.696, 3, 4696)]
    #[test_case(4.6967, 3, 4697)]
    #[test_case(4.775, 3, 4775)]
    #[test_case(4.7977, 3, 4798)]
    #[test_case(4.825, 3, 4825)]
    #[test_case(4.8334, 3, 4833)]
    #[test_case(4.8335, 3, 4834)]
    #[test_case(4.8602, 3, 4860)]
    #[test_case(4.9123, 3, 4912)]
    #[test_case(5.0153, 3, 5015)]
    #[test_case(5.143, 3, 5143)]
    #[test_case(5.1701, 3, 5170)]
    #[test_case(5.1721, 3, 5172)]
    #[test_case(5.1834, 3, 5183)]
    #[test_case(5.2639, 3, 5264)]
    #[test_case(5.2667, 3, 5267)]
    #[test_case(5.2775, 3, 5278)]
    #[test_case(5.3815, 3, 5382)]
    #[test_case(5.4786, 3, 5479)]
    #[test_case(5.4879, 3, 5488)]
    #[test_case(5.4883, 3, 5488)]
    #[test_case(5.494, 3, 5494)]
    #[test_case(5.5098, 3, 5510)]
    #[test_case(5.5364, 3, 5536)]
    #[test_case(5.5635, 3, 5564)]
    #[test_case(5.5847, 3, 5585)]
    #[test_case(5.6063, 3, 5606)]
    #[test_case(5.6352, 3, 5635)]
    #[test_case(5.6438, 3, 5644)]
    #[test_case(5.7062, 3, 5706)]
    #[test_case(5.7268, 3, 5727)]
    #[test_case(5.7535, 3, 5754)]
    #[test_case(5.8718, 3, 5872)]
    #[test_case(5.8901, 3, 5890)]
    #[test_case(5.956, 3, 5956)]
    #[test_case(5.9962, 3, 5996)]
    #[test_case(6.1368, 3, 6137)]
    #[test_case(6.1665, 3, 6167)]
    #[test_case(6.2001, 3, 6200)]
    #[test_case(6.286, 3, 6286)]
    #[test_case(6.2987, 3, 6299)]
    #[test_case(6.3282, 3, 6328)]
    #[test_case(6.3284, 3, 6328)]
    #[test_case(6.3707, 3, 6371)]
    #[test_case(6.3897, 3, 6390)]
    #[test_case(6.5623, 3, 6562)]
    #[test_case(6.5701, 3, 6570)]
    #[test_case(6.6014, 3, 6601)]
    #[test_case(6.6157, 3, 6616)]
    #[test_case(6.6995, 3, 6700)]
    #[test_case(6.7213, 3, 6721)]
    #[test_case(6.8694, 3, 6869)]
    #[test_case(6.932, 3, 6932)]
    #[test_case(6.9411, 3, 6941)]
    #[test_case(7.0225, 3, 7023)]
    #[test_case(7.032, 3, 7032)]
    #[test_case(7.1557, 3, 7156)]
    #[test_case(7.1647, 3, 7165)]
    #[test_case(7.183, 3, 7183)]
    #[test_case(7.1869, 3, 7187)]
    #[test_case(7.2222, 3, 7222)]
    #[test_case(7.2293, 3, 7229)]
    #[test_case(7.4952, 3, 7495)]
    #[test_case(7.563, 3, 7563)]
    #[test_case(7.5905, 3, 7591)]
    #[test_case(7.7602, 3, 7760)]
    #[test_case(7.7763, 3, 7776)]
    #[test_case(7.8228, 3, 7823)]
    #[test_case(7.8872, 3, 7887)]
    #[test_case(7.9229, 3, 7923)]
    #[test_case(7.9928, 3, 7993)]
    #[test_case(8.0465, 3, 8047)]
    #[test_case(8.0572, 3, 8057)]
    #[test_case(8.0623, 3, 8062)]
    #[test_case(8.0938, 3, 8094)]
    #[test_case(8.145, 3, 8145)]
    #[test_case(8.1547, 3, 8155)]
    #[test_case(8.162, 3, 8162)]
    #[test_case(8.1711, 3, 8171)]
    #[test_case(8.2104, 3, 8210)]
    #[test_case(8.2124, 3, 8212)]
    #[test_case(8.2336, 3, 8234)]
    #[test_case(8.2414, 3, 8241)]
    #[test_case(8.3364, 3, 8336)]
    #[test_case(8.5011, 3, 8501)]
    #[test_case(8.5729, 3, 8573)]
    #[test_case(8.7035, 3, 8704)]
    #[test_case(8.882, 3, 8882)]
    #[test_case(8.8834, 3, 8883)]
    #[test_case(8.8921, 3, 8892)]
    #[test_case(8.9127, 3, 8913)]
    #[test_case(8.9691, 3, 8969)]
    #[test_case(8.9782, 3, 8978)]
    #[test_case(9.0893, 3, 9089)]
    #[test_case(9.1449, 3, 9145)]
    #[test_case(9.1954, 3, 9195)]
    #[test_case(9.241, 3, 9241)]
    #[test_case(9.3169, 3, 9317)]
    #[test_case(9.3172, 3, 9317)]
    #[test_case(9.406, 3, 9406)]
    #[test_case(9.4351, 3, 9435)]
    #[test_case(9.5563, 3, 9556)]
    #[test_case(9.5958, 3, 9596)]
    #[test_case(9.6461, 3, 9646)]
    #[test_case(9.6985, 3, 9699)]
    #[test_case(9.7331, 3, 9733)]
    #[test_case(9.7433, 3, 9743)]
    #[test_case(9.7725, 3, 9773)]
    #[test_case(9.8178, 3, 9818)]
    #[test_case(9.8311, 3, 9831)]
    #[test_case(9.8323, 3, 9832)]
    #[test_case(9.8414, 3, 9841)]
    #[test_case(9.88, 3, 9880)]
    #[test_case(9.9107, 3, 9911)]
    fn to_fixed_decimal_works(value_float: f64, decimals: u8, expected: u128) {
        let value_fixed: FixedU128<U80> = value_float.to_fixed();
        let result: u128 = value_fixed.to_fixed_decimal(decimals).unwrap();
        // We allow for a small error because some floats like 9.9665 are actually 9.9664999... and
        // round down instead of up.
        assert_approx!(result, expected, 1);
    }
}

#[macro_export]
macro_rules! assert_approx {
    ($left:expr, $right:expr, $precision:expr $(,)?) => {
        match (&$left, &$right, &$precision) {
            (left_val, right_val, precision_val) => {
                let diff = if *left_val > *right_val {
                    *left_val - *right_val
                } else {
                    *right_val - *left_val
                };
                if diff > *precision_val {
                    panic!(
                        "assertion `left approx== right` failed\n      left: {}\n     right: {}\n \
                         precision: {}\ndifference: {}",
                        *left_val, *right_val, *precision_val, diff
                    );
                }
            }
        }
    };
}
