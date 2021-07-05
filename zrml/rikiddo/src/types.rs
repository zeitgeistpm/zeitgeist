use frame_support::dispatch::{fmt::Debug, Decode, Encode};
use substrate_fixed::traits::Fixed;

// use crate::traits::{MarketAverage, Sigmoid};

mod ema_market_volume;
mod sigmoid_fee;

pub use ema_market_volume::*;
pub use sigmoid_fee::*;

pub type UnixTimestamp = u64;

#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
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

/*
#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
pub struct RikiddoSigmoidMV<FI: Fixed, FE: Sigmoid<FI>, MA: MarketAverage<FI>> {
    pub fees: FE,
    pub ma_short: MA,
    pub ma_long: MA,
    pub _marker: PhantomData<FI>,
}
*/
