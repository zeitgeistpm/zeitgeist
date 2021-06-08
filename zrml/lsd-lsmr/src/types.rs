use crate::constants::*;
use crate::traits::Fee;
use alloc::collections::BTreeMap;
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
struct FeeSigmoid<A, S>
where
    A: Decode + Encode + EncodeLike + Eq + PartialEq,
    S: IterableStorageMap<A, Vec<TimestampedVolume>>,
{
    config: FeeSigmoidConfig,
    ema_short: EmaVolume<A, S>,
    ema_long: EmaVolume<A, S>,
}

#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
pub struct EmaVolumeConfig {
    pub ema_period: Timespan,
    pub smoothing: FixedU32<U24>,
}

impl Default for EmaVolumeConfig {
    fn default() -> Self {
        return Self {
            ema_period: EMA_SHORT,
            smoothing: SMOOTHING,
        };
    }
}

// TODO: docs
#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
struct EmaVolume<A, S>
where
    A: Decode + Encode + EncodeLike + Eq + PartialEq,
    S: IterableStorageMap<A, Vec<TimestampedVolume>>,
{
    config: EmaVolumeConfig,
    volumes_period: S,
    volumes_2period_minus_period: S,
    sma_period: BTreeMap<A, MaxBalance>,
    sma_2period_minus_period: BTreeMap<A, MaxBalance>,
    first_data_timestamp: Option<Timestamp>,
    ema: Option<MaxBalance>,
}

// TODO: docs
#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
struct LsdLmsr<FE: Fee> {
    fees: FE,
}
