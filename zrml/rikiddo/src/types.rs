use frame_support::dispatch::{Decode, Encode};
use sp_core::RuntimeDebug;
use sp_std::convert::TryFrom;
use substrate_fixed::{
    traits::{Fixed, FixedSigned, FixedUnsigned, LossyFrom, LossyInto, ToFixed},
    types::extra::{U127, U128},
    FixedI128, FixedU128, ParseFixedError,
};

mod ema_market_volume;
mod rikiddo_sigmoid_mv;
mod sigmoid_fee;

pub use ema_market_volume::*;
pub use rikiddo_sigmoid_mv::*;
pub use sigmoid_fee::*;

pub type UnixTimestamp = u64;

#[derive(Clone, RuntimeDebug, Decode, Default, Encode, Eq, PartialEq)]
pub struct TimestampedVolume<F: Fixed> {
    pub timestamp: UnixTimestamp,
    pub volume: F,
}

#[derive(Copy, Clone, RuntimeDebug, Decode, Encode, Eq, PartialEq, PartialOrd)]
pub enum Timespan {
    Seconds(u32),
    Minutes(u32),
    Hours(u32),
    Days(u16),
    Weeks(u16),
}

impl Timespan {
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

/// Returns integer part of FROM, if the msb is not set and and num does it into FROM
fn convert_common<FROM: Fixed, TO: Fixed>(num: FROM) -> Result<TO, &'static str> {
    // Check if number is negatie
    if num < FROM::from_num(0u8) {
        return Err("Cannot convert negative signed number into unsigned number");
    }

    // PartialOrd is bugged, therefore the workaround
    // https://github.com/encointer/substrate-fixed/issues/9
    let num_u128: u128 = num.int().to_num();
    if TO::max_value().int().to_num::<u128>() < num_u128 {
        return Err("Fixed point conversion failed: FROM type does not fit in TO type");
    }

    Ok(num_u128.to_fixed())
}

/// Converts an unsigned fixed point number into a signed fixed point number (fallible)
pub fn convert_to_signed<FROM: FixedUnsigned, TO: FixedSigned + LossyFrom<FixedI128<U127>>>(
    num: FROM,
) -> Result<TO, &'static str> {
    let integer_part: TO = convert_common(num)?;
    let fractional_part: FixedI128<U127> = num.frac().to_fixed();

    if let Some(res) = integer_part.checked_add(fractional_part.lossy_into()) {
        Ok(res)
    } else {
        // This error should be impossible to reach.
        Err("Something went wrong during FixedUnsigned to FixedSigned type conversion")
    }
}

/// Converts a signed fixed point number into an unsigned fixed point number (fallible)
pub fn convert_to_unsigned<FROM: FixedSigned, TO: FixedUnsigned + LossyFrom<FixedU128<U128>>>(
    num: FROM,
) -> Result<TO, &'static str> {
    // We can safely cast because until here we know that the msb is not set.
    let integer_part: TO = convert_common(num)?;
    let fractional_part: FixedU128<U128> = num.frac().to_fixed();

    if let Some(res) = integer_part.checked_add(fractional_part.lossy_into()) {
        Ok(res)
    } else {
        // This error should be impossible to reach.
        Err("Something went wrong during FixedSigned to FixedUnsigned type conversion")
    }
}

/// Converts a fixed point decimal number into another type
pub trait FromFixedDecimal<N: Into<u128>>
where
    Self: Sized,
{
    fn from_fixed_decimal(decimal: N, places: u8) -> Result<Self, ParseFixedError>;
}

/// Converts a fixed point decimal number into Fixed type
impl<F: Fixed, N: Into<u128>> FromFixedDecimal<N> for F {
    fn from_fixed_decimal(decimal: N, places: u8) -> Result<Self, ParseFixedError> {
        let decimal_u128 = decimal.into();
        let mut decimal_string = decimal_u128.to_string();

        if decimal_string.len() <= places as usize {
            decimal_string = "0.".to_owned()
                + &"0".repeat(places as usize - decimal_string.len())
                + &decimal_string;
        } else {
            decimal_string.insert(decimal_string.len() - places as usize, '.');
        }

        F::from_str(&decimal_string)
    }
}

/// Converts a fixed point decimal number into Fixed type (Balance -> Fixed)
pub trait IntoFixedFromDecimal<F: Fixed> {
    fn to_fixed_from_fixed_decimal(self, places: u8) -> Result<F, ParseFixedError>;
}

/// Converts a fixed point decimal number into Fixed type
impl<F, N> IntoFixedFromDecimal<F> for N
where
    F: Fixed + FromFixedDecimal<Self>,
    N: Into<u128>,
{
    fn to_fixed_from_fixed_decimal(self, places: u8) -> Result<F, ParseFixedError> {
        F::from_fixed_decimal(self, places)
    }
}

/// Converts a Fixed type into fixed point decimal number
pub trait FromFixedToDecimal<F>
where
    Self: Sized + TryFrom<u128>,
{
    fn from_fixed_to_fixed_decimal(fixed: F, places: u8) -> Result<Self, &'static str>;
}

/// Converts a Fixed type into a fixed point decimal number (Fixed -> Balance)
impl<F: Fixed, N: TryFrom<u128>> FromFixedToDecimal<F> for N {
    fn from_fixed_to_fixed_decimal(fixed: F, places: u8) -> Result<N, &'static str> {
        if places == 0 {
            let mut result = fixed.to_num::<u128>();

            // Arithmetic rounding (+1 if >= 0.5)
            if F::frac_nbits() > 0 {
                if let Some(two) = F::checked_from_num(2) {
                    // `from_num(1)` cannot panic if `from_num(2)` succeeded
                    if let Some(res) = F::from_num(1).checked_div(two) {
                        if fixed.frac() >= res {
                            result += 1;
                        }
                    }
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

        if fixed_frac == 0 {
            // Add `places` times 0 to pad all remaining fractional decimal places
            fixed_str += &"0".repeat(places as usize);
        } else {
            let frac_string = &fixed_frac.to_string()[2..];

            if frac_string.len() < places as usize {
                fixed_str.retain(|c| c != '.');
                // Padding to the right side up to `places`
                fixed_str += &"0".repeat(places as usize - frac_string.len());
            } else if frac_string.len() > places as usize {
                // Cutting down to `places` + arithmetic rounding of the last digit
                let frac_plus_one_digit_str = &frac_string[0..places as usize + 1];

                if let Ok(mut res) = u128::from_str_radix(&frac_plus_one_digit_str, 10) {
                    let last_digit = res % 10;
                    res /= 10;

                    if last_digit >= 5 {
                        res += 1;
                    }

                    fixed_str = fixed.int().to_string() + &res.to_string()
                } else {
                    // Impossible unless there is a bug in Fixed's to_string()
                    return Err("Error parsing the string representation of the fixed point number");
                };
            } // The other case requires no changes
        }

        let result = if let Ok(res) = u128::from_str_radix(&fixed_str, 10) {
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

/// Converts a fixed point decimal number into Fixed type
pub trait IntoFixedDecimal<N: TryFrom<u128>> {
    fn to_fixed_decimal(self, places: u8) -> Result<N, &'static str>;
}

/// Converts a fixed point decimal number into Fixed type
impl<F, N> IntoFixedDecimal<N> for F
where
    F: Fixed,
    N: FromFixedToDecimal<Self>,
{
    fn to_fixed_decimal(self, places: u8) -> Result<N, &'static str> {
        N::from_fixed_to_fixed_decimal(self, places)
    }
}
