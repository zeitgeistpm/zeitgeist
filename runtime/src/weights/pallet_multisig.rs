//! Autogenerated weights for pallet_multisig
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-06-16, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/zeitgeist
// benchmark
// pallet
// --chain=dev
// --steps=50
// --repeat=20
// --pallet=pallet_multisig
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

/// Weight functions for pallet_multisig (automatically generated)
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_multisig::weights::WeightInfo for WeightInfo<T> {
    fn as_multi_threshold_1(_z: u32) -> Weight {
        (29_238_000 as Weight)
    }
    // Storage: MultiSig Multisigs (r:1 w:1)
    // Storage: unknown [0x3a65787472696e7369635f696e646578] (r:1 w:0)
    fn as_multi_create(s: u32, z: u32) -> Weight {
        (92_504_000 as Weight)
            // Standard Error: 14_000
            .saturating_add((166_000 as Weight).saturating_mul(s as Weight))
            // Standard Error: 0
            .saturating_add((1_000 as Weight).saturating_mul(z as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: MultiSig Multisigs (r:1 w:1)
    // Storage: MultiSig Calls (r:1 w:1)
    // Storage: unknown [0x3a65787472696e7369635f696e646578] (r:1 w:0)
    fn as_multi_create_store(s: u32, z: u32) -> Weight {
        (84_340_000 as Weight)
            // Standard Error: 13_000
            .saturating_add((206_000 as Weight).saturating_mul(s as Weight))
            // Standard Error: 0
            .saturating_add((3_000 as Weight).saturating_mul(z as Weight))
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: MultiSig Multisigs (r:1 w:1)
    fn as_multi_approve(s: u32, z: u32) -> Weight {
        (52_600_000 as Weight)
            // Standard Error: 6_000
            .saturating_add((151_000 as Weight).saturating_mul(s as Weight))
            // Standard Error: 0
            .saturating_add((1_000 as Weight).saturating_mul(z as Weight))
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: MultiSig Multisigs (r:1 w:1)
    // Storage: MultiSig Calls (r:1 w:1)
    fn as_multi_approve_store(s: u32, z: u32) -> Weight {
        (84_637_000 as Weight)
            // Standard Error: 12_000
            .saturating_add((180_000 as Weight).saturating_mul(s as Weight))
            // Standard Error: 0
            .saturating_add((2_000 as Weight).saturating_mul(z as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: MultiSig Multisigs (r:1 w:1)
    // Storage: MultiSig Calls (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    fn as_multi_complete(s: u32, z: u32) -> Weight {
        (93_114_000 as Weight)
            // Standard Error: 15_000
            .saturating_add((521_000 as Weight).saturating_mul(s as Weight))
            // Standard Error: 0
            .saturating_add((3_000 as Weight).saturating_mul(z as Weight))
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    // Storage: MultiSig Multisigs (r:1 w:1)
    // Storage: unknown [0x3a65787472696e7369635f696e646578] (r:1 w:0)
    fn approve_as_multi_create(s: u32) -> Weight {
        (83_324_000 as Weight)
            // Standard Error: 11_000
            .saturating_add((187_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: MultiSig Multisigs (r:1 w:1)
    // Storage: MultiSig Calls (r:1 w:0)
    fn approve_as_multi_approve(s: u32) -> Weight {
        (51_657_000 as Weight)
            // Standard Error: 6_000
            .saturating_add((170_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: MultiSig Multisigs (r:1 w:1)
    // Storage: MultiSig Calls (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    fn approve_as_multi_complete(s: u32) -> Weight {
        (134_684_000 as Weight)
            // Standard Error: 15_000
            .saturating_add((174_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    // Storage: MultiSig Multisigs (r:1 w:1)
    // Storage: MultiSig Calls (r:1 w:1)
    fn cancel_as_multi(s: u32) -> Weight {
        (105_567_000 as Weight)
            // Standard Error: 14_000
            .saturating_add((148_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
}
