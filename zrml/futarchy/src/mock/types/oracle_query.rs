use crate::traits::OracleQuery;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

#[derive(Decode, Encode, MaxEncodedLen, TypeInfo)]
pub struct MockOracleQuery {
    value: bool,
}

impl MockOracleQuery {
    fn new(value: bool) -> Self {
        Self { value }
    }
}

impl OracleQuery for MockOracleQuery {
    fn evaluate(&self) -> bool {
        self.value
    }
}
