use super::{Timespan, TimestampedVolume, UnixTimestamp};
use crate::{
    constants::{EMA_SHORT, SMOOTHING},
    traits::MarketAverage,
};
use frame_support::dispatch::{fmt::Debug, Decode, Encode};
use substrate_fixed::{
    traits::{Fixed, FixedUnsigned, LossyFrom, LossyInto, ToFixed},
    types::extra::{U24, U64},
    FixedU128, FixedU32,
};

#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
pub struct EmaConfig<FI: Fixed> {
    pub ema_period: Timespan,
    pub ema_period_estimate_after: Option<Timespan>,
    pub smoothing: FI,
}

impl<FU: FixedUnsigned + LossyFrom<FixedU32<U24>>> EmaConfig<FU> {
    pub fn new(
        ema_period: Timespan,
        mut ema_period_estimate_after: Option<Timespan>,
        smoothing: FU,
    ) -> Self {
        if let Some(res) = ema_period_estimate_after {
            ema_period_estimate_after = if res >= ema_period { None } else { Some(res) };
        };

        Self { ema_period, ema_period_estimate_after, smoothing }
    }
}

impl<FU: FixedUnsigned + LossyFrom<FixedU32<U24>>> Default for EmaConfig<FU> {
    fn default() -> Self {
        Self::new(EMA_SHORT, None, SMOOTHING.lossy_into())
    }
}

#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
pub enum MarketVolumeState {
    Uninitialized,
    DataCollectionStarted,
    DataCollected,
}

#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
pub struct EmaMarketVolume<FU: FixedUnsigned> {
    pub config: EmaConfig<FU>,
    pub ema: FU,
    multiplier: FU,
    last_time: UnixTimestamp,
    state: MarketVolumeState,
    start_time: UnixTimestamp,
    volumes_per_period: FU,
}

impl<FU: FixedUnsigned> EmaMarketVolume<FU> {
    pub fn new(config: EmaConfig<FU>) -> Self {
        Self {
            config,
            ema: FU::from_num(0),
            state: MarketVolumeState::Uninitialized,
            start_time: 0,
            last_time: 0,
            volumes_per_period: FU::from_num(0),
            multiplier: FU::from_num(0),
        }
    }

    fn calculate_ema(
        &mut self,
        volume: &TimestampedVolume<FU>,
    ) -> Result<Option<FU>, &'static str> {
        // Overflow is impossible here (the library ensures that multiplier ∊ [0,1])
        let volume_times_multiplier = if let Some(res) = volume.volume.checked_mul(self.multiplier)
        {
            res
        } else {
            return Err("[EmaMarketVolume] Overflow during calculation: volume * multiplier");
        };

        // Overflow is impossible here.
        let one_minus_multiplier = if let Some(res) = FU::from_num(1).checked_sub(self.multiplier) {
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

    fn calculate_sma(
        &mut self,
        volume: &TimestampedVolume<FU>,
    ) -> Result<Option<FU>, &'static str> {
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
            .checked_div(self.volumes_per_period.saturating_add(FU::from_num(1)))
        {
            res
        } else {
            return Err(
                "[EmaMarketVolume] Overflow during calculation: sma = numerator / denominator"
            );
        };

        Ok(Some(self.ema))
    }

    pub fn multiplier(&self) -> &FU {
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

    pub fn volumes_per_period(&self) -> &FU {
        &self.volumes_per_period
    }
}

impl<FU: FixedUnsigned + From<u32> + LossyFrom<FixedU32<U24>>> Default for EmaMarketVolume<FU> {
    fn default() -> Self {
        EmaMarketVolume::new(EmaConfig::default())
    }
}

impl<FU: FixedUnsigned + From<u32>> MarketAverage for EmaMarketVolume<FU> {
    type FU = FU;

    /// Calculate average (sma, ema, wma, depending on the concrete implementation) of market volume
    fn get(&self) -> Option<Self::FU> {
        match &self.state {
            MarketVolumeState::DataCollected => Some(self.ema),
            _ => None,
        }
    }

    /// Clear market data
    fn clear(&mut self) {
        self.ema = FU::from_num(0);
        self.multiplier = FU::from_num(0);
        self.last_time = 0;
        self.state = MarketVolumeState::Uninitialized;
        self.start_time = 0;
        self.volumes_per_period = FU::from_num(0);
    }

    /// Update market volume
    fn update(
        &mut self,
        volume: &TimestampedVolume<Self::FU>,
    ) -> Result<Option<Self::FU>, &'static str> {
        if let Some(res) = volume.timestamp.checked_sub(self.last_time) {
            res
        } else {
            return Err(
                "[EmaMarketVolume] Incoming volume timestamp is older than previous timestamp"
            );
        };

        let mut result: Option<FU> = None;

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

                let mut comparison_value = if let Some(res) = self.config.ema_period_estimate_after
                {
                    res.to_seconds()
                } else {
                    self.config.ema_period.to_seconds()
                };

                // It should not state transit, if the amount of gathered data is too low.
                // This would result in a multiplier that is greater than 1, which can lead to
                // a negative ema. The amount depends on the size of the smoothing factor.
                if timestamp_sub_start_time > comparison_value as u64
                    && (*self.volumes_per_period() + 1.into()) >= self.config.smoothing
                {
                    // We extrapolate the txs per period
                    if self.config.ema_period_estimate_after.is_some() {
                        // Ensure that we don't divide by 0
                        if comparison_value == 0 {
                            comparison_value = 1
                        }

                        let premature_time_fixed = <FixedU128<U64>>::from(comparison_value);
                        let mature_time_fixed =
                            <FixedU128<U64>>::from(self.config.ema_period.to_seconds());
                        // Overflow impossible
                        let estimate_ratio = mature_time_fixed.saturating_div(premature_time_fixed);

                        if estimate_ratio.int() > FU::max_value().to_num::<u64>() {
                            // Cannot occur as long as the From<U32> trait is required for FU and
                            // Timespan::to_seconds() returns u32.
                            return Err("[EmaMarketVolume] Estimate ratio does not fit in FU");
                        }

                        // Can't panic due to the previous check
                        let estimate_ratio_fu: FU = estimate_ratio.to_fixed();

                        self.volumes_per_period = if let Some(res) =
                            self.volumes_per_period.checked_mul(estimate_ratio_fu)
                        {
                            res.saturating_ceil()
                        } else {
                            return Err("[EmaMarketVolume] Overflow during estimation of \
                                        transactions per period");
                        }
                    }

                    // Overflow is impossible here.
                    self.multiplier = if let Some(res) = self
                        .config
                        .smoothing
                        .checked_div(self.volumes_per_period.saturating_add(FU::from(1)))
                    {
                        res
                    } else {
                        return Err("[EmaMarketVolume] Overflow during calculation: multiplier = \
                                    smoothing / (1 + volumes_per_period)");
                    };

                    self.state = MarketVolumeState::DataCollected;
                    result = self.calculate_ema(volume)?;
                } else {
                    // During this phase the ema is still a sma.
                    let _ = self.calculate_sma(volume)?;
                    // In the context of blockchains, overflowing here is irrelevant (technically
                    // not realizable). In other contexts, ensure that FU can represent a number
                    // that is equal to the number of incoming volumes during one period.
                    self.volumes_per_period = self.volumes_per_period.saturating_add(FU::from(1));
                }
            }
            MarketVolumeState::DataCollected => {
                result = self.calculate_ema(volume)?;
            }
        }

        self.last_time = volume.timestamp;
        Ok(result)
    }
}
