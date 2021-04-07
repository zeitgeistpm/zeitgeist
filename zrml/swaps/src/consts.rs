use frame_support::dispatch::DispatchError;

pub const ARITHM_OF: DispatchError = DispatchError::Other("Arithmetic overflow");

/// The amount of precision to use in exponentiation.
pub const BPOW_PRECISION: u128 = 10;
/// The exit fee for removing liquidity from swaps.
pub const EXIT_FEE: u128 = 0;
