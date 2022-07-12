//! Autogenerated weights for pallet_scheduler
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-07-08, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/zeitgeist
// benchmark
// pallet
// --chain=dev
// --steps=50
// --repeat=20
// --pallet=pallet_scheduler
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

/// Weight functions for pallet_scheduler (automatically generated)
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_scheduler::weights::WeightInfo for WeightInfo<T> {
    // Storage: Scheduler Agenda (r:2 w:2)
    // Storage: Preimage PreimageFor (r:1 w:1)
    // Storage: Preimage StatusFor (r:1 w:1)
    // Storage: Scheduler Lookup (r:0 w:1)
    fn on_initialize_periodic_named_resolved(s: u32) -> Weight {
        (44_416_000 as Weight)
            // Standard Error: 474_000
            .saturating_add((48_242_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().reads((3 as Weight).saturating_mul(s as Weight)))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
            .saturating_add(T::DbWeight::get().writes((4 as Weight).saturating_mul(s as Weight)))
    }
    // Storage: Scheduler Agenda (r:1 w:1)
    // Storage: Preimage PreimageFor (r:1 w:1)
    // Storage: Preimage StatusFor (r:1 w:1)
    // Storage: Scheduler Lookup (r:0 w:1)
    fn on_initialize_named_resolved(s: u32) -> Weight {
        (50_197_000 as Weight)
            // Standard Error: 284_000
            .saturating_add((36_629_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().reads((2 as Weight).saturating_mul(s as Weight)))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
            .saturating_add(T::DbWeight::get().writes((3 as Weight).saturating_mul(s as Weight)))
    }
    // Storage: Scheduler Agenda (r:2 w:2)
    // Storage: Preimage PreimageFor (r:1 w:1)
    // Storage: Preimage StatusFor (r:1 w:1)
    fn on_initialize_periodic_resolved(s: u32) -> Weight {
        (8_100_000 as Weight)
            // Standard Error: 455_000
            .saturating_add((42_930_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().reads((3 as Weight).saturating_mul(s as Weight)))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
            .saturating_add(T::DbWeight::get().writes((3 as Weight).saturating_mul(s as Weight)))
    }
    // Storage: Scheduler Agenda (r:1 w:1)
    // Storage: Preimage PreimageFor (r:1 w:1)
    // Storage: Preimage StatusFor (r:1 w:1)
    fn on_initialize_resolved(s: u32) -> Weight {
        (13_719_000 as Weight)
            // Standard Error: 342_000
            .saturating_add((36_158_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().reads((2 as Weight).saturating_mul(s as Weight)))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
            .saturating_add(T::DbWeight::get().writes((2 as Weight).saturating_mul(s as Weight)))
    }
    // Storage: Scheduler Agenda (r:2 w:2)
    // Storage: Preimage PreimageFor (r:1 w:0)
    // Storage: Scheduler Lookup (r:0 w:1)
    fn on_initialize_named_aborted(s: u32) -> Weight {
        (20_404_000 as Weight)
            // Standard Error: 78_000
            .saturating_add((16_125_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(s as Weight)))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
            .saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(s as Weight)))
    }
    // Storage: Scheduler Agenda (r:2 w:2)
    // Storage: Preimage PreimageFor (r:1 w:0)
    fn on_initialize_aborted(s: u32) -> Weight {
        (25_240_000 as Weight)
            // Standard Error: 59_000
            .saturating_add((8_957_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(s as Weight)))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: Scheduler Agenda (r:2 w:2)
    // Storage: Scheduler Lookup (r:0 w:1)
    fn on_initialize_periodic_named(s: u32) -> Weight {
        (57_297_000 as Weight)
            // Standard Error: 112_000
            .saturating_add((23_429_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(s as Weight)))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
            .saturating_add(T::DbWeight::get().writes((2 as Weight).saturating_mul(s as Weight)))
    }
    // Storage: Scheduler Agenda (r:2 w:2)
    fn on_initialize_periodic(s: u32) -> Weight {
        (58_724_000 as Weight)
            // Standard Error: 179_000
            .saturating_add((16_953_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(s as Weight)))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
            .saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(s as Weight)))
    }
    // Storage: Scheduler Agenda (r:1 w:1)
    // Storage: Scheduler Lookup (r:0 w:1)
    fn on_initialize_named(s: u32) -> Weight {
        (35_503_000 as Weight)
            // Standard Error: 63_000
            .saturating_add((12_945_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
            .saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(s as Weight)))
    }
    // Storage: Scheduler Agenda (r:1 w:1)
    fn on_initialize(s: u32) -> Weight {
        (56_165_000 as Weight)
            // Standard Error: 153_000
            .saturating_add((9_751_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: Scheduler Agenda (r:1 w:1)
    fn schedule(s: u32) -> Weight {
        (37_674_000 as Weight)
            // Standard Error: 5_000
            .saturating_add((124_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: Scheduler Agenda (r:1 w:1)
    // Storage: Scheduler Lookup (r:0 w:1)
    fn cancel(s: u32) -> Weight {
        (35_256_000 as Weight)
            // Standard Error: 29_000
            .saturating_add((1_522_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: Scheduler Lookup (r:1 w:1)
    // Storage: Scheduler Agenda (r:1 w:1)
    fn schedule_named(s: u32) -> Weight {
        (48_242_000 as Weight)
            // Standard Error: 11_000
            .saturating_add((103_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: Scheduler Lookup (r:1 w:1)
    // Storage: Scheduler Agenda (r:1 w:1)
    fn cancel_named(s: u32) -> Weight {
        (42_748_000 as Weight)
            // Standard Error: 10_000
            .saturating_add((1_344_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
}
