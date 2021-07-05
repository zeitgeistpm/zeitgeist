use super::{Timespan, TimestampedVolume, UnixTimestamp};
use crate::{
    constants::{EMA_SHORT, SMOOTHING},
    traits::MarketAverage,
};
use frame_support::dispatch::{fmt::Debug, Decode, Encode};
use substrate_fixed::{
    traits::{Fixed, FixedUnsigned, LossyFrom, LossyInto},
    types::extra::U24,
    FixedU32,
};

#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
pub struct EmaVolumeConfig<F: Fixed> {
    pub ema_period: Timespan,
    pub smoothing: F,
}

impl<F: FixedUnsigned + LossyFrom<FixedU32<U24>>> EmaVolumeConfig<F> {
    pub fn new(ema_period: Timespan, smoothing: F) -> Self {
        Self { ema_period, smoothing }
    }
}

impl<F: FixedUnsigned + LossyFrom<FixedU32<U24>>> Default for EmaVolumeConfig<F> {
    fn default() -> Self {
        Self::new(EMA_SHORT, SMOOTHING.lossy_into())
    }
}

#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
pub enum MarketVolumeState {
    Uninitialized,
    DataCollectionStarted,
    DataCollected,
}

#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
pub struct EmaMarketVolume<F: FixedUnsigned> {
    pub config: EmaVolumeConfig<F>,
    pub ema: F,
    multiplier: F,
    last_time: UnixTimestamp,
    state: MarketVolumeState,
    start_time: UnixTimestamp,
    volumes_per_period: F,
}

impl<F: FixedUnsigned> EmaMarketVolume<F> {
    pub fn new(config: EmaVolumeConfig<F>) -> Self {
        Self {
            config,
            ema: F::from_num(0),
            state: MarketVolumeState::Uninitialized,
            start_time: 0,
            last_time: 0,
            volumes_per_period: F::from_num(0),
            multiplier: F::from_num(0),
        }
    }

    fn calculate_ema(&mut self, volume: &TimestampedVolume<F>) -> Result<Option<F>, &'static str> {
        // Overflow is impossible here (the library ensures that multiplier ∊ [0,1])
        let volume_times_multiplier = if let Some(res) = volume.volume.checked_mul(self.multiplier)
        {
            res
        } else {
            return Err("[EmaMarketVolume] Overflow during calculation: volume * multiplier");
        };

        // Overflow is impossible here.
        let one_minus_multiplier = if let Some(res) = F::from_num(1).checked_sub(self.multiplier) {
            res
        } else {
            return Err("[EmaMarketVolume] Overflow during calculation: 1 - multiplier");
        };

        // Overflow is impossible here.
        let ema_times_one_minus_multiplier =
            if let Some(res) = self.ema.checked_mul(one_minus_multiplier) {
                res
            } else {
                return Err("[EmaMarketVolume] Overflow during calculation: ema * (1 - multiplier)");
            };

        // Overflow is impossible here.
        self.ema = if let Some(res) =
            volume_times_multiplier.checked_add(ema_times_one_minus_multiplier)
        {
            res
        } else {
            return Err("[EmaMarketVolume] Overflow during calculation: ema = smoothed_new + \
                        smoothed_previous");
        };

        Ok(Some(self.ema))
    }

    fn calculate_sma(&mut self, volume: &TimestampedVolume<F>) -> Result<Option<F>, &'static str> {
        // This can only overflow if the ema field is set manually
        let sma_times_vpp = if let Some(res) = self.ema.checked_mul(self.volumes_per_period) {
            res
        } else {
            return Err("[EmaMarketVolume] Overflow during calculation: sma * volumes_per_period");
        };

        let sma_times_vpp_plus_volume =
            if let Some(res) = sma_times_vpp.checked_add(volume.volume) {
                res
            } else {
                return Err("[EmaMarketVolume] Overflow during calculation: sma * \
                            volumes_per_period + volume");
            };

        // This can't overflow.
        self.ema = if let Some(res) = sma_times_vpp_plus_volume
            .checked_div(self.volumes_per_period.saturating_add(F::from_num(1)))
        {
            res
        } else {
            return Err(
                "[EmaMarketVolume] Overflow during calculation: sma = numerator / denominator"
            );
        };

        Ok(Some(self.ema))
    }

    pub fn multiplier(&self) -> &F {
        &self.multiplier
    }

    pub fn last_time(&self) -> &UnixTimestamp {
        &self.last_time
    }

    // Following functions are required mainly for testing.
    pub fn state(&self) -> &MarketVolumeState {
        &self.state
    }

    pub fn start_time(&self) -> &UnixTimestamp {
        &self.start_time
    }

    pub fn volumes_per_period(&self) -> &F {
        &self.volumes_per_period
    }
}

impl<F: FixedUnsigned + From<u32> + LossyFrom<FixedU32<U24>>> Default for EmaMarketVolume<F> {
    fn default() -> Self {
        EmaMarketVolume::new(EmaVolumeConfig::default())
    }
}

impl<F: FixedUnsigned + From<u32>> MarketAverage<F> for EmaMarketVolume<F> {
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
        self.multiplier = F::from_num(0);
        self.last_time = 0;
        self.state = MarketVolumeState::Uninitialized;
        self.start_time = 0;
        self.volumes_per_period = F::from_num(0);
    }

    /// Update market volume
    fn update(&mut self, volume: TimestampedVolume<F>) -> Result<Option<F>, &'static str> {
        if let Some(res) = volume.timestamp.checked_sub(self.last_time) {
            res
        } else {
            return Err(
                "[EmaMarketVolume] Incoming volume timestamp is older than previous timestamp"
            );
        };

        let mut result: Option<F> = None;

        match self.state {
            MarketVolumeState::Uninitialized => {
                self.ema = volume.volume;
                self.start_time = volume.timestamp;
                self.last_time = volume.timestamp;
                self.volumes_per_period = 1.into();
                self.state = MarketVolumeState::DataCollectionStarted;
            }
            MarketVolumeState::DataCollectionStarted => {
                // This can never overflow, because every transactions timestamp is checked
                // against the timestamp of the previous transaction.
                let timestamp_sub_start_time = volume.timestamp.saturating_sub(self.start_time);

                // It should not state transit, if the amount of gathered data is too low.
                // This would result in a multiplier that is greater than 1, which can lead to
                // a negative ema. The amount depends on the size of the smoothing factor.
                if timestamp_sub_start_time > self.config.ema_period.to_seconds() as u64
                    && (*self.volumes_per_period() + 1.into()) >= self.config.smoothing
                {
                    // Overflow is impossible here.
                    self.multiplier = if let Some(res) = self
                        .config
                        .smoothing
                        .checked_div(self.volumes_per_period.saturating_add(F::from(1)))
                    {
                        res
                    } else {
                        return Err("[EmaMarketVolume] Overflow during calculation: multiplier = \
                                    smoothing / (1 + volumes_per_period)");
                    };
                    self.state = MarketVolumeState::DataCollected;
                    result = self.calculate_ema(&volume)?;
                } else {
                    // During this phase the ema is still a sma.
                    result = self.calculate_sma(&volume)?;
                    // In the context of blockchains, overflowing here is irrelevant (technically
                    // not realizable). In other contexts, ensure that F can represent a number
                    // that is equal to the number of incoming volumes during one period.
                    self.volumes_per_period = self.volumes_per_period.saturating_add(F::from(1));
                }
            }
            MarketVolumeState::DataCollected => {
                result = self.calculate_ema(&volume)?;
            }
        }

        self.last_time = volume.timestamp;
        Ok(result)
    }
}
