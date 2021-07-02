use crate::types::Timespan;
use substrate_fixed::{types::extra::U64, FixedI128};

// --- Default configuration for EmaConfig struct ---
/// Default short EMA period (in seconds)
/// One hour.
pub const EMA_SHORT: Timespan = Timespan::Hours(1);

/// Default long EMA period (in seconds)
/// Six hours.
pub const EMA_LONG: Timespan = Timespan::Hours(6);

/// Default smoothing factor for EMA calculation
/// 2.0
pub const SMOOTHING: FixedI128<U64> =
    <FixedI128<U64>>::from_bits(0x0000_0000_0000_0002_0000_0000_0000_0000);

// --- Default configuration for FeeSigmoidConfig struct ---
/// Initial fee f
/// 0.005
pub const INITIAL_FEE: FixedI128<U64> =
    <FixedI128<U64>>::from_bits(0x0000_0000_0000_0000_0147_AE14_7AE1_47B0);

/// Minimal revenue w (proportion of initial fee f)
/// f * β = 0.005 * 0.7 = 0.0035
pub const MINIMAL_REVENUE: FixedI128<U64> =
    <FixedI128<U64>>::from_bits(0x0000_0000_0000_0000_00E5_6041_8937_4BC8);

/// m value
/// 0.01
pub const M: FixedI128<U64> =
    <FixedI128<U64>>::from_bits(0x0000_0000_0000_0000_028F_5C28_F5C2_8F60);

/// p value
/// 2.0
pub const P: FixedI128<U64> =
    <FixedI128<U64>>::from_bits(0x0000_0000_0000_0002_0000_0000_0000_0000);

/// n value
/// 0.0
pub const N: FixedI128<U64> =
    <FixedI128<U64>>::from_bits(0x0000_0000_0000_0000_0000_0000_0000_0000);
