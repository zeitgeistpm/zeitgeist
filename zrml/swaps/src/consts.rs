use frame_support::dispatch::DispatchError;

pub const ARITHM_OF: DispatchError = DispatchError::Other("Arithmetic overflow");
/// The base number of decimals places to use for math.
pub const BASE: u128 = 10_000_000_000;
pub const BPOW_PRECISION: u128 = 10;
pub const EXIT_FEE: u128 = 0;
