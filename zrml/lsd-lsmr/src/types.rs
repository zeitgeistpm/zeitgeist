use crate::constants::*;
use frame_support::{Parameter, dispatch::{Decode, Encode, fmt::Debug}};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::traits::{AtLeast32Bit, Hash};
use substrate_fixed::{FixedU32, types::extra::{U24, U32}};

// TODO: derive essential trait implementations
// TODO: implement default trait
// TODO: docs

#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
// #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct AssetPair<A: Eq + Hash + PartialEq> {
    pub asset1: A,
    pub asset2: A,
}

// TODO: docs
#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
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
            n: N
        }
	}
}

// TODO: docs
#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct EmaConfig<M: AtLeast32Bit + Parameter + Default + Copy> {
    pub ema_long: M,
    pub ema_short: M,
    pub smoothing: FixedU32<U24>,
}

macro_rules! impl_emaconfig_default {
    (for $($t:ty),+) => {
        $(impl Default for EmaConfig<$t> {
            fn default() -> Self {
                return Self{
                    ema_short: EMA_SHORT.into(),
                    ema_long: EMA_LONG.into(),
                    smoothing: SMOOTHING
                }
            }
        })*
    }
}

impl_emaconfig_default!(for u32, u64, u128);

// TODO: docs
#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
// #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct LsdLsmrConfig<A, M>
    where
        A: Eq + Hash + PartialEq,
        M: AtLeast32Bit + Parameter + Default + Copy
{
    pub assets: AssetPair<A>,
    pub fees: FeeSigmoidConfig, // TODO: replace by trait for FeeConfig (calculate())
    pub indicators: EmaConfig<M>,
}