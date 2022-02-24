//! This module contains utility functions for creating commonly used
//! fixed point type constants.

use substrate_fixed::traits::Fixed;

/// Create a fixed point number that represents 0 (zero).
pub fn fixed_zero<FixedType: Fixed>() -> Result<FixedType, &'static str> {
    if let Some(res) = FixedType::checked_from_num(0) {
        Ok(res)
    } else {
        Err("Unexpectedly failed to convert zero to fixed point type")
    }
}

/// Return the maximum value of FixedType as u128.
pub fn max_value_u128<FixedType: Fixed>() -> Result<u128, &'static str> {
    if let Some(res) = FixedType::max_value().int().checked_to_num() {
        Ok(res)
    } else {
        Err("Unexpectedly failed to convert max_value of fixed point type to u128")
    }
}
