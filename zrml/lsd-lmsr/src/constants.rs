use crate::types::Timespan;
use substrate_fixed::{types::extra::U24, types::extra::U32, FixedU32};

// --- Default configuration for EmaConfig struct ---
/// Default short EMA period (in seconds)
/// One hour.
pub const EMA_SHORT: Timespan = Timespan::Hours(1);

/// Default long EMA period (in seconds)
/// Six hours.
pub const EMA_LONG: Timespan = Timespan::Hours(6);

/// Default smoothing factor for EMA calculation
/// 2.0
pub const SMOOTHING: FixedU32<U24> = <FixedU32<U24>>::from_bits(0x0200_0000);

// --- Default configuration for FeeSigmoidConfig struct ---
/// Initial fee f
/// 0.03
pub const INITIAL_FEE: FixedU32<U32> = <FixedU32<U32>>::from_bits(0x07AE_1478);

/// Minimal revenue w (proportion of initial fee f)
/// f * β = 0.03 * 0.07 = 0.021
pub const MINIMAL_REVENUE: FixedU32<U32> = <FixedU32<U32>>::from_bits(0x0560_4188);

/// m value
/// 0.8
pub const M: FixedU32<U24> = <FixedU32<U24>>::from_bits(0xCCCC_CD00);

/// p value
/// 8.0
pub const P: FixedU32<U24> = <FixedU32<U24>>::from_bits(0x0800_0000);

/// n value
/// 1.0
pub const N: FixedU32<U24> = <FixedU32<U24>>::from_bits(0x0100_0000);
