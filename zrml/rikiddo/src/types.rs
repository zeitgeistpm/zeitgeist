use std::u32;

use crate::{
    constants::*,
    traits::{MarketAverage, RikiddoFee},
};
use frame_support::dispatch::{fmt::Debug, Decode, Encode};
use sp_std::marker::PhantomData;
use substrate_fixed::{
    traits::{Fixed, FixedSigned, LossyFrom, LossyInto},
    transcendental::sqrt,
    types::{extra::U64, I9F23},
    FixedI128,
};

pub type UnixTimestamp = u64;

#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
pub struct TimestampedVolume<F: Fixed> {
    timestamp: UnixTimestamp,
    volume: F,
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
    pub fn into_seconds(&self) -> u32 {
        match *self {
            Timespan::Seconds(d) => d,
            Timespan::Minutes(d) => d * 60,
            Timespan::Hours(d) => d * 60 * 60,
            Timespan::Days(d) => u32::from(d) * 60 * 60 * 24,
            Timespan::Weeks(d) => u32::from(d) * 60 * 60 * 24 * 7,
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

impl<F> RikiddoFee<F> for FeeSigmoid
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
pub struct EmaVolumeConfig<F: Fixed> {
    pub ema_period: Timespan,
    pub smoothing: F,
}

impl<F: FixedSigned + LossyFrom<FixedI128<U64>>> EmaVolumeConfig<F> {
    pub fn new(ema_period: Timespan, smoothing: F) -> Self {
        Self { ema_period, smoothing }
    }
}

impl<F: FixedSigned + LossyFrom<FixedI128<U64>>> Default for EmaVolumeConfig<F> {
    fn default() -> Self {
        Self::new(EMA_SHORT, SMOOTHING.lossy_into())
    }
}

#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
enum MarketVolumeState {
    Uninitialized,
    DataCollectionStarted,
    DataCollected,
}

#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
pub struct EmaMarketVolume<F: Fixed> {
    pub config: EmaVolumeConfig<F>,
    pub ema: F,
    state: MarketVolumeState,
    start_time: UnixTimestamp,
    volumes_per_period: F,
    multiplier: F,
}

impl<F: FixedSigned> EmaMarketVolume<F> {
    pub fn new(config: EmaVolumeConfig<F>) -> Self {
        Self {
            config,
            ema: F::from_num(0),
            state: MarketVolumeState::Uninitialized,
            start_time: 0,
            volumes_per_period: F::from_num(0),
            multiplier: F::from_num(0),
        }
    }
}

impl<F: FixedSigned + From<u32> + LossyFrom<FixedI128<U64>>> Default for EmaMarketVolume<F> {
    fn default() -> Self {
        EmaMarketVolume::new(EmaVolumeConfig::default())
    }
}

impl<F: FixedSigned + From<u32>> MarketAverage<F> for EmaMarketVolume<F> {
    /// Calculate average (sma, ema, wma, depending on the concrete implementation) of market volume
    fn get(&self) -> Option<F> {
        match &self.state {
            MarketVolumeState::DataCollected => Some(self.ema),
            _ => None,
        }
    }

    /// Clear market data
    fn clear(&mut self) {
        self.ema = F::from_num(0);
        self.state = MarketVolumeState::Uninitialized;
        self.start_time = 0;
        self.volumes_per_period = F::from_num(0);
    }

    /// Update market volume
    fn update(&mut self, volume: TimestampedVolume<F>) -> Result<Option<F>, &'static str> {
        match self.state {
            MarketVolumeState::Uninitialized => {
                self.ema = volume.volume;
                self.start_time = volume.timestamp;
                self.volumes_per_period = 1.into();
                self.state = MarketVolumeState::DataCollectionStarted;
            }
            MarketVolumeState::DataCollectionStarted => {
                // During this phase the ema is still a sma.
                let ema_times_vpp =
                    if let Some(res) = self.ema.checked_mul(self.volumes_per_period) {
                        res
                    } else {
                        return Err("[EmaMarketVolume] Overflow during calculation: sma * \
                                    volumes_per_period");
                    };

                let ema_times_vpp_plus_volume =
                    if let Some(res) = ema_times_vpp.checked_add(volume.volume) {
                        res
                    } else {
                        return Err("[EmaMarketVolume] Overflow during calculation: sma * \
                                    volumes_per_period + volume");
                    };

                self.ema = if let Some(res) = ema_times_vpp_plus_volume
                    .checked_div(self.volumes_per_period.saturating_add(F::from(1)))
                {
                    res
                } else {
                    return Err("[EmaMarketVolume] Overflow during calculation: sma = numerator \
                                / denominator");
                };

                // In the context of blockchains, overflowing here is irrelevant (not realizable).
                // In other contexts, ensure that F can represent a number that is equal to the
                // incoming volumes during one period.
                self.volumes_per_period.saturating_add(F::from(1));

                let timestamp_sub_start_time =
                    if let Some(res) = volume.timestamp.checked_sub(self.start_time) {
                        res
                    } else {
                        return Err("[EmaMarketVolume] Incoming volume timestamp is older than \
                                    first recorded timestamp");
                    };

                if timestamp_sub_start_time >= self.config.ema_period.into_seconds() as u64 {
                    // Overflow is impossible here.
                    self.multiplier = self.config.smoothing
                        / (self.volumes_per_period.saturating_add(F::from(1u32)));
                    self.state = MarketVolumeState::DataCollected;
                }
            }
            MarketVolumeState::DataCollected => {
                let volume_times_multiplier =
                    if let Some(res) = volume.volume.checked_mul(self.multiplier) {
                        res
                    } else {
                        return Err(
                            "[EmaMarketVolume] Overflow during calculation: volume * multiplier"
                        );
                    };

                // Overflow is impossible here.
                let one_minus_multiplier = F::from(1) - self.multiplier;

                let ema_times_one_minus_multiplier =
                    if let Some(res) = self.ema.checked_mul(one_minus_multiplier) {
                        res
                    } else {
                        return Err(
                            "[EmaMarketVolume] Overflow during calculation: ema * (1 - multiplier)"
                        );
                    };

                self.ema = if let Some(res) =
                    volume_times_multiplier.checked_add(ema_times_one_minus_multiplier)
                {
                    res
                } else {
                    return Err("[EmaMarketVolume] Overflow during calculation: ema = a + b");
                };

                return Ok(Some(self.ema));
            }
        }

        Ok(None)
    }
}

#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
pub struct RikiddoSigmoidMV<FI: Fixed, FE: RikiddoFee<FI>, MA: MarketAverage<FI>> {
    pub fees: FE,
    pub ma_short: MA,
    pub ma_long: MA,
    pub _marker: PhantomData<FI>,
}
