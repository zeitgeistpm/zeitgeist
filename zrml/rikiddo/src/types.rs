use frame_support::dispatch::{fmt::Debug, Decode, Encode};
use substrate_fixed::{FixedI128, FixedU128, ParseFixedError, traits::{Fixed, FixedSigned, FixedUnsigned, LossyFrom, LossyInto, ToFixed}, types::extra::{U127, U128}};

mod ema_market_volume;
mod rikiddo_sigmoid_mv;
mod sigmoid_fee;

pub use ema_market_volume::*;
pub use rikiddo_sigmoid_mv::*;
pub use sigmoid_fee::*;

pub type UnixTimestamp = u64;

#[derive(Clone, Debug, Decode, Default, Encode, Eq, PartialEq)]
pub struct TimestampedVolume<F: Fixed> {
    pub timestamp: UnixTimestamp,
    pub volume: F,
}

#[derive(Copy, Clone, Debug, Decode, Encode, Eq, PartialEq, PartialOrd)]
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

pub trait FromFixedDecimal<F: Fixed, N: Into<u128>> {
    fn from_fixed_decimal(decimal: N, places: u8) -> Result<F, ParseFixedError>;
}


impl<F: Fixed, N: Into<u128>> FromFixedDecimal<F, N> for F {
    fn from_fixed_decimal(decimal: N, places: u8) -> Result<F, ParseFixedError> {
        let decimal_u128 = decimal.into();
		let mut decimal_string = decimal_u128.to_string();
		// Can panic (check index)
		if decimal_string.len() <= places as usize {
		    decimal_string = "0.".to_owned() + &decimal_string;
		} else {
    		decimal_string.insert(decimal_string.len() - places as usize, '.');
		}
		
		F::from_str(&decimal_string)
    }
}
