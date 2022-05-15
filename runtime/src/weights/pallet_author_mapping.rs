//! Autogenerated weights for pallet_author_mapping
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
// --pallet=pallet_author_mapping
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

/// Weight functions for pallet_author_mapping (automatically generated)
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_author_mapping::weights::WeightInfo for WeightInfo<T> {
    // Storage: AuthorMapping MappingWithDeposit (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    #[rustfmt::skip]
    fn add_association() -> Weight {
        (34_696_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: AuthorMapping MappingWithDeposit (r:2 w:2)
    #[rustfmt::skip]
    fn update_association() -> Weight {
        (26_877_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: AuthorMapping MappingWithDeposit (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    #[rustfmt::skip]
    fn clear_association() -> Weight {
        (36_253_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
}
