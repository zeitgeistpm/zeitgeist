//! This module contains constants that are used during the processing of market data and
//! the calculation of fees.

use crate::types::Timespan;
use substrate_fixed::{
    types::extra::{U24, U32},
    FixedI32, FixedU32,
};

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
/// m value
/// 0.01
pub const M: FixedI32<U24> = <FixedI32<U24>>::from_bits(0x0002_8F5C);

/// p value
/// 2.0
pub const P: FixedI32<U24> = <FixedI32<U24>>::from_bits(0x0200_0000);

/// n value
/// 0.0
pub const N: FixedI32<U24> = <FixedI32<U24>>::from_bits(0x0000_0000);

// --- Default configuration for RikiddoConfig struct ---
/// Initial fee f
/// 0.005
pub const INITIAL_FEE: FixedU32<U32> = <FixedU32<U32>>::from_bits(0x0147_AE14);

/// Minimal revenue w (proportion of initial fee f)
/// f * β = 0.005 * 0.7 = 0.0035
pub const MINIMAL_REVENUE: FixedU32<U32> = <FixedU32<U32>>::from_bits(0x00E5_6042);
