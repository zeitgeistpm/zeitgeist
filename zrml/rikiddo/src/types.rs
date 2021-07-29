use frame_support::dispatch::{fmt::Debug, Decode, Encode};
use substrate_fixed::{
    traits::{Fixed, FixedSigned, FixedUnsigned, LossyFrom, LossyInto, ToFixed},
    types::extra::{U127, U128},
    FixedI128, FixedU128,
};

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

pub trait DecimalConversion<F> {
	fn from_decimal(decimal: u128, places: u8) -> F;
	fn to_decimal(other: F, places: u8) -> u128;
}

impl<F: Fixed> DecimalConversion<F> for F {
	// can panic
    fn from_decimal(decimal: u128, places: u8) -> F {
		let decimal_string = decimal.to_string();
		// Can panic (check index)
		let parsed_integer = &decimal_string[..decimal_string.len() - places as usize];
		let parsed_fraction = &decimal_string[decimal_string.len() - places as usize..];
		// Panic impossible, it must contain a base 10 string, since it is the repr. of one
		let integer = u128::from_str_radix(parsed_integer, 10).unwrap();
		let fraction = u128::from_str_radix(parsed_fraction, 10).unwrap();
	    
		let base = 10u128.pow(places as u32);
		// Can panic (needs checked variant)
		let converted = F::from_num(integer);
		let mut bits = String::from("0.");
		let mut cmp_num = 0u128;
		
		for shift in 0..F::frac_nbits() {
		    let shiftval = 2 << shift;
		    //let addnum = base / (2 << shift);
		    let addnum = base / shiftval + u128::from(base % shiftval == 0);
		    cmp_num += addnum;
		    
		    if cmp_num > fraction {
		        cmp_num -= addnum;
		        bits.push('0');
		    } else {
		        bits.push('1');
		    }
        }
		
		// Can panic
		converted + F::from_str_binary(&bits).unwrap()
	}

	fn to_decimal(other: F, places: u8) -> u128 {
		42
	}
}