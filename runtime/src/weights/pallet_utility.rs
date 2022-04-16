//! Autogenerated weights for pallet_utility
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-04-15, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/zeitgeist
// benchmark
// --chain=dev
// --steps=50
// --repeat=20
// --pallet=pallet_utility
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --template=./misc/frame_weight_template.hbs
// --output=./runtime/src/weights/

#![allow(unused_parens)]
#![allow(unused_imports)]

use core::marker::PhantomData;
use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, Weight},
};

/// Weight functions for pallet_utility (automatically generated)
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_utility::weights::WeightInfo for WeightInfo<T> {
    fn batch(c: u32) -> Weight {
        (15_257_000 as Weight)
            // Standard Error: 17_000
            .saturating_add((6_711_000 as Weight).saturating_mul(c as Weight))
    }
    fn as_derivative() -> Weight {
        (4_690_000 as Weight)
    }
    fn batch_all(c: u32) -> Weight {
        (52_000_000 as Weight)
            // Standard Error: 25_000
            .saturating_add((7_176_000 as Weight).saturating_mul(c as Weight))
    }
    fn dispatch_as() -> Weight {
        (27_680_000 as Weight)
    }
}
