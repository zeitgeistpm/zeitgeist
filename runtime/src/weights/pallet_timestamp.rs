//! Autogenerated weights for pallet_timestamp
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-03-23, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:// ./target/release/zeitgeist// benchmark// --chain=dev// --steps=50// --repeat=20// --pallet=pallet_timestamp// --extrinsic=*// --execution=wasm// --wasm-execution=compiled// --heap-pages=4096// --template=./misc/frame_weight_template.hbs// --output=./runtime/src/weights/

#![allow(unused_parens)]
#![allow(unused_imports)]

use core::marker::PhantomData;
use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, Weight},
};

/// Weight functions for pallet_timestamp (automatically generated)
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_timestamp::weights::WeightInfo for WeightInfo<T> {
    // Storage: Timestamp Now (r:1 w:1)
    // Storage: Aura CurrentSlot (r:1 w:0)
    fn set() -> Weight {
        (17_850_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }

    fn on_finalize() -> Weight {
        (6_390_000 as Weight)
    }
}
