use crate::{
    constants::*,
    traits::{LsdlmsrFee, MarketAverage},
};
use frame_support::dispatch::{fmt::Debug, Decode, Encode};
use sp_std::marker::PhantomData;
use substrate_fixed::{FixedI128, FixedU128, FixedU32, traits::{FixedSigned, FixedUnsigned}, transcendental::sqrt, types::extra::{U24, U32, U64}};

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
    pub m: FixedI128<U64>,
    pub p: FixedI128<U64>,
    pub n: FixedI128<U64>,
}

impl Default for FeeSigmoidConfig {
    fn default() -> Self {
        Self { m: M, p: P, n: N }
    }
}

#[derive(Clone, Debug, Decode, Default, Encode, Eq, PartialEq)]
pub struct FeeSigmoid {
    pub config: FeeSigmoidConfig,
}

impl<FI: FixedSigned + Into<FixedI128<U64>>> LsdlmsrFee<FI> for FeeSigmoid {
    type Output = FixedI128<U64>;

    // z(r) in https://files.kyber.network/DMM-Feb21.pdf
    fn calculate(&self, r: FI) -> Result<Self::Output, &'static str>{
        let r_minus_n = if let Some(res) = r.into().checked_sub(self.config.n)  {
            res
        } else {
            return Err("[FeeSigmoid] Overflow during calculation: r - n");
        };

        let numerator = if let Some(res) = r_minus_n.checked_mul(self.config.m)  {
            res
        } else {
            return Err("[FeeSigmoid] Overflow during calculation: m * (r-n)");
        };

        let r_minus_n_squared =  if let Some(res) = r_minus_n.checked_mul(r_minus_n)  {
            res
        } else {
            return Err("[FeeSigmoid] Overflow during calculation: (r-n)^2");
        };

        let p_plus_r_minus_n = if let Some(res) = self.config.p.checked_add(r_minus_n_squared)  {
            res
        } else {
            return Err("[FeeSigmoid] Overflow during calculation: p + (r-n)^2");
        };

        let denominator = sqrt::<FixedI128<U64>,FixedI128<U64>>(p_plus_r_minus_n)?;

        let _ = if let Some(res) = numerator.checked_div(denominator)  {
            return Ok(res);
        } else {
            return Err("[FeeSigmoid] Overflow during calculation: numerator / denumerator");
        };
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
