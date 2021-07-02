use crate::{
    constants::*,
    traits::{LsdlmsrFee, MarketAverage},
};
use frame_support::dispatch::{fmt::Debug, Decode, Encode};
use sp_std::{collections::vec_deque::VecDeque, marker::PhantomData};
use substrate_fixed::{
    traits::{Fixed, FixedSigned, FixedUnsigned, LossyFrom, LossyInto},
    transcendental::sqrt,
    types::{extra::U64, I9F23},
    FixedI128,
};

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

impl<F> LsdlmsrFee<F> for FeeSigmoid
where
    F: FixedSigned + LossyFrom<FixedI128<U64>> + PartialOrd<I9F23>,
{
    // z(r) in https://files.kyber.network/DMM-Feb21.pdf
    fn calculate(&self, r: F) -> Result<F, &'static str> {
        let r_minus_n = if let Some(res) = r.checked_sub(self.config.n.lossy_into()) {
            res
        } else {
            return Err("[FeeSigmoid] Overflow during calculation: r - n");
        };

        let numerator = if let Some(res) = r_minus_n.checked_mul(self.config.m.lossy_into()) {
            res
        } else {
            return Err("[FeeSigmoid] Overflow during calculation: m * (r-n)");
        };

        let r_minus_n_squared = if let Some(res) = r_minus_n.checked_mul(r_minus_n) {
            res
        } else {
            return Err("[FeeSigmoid] Overflow during calculation: (r-n)^2");
        };

        let p_plus_r_minus_n_squared =
            if let Some(res) = F::lossy_from(self.config.p).checked_add(r_minus_n_squared) {
                res
            } else {
                return Err("[FeeSigmoid] Overflow during calculation: p + (r-n)^2");
            };

        let denominator = sqrt::<F, F>(p_plus_r_minus_n_squared)?;

        let _ = if let Some(res) = numerator.checked_div(denominator) {
            return Ok(res);
        } else {
            return Err("[FeeSigmoid] Overflow during calculation: numerator / denominator");
        };
    }
}

#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
pub struct EmaVolumeConfig {
    pub ema_period: Timespan,
    pub multiplier: FixedI128<U64>,
}

impl EmaVolumeConfig {
    pub fn new(ema_period: Timespan, smoothing: FixedI128<U64>) -> Self {
        let duration: u32 = match ema_period {
            Timespan::Minutes(d) => d,
            Timespan::Hours(d) => d,
            Timespan::Days(d) => d.into(),
            Timespan::Weeks(d) => d.into(),
        };

        let one = FixedI128::<U64>::from_num(1);
        let fduration = FixedI128::<U64>::from_num(duration);

        Self { ema_period, multiplier: smoothing / (one + fduration) }
    }
}

impl Default for EmaVolumeConfig {
    fn default() -> Self {
        Self::new(EMA_SHORT, SMOOTHING)
    }
}

#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
enum MarketVolumeState {
    Initialized,
    SmaCollectionStarted,
    SmaCollected,
    EmaCollected,
}

#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
pub struct EmaMarketVolume<F: Fixed> {
    pub config: EmaVolumeConfig,
    pub sma: F,
    pub ema_chains: Vec<F>,
    state: MarketVolumeState,
    start_time: Option<UnixTimestamp>,
    volumes: VecDeque<F>,
    volumes_per_period: u64,
    total_volumes: u64,
}

impl<F: Fixed> EmaMarketVolume<F> {
    pub fn new(config: EmaVolumeConfig) -> Self {
        Self {
            config,
            sma: F::from_num(0),
            ema_chains: Vec::new(),
            state: MarketVolumeState::Initialized,
            start_time: None,
            volumes: VecDeque::new(),
            volumes_per_period: 0,
            total_volumes: 0,
        }
    }
}

impl<F: Fixed> Default for EmaMarketVolume<F> {
    fn default() -> Self {
        EmaMarketVolume::new(EmaVolumeConfig::default())
    }
}

impl<F: FixedSigned> MarketAverage<F> for EmaMarketVolume<F> {
    /// Update market volume
    fn update(&mut self, volume: F) -> Option<F> {
        // TODO
        Some(volume)
    }

    /// Clear market data
    fn clear(&mut self) {
        self.sma = F::from_num(0);
        self.ema_chains = Vec::new();
        self.state = MarketVolumeState::Initialized;
        self.start_time = None;
        self.volumes = VecDeque::new();
        self.volumes_per_period = 0;
        self.total_volumes = 0;
    }

    /// Calculate average (sma, ema, wma, depending on the concrete implementation) of market volume
    fn calculate(&self) -> Option<F> {
        match &self.state {
            MarketVolumeState::SmaCollected => Some(self.sma),
            MarketVolumeState::EmaCollected => {
                let idx = ((self.total_volumes - 1) % self.volumes_per_period) as usize;

                if self.ema_chains.len() > idx {
                    Some(self.ema_chains[idx])
                } else {
                    // This should not happen
                    None
                }
            }
            _ => None,
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
