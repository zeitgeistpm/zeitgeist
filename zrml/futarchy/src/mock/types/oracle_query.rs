use crate::traits::OracleQuery;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use alloc::fmt::Debug;
use frame_support::pallet_prelude::Weight;

#[derive(Clone, Debug, Decode, Encode, Eq, MaxEncodedLen, PartialEq, TypeInfo)]
pub struct MockOracleQuery {
    weight: Weight,
    value: bool,
}

impl OracleQuery for MockOracleQuery {
    fn evaluate(&self) -> (Weight, bool) {
        (self.weight, self.value)
    }
}
