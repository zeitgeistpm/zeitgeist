use std::marker::PhantomData;

use crate::constants::*;
use crate::traits::Fee;
use frame_support::{
    dispatch::{fmt::Debug, Decode, Encode},
    storage::IterableStorageMap,
};
use parity_scale_codec::EncodeLike;
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

// TODO: docs
#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
pub struct FeeSigmoidConfig {
    initial_fee: FixedU32<U32>,
    minimal_revenue: FixedU32<U32>,
    m: FixedU32<U24>,
    p: FixedU32<U24>,
    n: FixedU32<U24>,
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
pub struct FeeSigmoid<A, S>
where
    A: Decode + Encode + EncodeLike + Eq + PartialEq,
    S: IterableStorageMap<A, Vec<TimestampedVolume>>,
{
    config: FeeSigmoidConfig,
    ema_short: EmaVolume<A, S>,
    ema_long: EmaVolume<A, S>,
}

/*
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
*/

#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
pub struct EmaVolumeConfig {
    ema_period: Timespan,
    smoothing: FixedU32<U24>,
}

// TODO: docs
#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
pub struct EmaVolume<A, S>
where
    A: Decode + Encode + EncodeLike + Eq + PartialEq,
    S: IterableStorageMap<A, Vec<TimestampedVolume>>,
{
    config: EmaVolumeConfig,
    volumes_period: S,
    volumes_2period_minus_period: S,
    sma_period: MaxBalance,
    sma_2period_minus_period: MaxBalance,
    first_data_timestamp: Timestamp,
    ema: MaxBalance,
    _marker: PhantomData<A>,
}

/*
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
*/

// TODO: docs
#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
pub struct LsdLmsr<FE: Fee> {
    fees: FE,
}
