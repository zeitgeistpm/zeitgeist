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
