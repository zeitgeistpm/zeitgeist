use alloc::fmt::Debug;
use frame_support::pallet_prelude::Weight;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use zeitgeist_primitives::traits::FutarchyOracle;

#[derive(Clone, Debug, Decode, Encode, Eq, MaxEncodedLen, PartialEq, TypeInfo)]
pub struct MockOracle {
    weight: Weight,
    value: bool,
}

impl MockOracle {
    pub fn new(weight: Weight, value: bool) -> Self {
        Self { weight, value }
    }
}

impl FutarchyOracle for MockOracle {
    fn evaluate(&self) -> (Weight, bool) {
        (self.weight, self.value)
    }
}
