use crate::traits::OracleQuery;
use alloc::fmt::Debug;
use frame_support::pallet_prelude::Weight;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

#[derive(Clone, Debug, Decode, Encode, Eq, MaxEncodedLen, PartialEq, TypeInfo)]
pub struct MockOracleQuery {
    weight: Weight,
    value: bool,
}

impl MockOracleQuery {
    pub fn new(weight: Weight, value: bool) -> Self {
        Self { weight, value }
    }
}

impl OracleQuery for MockOracleQuery {
    fn evaluate(&self) -> (Weight, bool) {
        (self.weight, self.value)
    }
}
