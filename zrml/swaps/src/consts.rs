use frame_support::dispatch::DispatchError;
use zeitgeist_primitives::constants::BASE;

pub const ARITHM_OF: DispatchError = DispatchError::Other("Arithmetic overflow");

/// The amount of precision to use in exponentiation.
pub const BPOW_PRECISION: u128 = 10;
/// The minimum value of the base parameter in bpow_approx.
pub const BPOW_APPROX_BASE_MIN: u128 = BASE / 4;
/// The maximum value of the base parameter in bpow_approx.
pub const BPOW_APPROX_BASE_MAX: u128 = 7 * BASE / 4;
/// The exit fee for removing liquidity from swaps.
pub const EXIT_FEE: u128 = 0;
