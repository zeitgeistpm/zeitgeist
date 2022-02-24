//! This module contains a collection of types that are required to implement the Rikiddo core
//! functionality, as well as the Rikiddo core functionality itself.

extern crate alloc;
use super::{
    traits::{FromFixedDecimal, FromFixedToDecimal, IntoFixedDecimal, IntoFixedFromDecimal},
    utils::{fixed_zero, max_value_u128},
};
use alloc::{borrow::ToOwned, string::ToString};
#[cfg(feature = "arbitrary")]
use arbitrary::{Arbitrary, Result as ArbiraryResult, Unstructured};
#[cfg(feature = "arbitrary")]
use core::mem;
use core::{cmp::Ordering, convert::TryFrom};
use frame_support::dispatch::{Decode, Encode};
use sp_core::RuntimeDebug;
use substrate_fixed::{
    traits::{Fixed, FixedSigned, FixedUnsigned, LossyFrom, LossyInto, ToFixed},
    types::extra::{U127, U128},
    FixedI128, FixedU128, ParseFixedError,
};
#[cfg(feature = "arbitrary")]
use substrate_fixed::{
    types::extra::{LeEqU128, LeEqU16, LeEqU32, LeEqU64, LeEqU8},
    FixedI16, FixedI32, FixedI64, FixedI8, FixedU16, FixedU32, FixedU64, FixedU8,
};

mod ema_market_volume;
mod rikiddo_sigmoid_mv;
mod sigmoid_fee;

pub use ema_market_volume::*;
pub use rikiddo_sigmoid_mv::*;
pub use sigmoid_fee::*;

/// A timestamp that contains the seconds since January 1st, 1970 at UTC.
pub type UnixTimestamp = u64;

/// A 2-tuple containing an unix timestamp and a volume.
#[derive(scale_info::TypeInfo, Clone, RuntimeDebug, Decode, Default, Encode, Eq, PartialEq)]
pub struct TimestampedVolume<F: Fixed> {
    /// The timestamp of the volume.
    pub timestamp: UnixTimestamp,
    /// The volume.
    pub volume: F,
}

#[cfg(feature = "arbitrary")]
macro_rules! impl_arbitrary_for_timestamped_volume {
    ( $t:ident, $LeEqU:ident, $p:ty ) => {
        #[allow(clippy::integer_arithmetic)]
        impl<'a, Frac> Arbitrary<'a> for TimestampedVolume<$t<Frac>>
        where
            Frac: $LeEqU,
            $t<Frac>: Fixed,
        {
            fn arbitrary(u: &mut Unstructured<'a>) -> ArbiraryResult<Self> {
                return Ok(TimestampedVolume {
                    timestamp: <UnixTimestamp as Arbitrary<'a>>::arbitrary(u)?,
                    volume: <$t<Frac>>::from_bits(<$p as Arbitrary<'a>>::arbitrary(u)?),
                });
            }

            #[inline]
            fn size_hint(_depth: usize) -> (usize, Option<usize>) {
                let bytecount_fixed = mem::size_of::<$t<Frac>>();
                let bytecount_timestamp = mem::size_of::<UnixTimestamp>();
                let required_bytes = bytecount_fixed + bytecount_timestamp;
                (required_bytes, Some(required_bytes))
            }
        }
    };
}

cfg_if::cfg_if! {
    if #[cfg(feature = "arbitrary")] {
        impl_arbitrary_for_timestamped_volume! {FixedI8, LeEqU8, i8}
        impl_arbitrary_for_timestamped_volume! {FixedI16, LeEqU16, i16}
        impl_arbitrary_for_timestamped_volume! {FixedI32, LeEqU32, i32}
        impl_arbitrary_for_timestamped_volume! {FixedI64, LeEqU64, i64}
        impl_arbitrary_for_timestamped_volume! {FixedI128, LeEqU128, i128}
        impl_arbitrary_for_timestamped_volume! {FixedU8, LeEqU8, u8}
        impl_arbitrary_for_timestamped_volume! {FixedU16, LeEqU16, u16}
        impl_arbitrary_for_timestamped_volume! {FixedU32, LeEqU32, u32}
        impl_arbitrary_for_timestamped_volume! {FixedU64, LeEqU64, u64}
        impl_arbitrary_for_timestamped_volume! {FixedU128, LeEqU128, u128}
    }
}

/// A enum that wrappes an amount of time in different units.
/// An enum that wrappes an amount of time in different units.
#[derive(
    scale_info::TypeInfo, Copy, Clone, RuntimeDebug, Decode, Encode, Eq, PartialEq, PartialOrd,
)]
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
pub enum Timespan {
    /// Contains seconds.
    Seconds(u32),
    /// Contains minutes.
    Minutes(u32),
    /// Contains hours.
    Hours(u32),
    /// Contains days.
    Days(u16),
    /// Contains weeks.
    Weeks(u16),
}

impl Timespan {
    /// Convert the current `Timespan` into a number of seconds.
    pub fn to_seconds(&self) -> u32 {
        match *self {
            // Any value that leads to a saturation is greater than
            // 4294967295 seconds, which is about 136 years.
            Timespan::Seconds(d) => d,
            Timespan::Minutes(d) => d.saturating_mul(60),
            Timespan::Hours(d) => d.saturating_mul(60).saturating_mul(60),
            Timespan::Days(d) => {
                u32::from(d).saturating_mul(60).saturating_mul(60).saturating_mul(24)
            }
            Timespan::Weeks(d) => u32::from(d)
                .saturating_mul(60)
                .saturating_mul(60)
                .saturating_mul(24)
                .saturating_mul(7),
        }
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

    if let Some(res) = num_u128.checked_to_fixed() {
        Ok(res)
    } else {
        Err("Integer part of fixed point number does not fit into u128")
    }
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
                if let Some(two) = F::checked_from_num(2) {
                    if let Some(one) = F::checked_from_num(1) {
                        if let Some(res) = one.checked_div(two) {
                            if fixed.frac() >= res {
                                result = result.saturating_add(1);
                            }
                        } else {
                            return Err("Unexpected overflow when dividing one by two");
                        }
                    } else {
                        return Err("Unexpectedly failed to convert 1 to fixed point number");
                    }
                } else {
                    return Err("Unexpectedly failed to convert 2 to fixed point number");
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
