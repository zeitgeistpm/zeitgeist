use frame_support::dispatch::{fmt::Debug, Decode, Encode};
use substrate_fixed::traits::{Fixed, ToFixed};

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

#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
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
            Timespan::Seconds(d) => d,
            Timespan::Minutes(d) => d * 60,
            Timespan::Hours(d) => d * 60 * 60,
            Timespan::Days(d) => u32::from(d) * 60 * 60 * 24,
            Timespan::Weeks(d) => u32::from(d) * 60 * 60 * 24 * 7,
        }
    }
}

/// Converts one Fixed number into another (fallible)
// TODO: test
fn convert<FROM: Fixed, TO: Fixed>(num: FROM) -> Result<TO, &'static str> {
        let msb_num = 1.to_fixed::<FROM>() << (FROM::int_nbits() - 1);

        // Check if msb is set
        if num & msb_num != 0 {
            return Err("Fixed conversion failed: MSB is set");
        }

        // PartialOrd is bugged, therefore the workaround
        // https://github.com/encointer/substrate-fixed/issues/9
        if TO::max_value().int().to_num::<u128>() < num.int().to_num::<u128>() {
            return Err("Fixed conversion failed: FROM type does not fit in TO type");
        }

        let integer_part_signed = i128::from_fixed(sigmoid_result.int());
        // We can safely cast because until here we know that the integer part is unsigned.
        let integer_part: Self::FOUT = (integer_part_signed as u128).to_fixed();
        let fractional_part: FixedU128<U128> = sigmoid_result.frac().to_fixed();

        if let Some(res) = integer_part.checked_add(fractional_part.lossy_into()) {
            return Ok(res);
        } else {
            // This error should be impossible to reach.
            return Err("[FeeSigmoid] Something went wrong during FIN to FOUT type conversion");
        };
}