//! Autogenerated weights for zrml_styx
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-08-02, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/zeitgeist
// benchmark
// pallet
// --chain=dev
// --steps=50
// --repeat=20
// --pallet=zrml_styx
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --template=./misc/weight_template.hbs
// --output=./zrml/styx/src/weights.rs

#![allow(unused_parens)]
#![allow(unused_imports)]

use core::marker::PhantomData;
use frame_support::{traits::Get, weights::Weight};

///  Trait containing the required functions for weight retrival within
/// zrml_styx (automatically generated)
pub trait WeightInfoZeitgeist {
    fn cross() -> Weight;
    fn set_burn_amount() -> Weight;
}

/// Weight functions for zrml_styx (automatically generated)
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfoZeitgeist for WeightInfo<T> {
    // Storage: Styx Crossings (r:1 w:1)
    // Storage: Styx BurnAmount (r:1 w:0)
    fn cross() -> Weight {
        (25_000_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: Styx BurnAmount (r:0 w:1)
    fn set_burn_amount() -> Weight {
        (9_000_000 as Weight).saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
}
