use sp_std::ops::Sub;

use crate::{
    constants::*,
    traits::{LsdlmsrFee, MarketAverage},
};
use frame_support::dispatch::{fmt::Debug, Decode, Encode};
use sp_std::marker::PhantomData;
use substrate_fixed::{FixedU128, FixedU32, traits::FixedUnsigned, transcendental::sqrt, types::extra::{U24, U32, U64}};

pub type UnixTimestamp = u64;

#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
pub enum Timespan {
    Minutes(u32),
    Hours(u32),
    Days(u16),
    Weeks(u16),
}

impl Timespan {
    pub fn into_seconds(timespan: Timespan) -> u64 {
        match timespan {
            Timespan::Minutes(d) => u64::from(d) * 60,
            Timespan::Hours(d) => u64::from(d) * 60 * 60,
            Timespan::Days(d) => u64::from(d) * 60 * 60 * 24,
            Timespan::Weeks(d) => u64::from(d) * 60 * 60 * 24 * 7,
        }
    }
}

#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
pub struct FeeSigmoidConfig {
    pub initial_fee: FixedU32<U32>,
    pub minimal_revenue: FixedU32<U32>,
    pub m: FixedU32<U24>,
    pub p: FixedU32<U24>,
    pub n: FixedU32<U24>,
}

impl Default for FeeSigmoidConfig {
    fn default() -> Self {
        Self { initial_fee: INITIAL_FEE, minimal_revenue: MINIMAL_REVENUE, m: M, p: P, n: N }
    }
}

#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
pub struct FeeSigmoid {
    pub config: FeeSigmoidConfig,
}

// TODO
impl<FI> LsdlmsrFee<FI> for FeeSigmoid
where
    FI: FixedUnsigned
{
    fn calculate(&self, r: FI) -> FI {
        r
    }
}

#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
pub struct EmaVolumeConfig {
    pub ema_period: Timespan,
    pub multiplier: FixedU32<U24>,
}

impl EmaVolumeConfig {
    pub fn new(ema_period: Timespan, smoothing: FixedU32<U24>) -> Self {
        let duration: u32 = match ema_period {
            Timespan::Minutes(d) => d,
            Timespan::Hours(d) => d,
            Timespan::Days(d) => d.into(),
            Timespan::Weeks(d) => d.into(),
        };

        let one = FixedU32::<U24>::from_num(1);
        let fduration = FixedU32::<U24>::from_num(duration);

        Self { ema_period, multiplier: smoothing / (one + fduration) }
    }
}

impl Default for EmaVolumeConfig {
    fn default() -> Self {
        Self::new(EMA_SHORT, SMOOTHING)
    }
}

#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
pub struct EmaMarketVolume<FI: FixedUnsigned> {
    pub config: EmaVolumeConfig,
    sma_current_period: FI,
    sma_current_element_count: u64,
    sma_current_period_start: Option<UnixTimestamp>,
    pub ema: Option<FI>,
}

impl<FI: FixedUnsigned> EmaMarketVolume<FI> {
    pub fn new(config: EmaVolumeConfig) -> Self {
        Self {
            config,
            sma_current_period: FI::from_num(0),
            sma_current_element_count: 0,
            sma_current_period_start: None,
            ema: None,
        }
    }
}



#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
pub struct LsdLmsrSigmoidMV<FI: FixedUnsigned, FE: LsdlmsrFee<FI>, MA: MarketAverage<FI>> {
    pub fees: FE,
    pub ma_short: MA,
    pub ma_long: MA,
    pub _marker: PhantomData<FI>,
}
