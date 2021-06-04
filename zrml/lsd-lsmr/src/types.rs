use crate::constants::*;
use crate::traits::Fee;
use frame_support::dispatch::{fmt::Debug, Decode, Encode};
#[cfg(feature = "std")]
use sp_runtime::traits::Hash;
use substrate_fixed::{
    types::extra::{U24, U32},
    FixedU32,
};

// TODO: adjust visibility and implement constructor
// TODO: docs

pub type Timestamp = u64;
pub type MaxBalance = u128;
pub type TimestampedVolume = (Timestamp, MaxBalance);

#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
pub enum Timespan {
    Minutes(u32),
    Hours(u32),
    Days(u16),
    Weeks(u16),
    Months(u16),
    Years(u8),
}

#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
pub struct AssetPair<A: Eq + Hash + PartialEq> {
    asset: A,
    base_asset: A,
}

// TODO: docs
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
        Self {
            initial_fee: INITIAL_FEE,
            minimal_revenue: MINIMAL_REVENUE,
            m: M,
            p: P,
            n: N,
        }
    }
}

#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
pub struct FeeSigmoid {
    pub config: FeeSigmoidConfig,
    pub ema_short: EmaVolume,
    pub ema_long: EmaVolume,
}

impl Default for FeeSigmoid {
    fn default() -> Self {
        return Self {
            config: FeeSigmoidConfig::default(),
            ema_short: EmaVolume::default(),
            ema_long: EmaVolume {
                ema_period: EMA_LONG.into(),
                smoothing: SMOOTHING,
                volumes_period: Vec::new(),
                volumes_2period_minus_period: Vec::new(),
                sma_period: 0,
                sma_2period_minus_period: 0,
                first_data_timestamp: 0,
                ema: 0,
            },
        };
    }
}

// TODO: docs
#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
pub struct EmaVolume {
    pub ema_period: Timespan,
    pub smoothing: FixedU32<U24>,
    pub volumes_period: Vec<TimestampedVolume>,
    pub volumes_2period_minus_period: Vec<TimestampedVolume>,
    pub sma_period: MaxBalance,
    pub sma_2period_minus_period: MaxBalance,
    pub first_data_timestamp: Timestamp,
    pub ema: MaxBalance,
}

impl Default for EmaVolume {
    fn default() -> Self {
        return Self {
            ema_period: EMA_SHORT,
            smoothing: SMOOTHING,
            volumes_period: Vec::new(),
            volumes_2period_minus_period: Vec::new(),
            sma_period: 0,
            sma_2period_minus_period: 0,
            first_data_timestamp: 0,
            ema: 0,
        };
    }
}

// TODO: docs
#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
pub struct LsdLsmr<A, F>
where
    A: Eq + Hash + PartialEq,
    F: Fee,
{
    pub assets: AssetPair<A>,
    pub fees: F,
}
