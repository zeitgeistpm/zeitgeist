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

//! This module contains utility functions for creating commonly used
//! fixed point type constants.

use super::traits::{FromFixedDecimal, FromFixedToDecimal, IntoFixedDecimal, IntoFixedFromDecimal};
use alloc::{borrow::ToOwned, string::ToString};
use core::{cmp::Ordering, convert::TryFrom};
use substrate_fixed::{
    traits::{Fixed, FixedSigned, FixedUnsigned, LossyFrom, LossyInto, ToFixed},
    types::extra::{U127, U128},
    FixedI128, FixedU128, ParseFixedError,
};

/// Create a fixed point number that represents 0 (zero).
pub fn fixed_zero<FixedType: Fixed>() -> Result<FixedType, &'static str> {
    if let Some(res) = FixedType::checked_from_num(0) {
        Ok(res)
    } else {
        Err("Unexpectedly failed to convert zero to fixed point type")
    }
}

/// Return the maximum value of FixedType as u128.
pub fn max_value_u128<FixedType: Fixed>() -> Result<u128, &'static str> {
    if let Some(res) = FixedType::max_value().int().checked_to_num() {
        Ok(res)
    } else {
        Err("Unexpectedly failed to convert max_value of fixed point type to u128")
    }
}

/// Returns integer part of `FROM`, if the msb is not set and and num does it into `FROM`.
fn convert_common<FROM: Fixed, TO: Fixed>(num: FROM) -> Result<TO, &'static str> {
    // Check if number is negatie
    if num < fixed_zero::<FROM>()? {
        return Err("Cannot convert negative signed number into unsigned number");
    }

    // PartialOrd is bugged, therefore the workaround
    // https://github.com/encointer/substrate-fixed/issues/9
    let num_u128: u128 = num.int().checked_to_num().ok_or("Conversion from FROM to u128 failed")?;
    if max_value_u128::<TO>()? < num_u128 {
        return Err("Fixed point conversion failed: FROM type does not fit in TO type");
    }

    num_u128.checked_to_fixed().ok_or("Integer part of fixed point number does not fit into u128")
}

/// Converts an unsigned fixed point number into a signed fixed point number (fallible).
pub fn convert_to_signed<FROM: FixedUnsigned, TO: FixedSigned + LossyFrom<FixedI128<U127>>>(
    num: FROM,
) -> Result<TO, &'static str> {
    let integer_part: TO = convert_common(num)?;
    let fractional_part: FixedI128<U127> = num
        .frac()
        .checked_to_fixed()
        .ok_or("Fraction of fixed point number unexpectedly does not fit into FixedI128<U127>")?;

    if let Some(res) = integer_part.checked_add(fractional_part.lossy_into()) {
        Ok(res)
    } else {
        // This error should be impossible to reach.
        Err("Something went wrong during FixedUnsigned to FixedSigned type conversion")
    }
}

/// Converts a signed fixed point number into an unsigned fixed point number (fallible).
pub fn convert_to_unsigned<FROM: FixedSigned, TO: FixedUnsigned + LossyFrom<FixedU128<U128>>>(
    num: FROM,
) -> Result<TO, &'static str> {
    // We can safely cast because until here we know that the msb is not set.
    let integer_part: TO = convert_common(num)?;
    let fractional_part: FixedU128<U128> = num
        .frac()
        .checked_to_fixed()
        .ok_or("Fraction of fixed point number unexpectedly does not fit into FixedU128<U128>")?;
    if let Some(res) = integer_part.checked_add(fractional_part.lossy_into()) {
        Ok(res)
    } else {
        // This error should be impossible to reach.
        Err("Something went wrong during FixedSigned to FixedUnsigned type conversion")
    }
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
            // This can never underflow (len >= places). Saturating subtraction to satisfy clippy.
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

impl<F: Fixed, N: TryFrom<u128>> FromFixedToDecimal<F> for N {
    /// Craft a fixed point decimal number from a `Fixed` type (e.g. `Fixed` -> `Balance`).
    fn from_fixed_to_fixed_decimal(fixed: F, places: u8) -> Result<N, &'static str> {
        if places == 0 {
            let mut result = fixed
                .checked_to_num::<u128>()
                .ok_or("The fixed point number does not fit into a u128")?;

            // Arithmetic rounding (+1 if >= 0.5)
            if F::frac_nbits() > 0 {
                let two = F::checked_from_num(2)
                    .ok_or("Unexpectedly failed to convert 2 to fixed point number")?;
                let one = F::checked_from_num(1)
                    .ok_or("Unexpectedly failed to convert 1 to fixed point number")?;
                let half =
                    one.checked_div(two).ok_or("Unexpected overflow when dividing one by two")?;

                if fixed.frac() >= half {
                    result = result.saturating_add(1);
                }
            }

            if let Ok(res) = N::try_from(result) {
                return Ok(res);
            } else {
                return Err(
                    "The parsed fixed decimal representation does not fit into the target type"
                );
            }
        }

        let mut fixed_str = fixed.to_string();
        let fixed_frac = fixed.frac();

        let places_usize: usize = places.into();

        if fixed_frac == 0 {
            // Add `places` times 0 to pad all remaining fractional decimal places
            fixed_str += &"0".repeat(places_usize);
        } else {
            let frac_string = fixed_frac.to_string();
            let frac_str = frac_string.get(2..).unwrap_or_default();

            match frac_str.len().cmp(&places_usize) {
                Ordering::Less => {
                    fixed_str.retain(|c| c != '.');
                    // Padding to the right side up to `places`. Cannot underflow.
                    fixed_str += &"0".repeat(places_usize.saturating_sub(frac_str.len()));
                }
                Ordering::Greater => {
                    // Cutting down to `places` + arithmetic rounding of the last digit
                    let frac_plus_one_digit_str =
                        frac_str.get(..places_usize.saturating_add(1)).unwrap_or_default();

                    if let Ok(mut res) = frac_plus_one_digit_str.parse::<u128>() {
                        let last_digit = res % 10;
                        res /= 10;

                        if last_digit >= 5 {
                            res = res.saturating_add(1);
                        }

                        fixed_str = fixed.int().to_string() + &res.to_string()
                    } else {
                        // Impossible unless there is a bug in Fixed's to_string()
                        return Err(
                            "Error parsing the string representation of the fixed point number"
                        );
                    };
                }
                Ordering::Equal => fixed_str.retain(|c| c != '.'),
            };
        }

        let result = if let Ok(res) = fixed_str[..].parse::<u128>() {
            res
        } else {
            // Impossible unless there is a bug in Fixed's to_string()
            return Err("Error parsing the string representation of the fixed point number");
        };

        if let Ok(res) = N::try_from(result) {
            Ok(res)
        } else {
            Err("The parsed fixed decimal representation does not fit into the target type")
        }
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
