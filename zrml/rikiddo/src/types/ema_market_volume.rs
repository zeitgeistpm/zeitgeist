//! This module contains the structures used to calculate the exponential moving average.
use super::{Timespan, TimestampedVolume, UnixTimestamp};
use crate::{
    constants::{EMA_SHORT, SMOOTHING},
    traits::MarketAverage,
};
#[cfg(feature = "arbitrary")]
use arbitrary::{Arbitrary, Result as ArbitraryResult, Unstructured};
#[cfg(feature = "arbitrary")]
use core::mem;
use frame_support::dispatch::{Decode, Encode};
use parity_scale_codec::MaxEncodedLen;
use scale_info::TypeInfo;
use sp_core::RuntimeDebug;
use substrate_fixed::{
    traits::{Fixed, FixedUnsigned, LossyFrom, LossyInto, ToFixed},
    types::extra::{U24, U64},
    FixedU128, FixedU32,
};
#[cfg(feature = "arbitrary")]
use substrate_fixed::{
    types::extra::{LeEqU128, LeEqU16, LeEqU32, LeEqU64, LeEqU8},
    FixedI128, FixedI16, FixedI32, FixedI64, FixedI8, FixedU16, FixedU64, FixedU8,
};

/// Configuration values used during the calculation of the exponenial moving average.
#[derive(Clone, Decode, Encode, Eq, MaxEncodedLen, PartialEq, RuntimeDebug, TypeInfo)]
pub struct EmaConfig<FI: Fixed> {
    /// The duration of one ema period.
    pub ema_period: Timespan,
    /// It is possible to estimate the values required to calculate the ema before `ema_period`
    /// has passed. The count of added volumes after `ema_period` is estimated from the current
    /// count of volumes.
    pub ema_period_estimate_after: Option<Timespan>,
    /// The smoothing factor that's used to calculate the multiplier
    /// `k = smoothing / (1 + tx_count_per_period)`.
    pub smoothing: FI,
}

#[cfg(feature = "arbitrary")]
macro_rules! impl_arbitrary_for_ema_config {
    ( $t:ident, $LeEqU:ident, $p:ty ) => {
        impl<'a, Frac> Arbitrary<'a> for EmaConfig<$t<Frac>>
        where
            Frac: $LeEqU,
            $t<Frac>: Fixed,
        {
            fn arbitrary(u: &mut Unstructured<'a>) -> ArbitraryResult<Self> {
                Ok(EmaConfig::<$t<Frac>> {
                    ema_period: <Timespan as Arbitrary<'a>>::arbitrary(u)?,
                    ema_period_estimate_after: Some(<Timespan as Arbitrary<'a>>::arbitrary(u)?),
                    smoothing: <$t<Frac>>::from_bits(<$p as Arbitrary<'a>>::arbitrary(u)?),
                })
            }

            #[inline]
            fn size_hint(depth: usize) -> (usize, Option<usize>) {
                let (min, max) = <Timespan as Arbitrary<'a>>::size_hint(depth);
                let fsc_size = <$p as Arbitrary<'a>>::size_hint(depth);
                let max_accumulated = max
                    .unwrap_or(min)
                    .saturating_mul(2)
                    .saturating_add(fsc_size.1.unwrap_or(fsc_size.0));
                let min_accumulated = min.saturating_mul(2).saturating_add(fsc_size.0);

                if max_accumulated == usize::MAX {
                    (min_accumulated, None)
                } else {
                    (min_accumulated, Some(max_accumulated))
                }
            }
        }
    };
}

cfg_if::cfg_if! {
    if #[cfg(feature = "arbitrary")] {
        impl_arbitrary_for_ema_config! {FixedI8, LeEqU8, i8}
        impl_arbitrary_for_ema_config! {FixedI16, LeEqU16, i16}
        impl_arbitrary_for_ema_config! {FixedI32, LeEqU32, i32}
        impl_arbitrary_for_ema_config! {FixedI64, LeEqU64, i64}
        impl_arbitrary_for_ema_config! {FixedI128, LeEqU128, i128}
        impl_arbitrary_for_ema_config! {FixedU8, LeEqU8, u8}
        impl_arbitrary_for_ema_config! {FixedU16, LeEqU16, u16}
        impl_arbitrary_for_ema_config! {FixedU32, LeEqU32, u32}
        impl_arbitrary_for_ema_config! {FixedU64, LeEqU64, u64}
        impl_arbitrary_for_ema_config! {FixedU128, LeEqU128, u128}
    }
}

impl<FU: FixedUnsigned + LossyFrom<FixedU32<U24>>> EmaConfig<FU> {
    /// Create a new `EmaConfig` instance based on a [`EmaConfig`](struct@EmaConfig)
    /// configuration. Use `default()` if uncertain which values to use.
    ///
    /// # Arguments
    ///
    /// * See [`EmaConfig`](struct@EmaConfig).
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

/// The current state of an instance of the [`EmaMarketVolume`](struct@EmaMarketVolume) struct.
#[derive(Clone, Decode, Encode, Eq, MaxEncodedLen, PartialEq, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub enum MarketVolumeState {
    /// The first state before any data was received.
    Uninitialized,
    /// The second state. Used as long not enough data was gathered.
    DataCollectionStarted,
    /// The final state, during which the evaluation of the data can happen.
    DataCollected,
}

/// The EmaMarketVolume `struct` offers functionality to collect and evaluate market volume data
/// into an exponential moving average value.
#[derive(Clone, Decode, Encode, Eq, MaxEncodedLen, PartialEq, RuntimeDebug, TypeInfo)]
pub struct EmaMarketVolume<FU: FixedUnsigned> {
    /// See [`EmaConfig`](struct@EmaConfig).
    pub config: EmaConfig<FU>,
    /// The current ema value or any intermediate value.
    pub ema: FU,
    multiplier: FU,
    last_time: UnixTimestamp,
    state: MarketVolumeState,
    start_time: UnixTimestamp,
    volumes_per_period: FU,
}

#[cfg(feature = "arbitrary")]
macro_rules! impl_arbitrary_for_ema_market_volume {
    ( $t:ident, $LeEqU:ident, $p:ty ) => {
        impl<'a, Frac> Arbitrary<'a> for EmaMarketVolume<$t<Frac>>
        where
            Frac: $LeEqU,
            $t<Frac>: FixedUnsigned + From<u8>,
        {
            fn arbitrary(u: &mut Unstructured<'a>) -> ArbitraryResult<Self> {
                Ok(EmaMarketVolume::<$t<Frac>>::new(
                    <EmaConfig<$t<Frac>> as Arbitrary<'a>>::arbitrary(u)?,
                ))
            }

            #[inline]
            fn size_hint(depth: usize) -> (usize, Option<usize>) {
                let (min, max) = <EmaConfig<$t<Frac>> as Arbitrary<'a>>::size_hint(depth);
                let fixed_size = mem::size_of::<$t<Frac>>();
                let ema_size = (fixed_size, fixed_size);
                let multiplier_size = (fixed_size, fixed_size);
                let last_time_size = <UnixTimestamp as Arbitrary<'a>>::size_hint(depth);
                let state_size = <MarketVolumeState as Arbitrary<'a>>::size_hint(depth);
                let start_time_size = <UnixTimestamp as Arbitrary<'a>>::size_hint(depth);
                let volumes_per_period = (fixed_size, fixed_size);

                let max_accumulated = max
                    .unwrap_or(0)
                    .saturating_mul(2)
                    .saturating_add(ema_size.1)
                    .saturating_add(multiplier_size.1)
                    .saturating_add(last_time_size.1.unwrap_or(last_time_size.0))
                    .saturating_add(state_size.1.unwrap_or(state_size.0))
                    .saturating_add(start_time_size.1.unwrap_or(start_time_size.0))
                    .saturating_add(volumes_per_period.0);
                let min_accumulated = min
                    .saturating_add(ema_size.0)
                    .saturating_add(multiplier_size.0)
                    .saturating_add(last_time_size.0)
                    .saturating_add(state_size.0)
                    .saturating_add(start_time_size.0)
                    .saturating_add(volumes_per_period.0);

                if max_accumulated == usize::MAX {
                    (min_accumulated, None)
                } else {
                    (min_accumulated, Some(max_accumulated))
                }
            }
        }
    };
}

cfg_if::cfg_if! {
    if #[cfg(feature = "arbitrary")] {
        impl_arbitrary_for_ema_market_volume! {FixedI8, LeEqU8, i8}
        impl_arbitrary_for_ema_market_volume! {FixedI16, LeEqU16, i16}
        impl_arbitrary_for_ema_market_volume! {FixedI32, LeEqU32, i32}
        impl_arbitrary_for_ema_market_volume! {FixedI64, LeEqU64, i64}
        impl_arbitrary_for_ema_market_volume! {FixedI128, LeEqU128, i128}
        impl_arbitrary_for_ema_market_volume! {FixedU8, LeEqU8, u8}
        impl_arbitrary_for_ema_market_volume! {FixedU16, LeEqU16, u16}
        impl_arbitrary_for_ema_market_volume! {FixedU32, LeEqU32, u32}
        impl_arbitrary_for_ema_market_volume! {FixedU64, LeEqU64, u64}
        impl_arbitrary_for_ema_market_volume! {FixedU128, LeEqU128, u128}
    }
}

impl<FU: FixedUnsigned + From<u8>> EmaMarketVolume<FU> {
    /// Initialize the structure based on a configuration.
    ///
    /// # Arguments
    ///
    /// * See [`EmaConfig`](struct@EmaConfig)
    pub fn new(config: EmaConfig<FU>) -> Self {
        Self {
            config,
            ema: 0u8.into(),
            state: MarketVolumeState::Uninitialized,
            start_time: 0,
            last_time: 0,
            volumes_per_period: 0u8.into(),
            multiplier: 0u8.into(),
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
        let one_minus_multiplier = if let Some(res) = FU::from(1u8).checked_sub(self.multiplier) {
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
            .checked_div(self.volumes_per_period.saturating_add(FU::from(1u8)))
        {
            res
        } else {
            return Err(
                "[EmaMarketVolume] Overflow during calculation: sma = numerator / denominator"
            );
        };

        Ok(Some(self.ema))
    }

    /// When the final state is reached, this function can be used to retrieve the multiplier.
    pub fn multiplier(&self) -> &FU {
        &self.multiplier
    }

    /// Returns the `UnixTimestamp` of the last volume that was added.
    pub fn last_time(&self) -> &UnixTimestamp {
        &self.last_time
    }

    /// Returns the current state of the data collection and evaluation automaton.
    pub fn state(&self) -> &MarketVolumeState {
        &self.state
    }

    /// Returns the `UnixTimestamp` of the first volume that was added.
    pub fn start_time(&self) -> &UnixTimestamp {
        &self.start_time
    }

    /// Returns how many separate volumes one ema period contains.
    pub fn volumes_per_period(&self) -> &FU {
        &self.volumes_per_period
    }
}

impl<FU: FixedUnsigned + From<u8> + LossyFrom<FixedU32<U24>>> Default for EmaMarketVolume<FU> {
    fn default() -> Self {
        EmaMarketVolume::new(EmaConfig::default())
    }
}

impl<FU: FixedUnsigned + From<u8>> MarketAverage for EmaMarketVolume<FU> {
    type FU = FU;

    /// Get the ema value if available, otherwise `None`. Using this function is prefered in most
    /// cases, because directly accessing the `ema` field will also return a valid result during
    /// the time when not enough data was gathered, but the `ema` value is still a
    /// `sma` value at this time.
    fn get(&self) -> Option<Self::FU> {
        match &self.state {
            MarketVolumeState::DataCollected => Some(self.ema),
            _ => None,
        }
    }

    /// Clear market data.
    fn clear(&mut self) {
        self.ema = 0u8.into();
        self.multiplier = 0u8.into();
        self.last_time = 0;
        self.state = MarketVolumeState::Uninitialized;
        self.start_time = 0;
        self.volumes_per_period = 0u8.into();
    }

    /// Update market data. This implementation will count the
    /// [`TimestampedVolume`](struct@TimestampedVolume)s for one period and then use the amount
    /// to derive the multiplier based on the smoothing factor. It will continue counting volumes,
    /// if the resulting multiplier is still greater than one (very rare case, could lead to
    /// negative ema). Once enough data is gathered, the ema is updated after every new incoming
    /// `volume`.
    ///
    /// # Arguments
    ///
    /// * `volume`: The timestamped volume that should be added to the market data.
    fn update_volume(
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
                if timestamp_sub_start_time > u64::from(comparison_value)
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

                        // Cannot fail as long as the From<U32> trait is required for FU and
                        // Timespan::to_seconds() returns u32.
                        let estimate_ratio_fu: FU = estimate_ratio
                            .checked_to_fixed()
                            .ok_or("[EmaMarketVolume] Estimate ratio does not fit in FU")?;

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
