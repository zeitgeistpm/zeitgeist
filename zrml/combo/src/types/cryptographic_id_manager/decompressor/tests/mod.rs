#![cfg(test)]

use super::*;

mod decompress_hash;
mod field_modulus;
mod get_collection_id;
mod matching_y_coordinate;
mod pow_magic_number;

#[derive(Debug)]
enum FromStrPrefixedError {
    /// Failed to convert bytes to scalar.
    FromBytesError,

    /// Failed to convert prefixed string to U256.
    ParseIntError(core::num::ParseIntError),
}

trait FromStrPrefixed
where
    Self: Sized,
{
    fn from_str_prefixed(x: &str) -> Result<Self, FromStrPrefixedError>;
}

impl FromStrPrefixed for Fq {
    fn from_str_prefixed(x: &str) -> Result<Fq, FromStrPrefixedError> {
        let x_u256 =
            U256::from_str_prefixed(x).map_err(|e| FromStrPrefixedError::ParseIntError(e))?;
        Fq::from_u256(x_u256).ok_or(FromStrPrefixedError::FromBytesError)
    }
}
