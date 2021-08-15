#![no_main]
//! Fuzz test: Conversion Balance -> FixedU

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use zrml_rikiddo::{
    traits::Sigmoid,
    types::{FeeSigmoid, FeeSigmoidConfig},
};

mod shared;
use shared::fixed_from_u128;

fuzz_target!(|data: Data| {
});

#[derive(Debug, Arbitrary)]
struct Data {
}
