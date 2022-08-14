//! Autogenerated weights for pallet_multisig
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-08-13, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
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
// --output=./runtime/common/src/weights/

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
    fn as_multi_threshold_1(z: u32) -> Weight {
        (27_230_000 as Weight)
            // Standard Error: 0
            .saturating_add((1_000 as Weight).saturating_mul(z as Weight))
    }
    // Storage: MultiSig Multisigs (r:1 w:1)
    // Storage: unknown [0x3a65787472696e7369635f696e646578] (r:1 w:0)
    fn as_multi_create(s: u32, z: u32) -> Weight {
        (83_858_000 as Weight)
            // Standard Error: 11_000
            .saturating_add((298_000 as Weight).saturating_mul(s as Weight))
            // Standard Error: 0
            .saturating_add((2_000 as Weight).saturating_mul(z as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: MultiSig Multisigs (r:1 w:1)
    // Storage: MultiSig Calls (r:1 w:1)
    // Storage: unknown [0x3a65787472696e7369635f696e646578] (r:1 w:0)
    fn as_multi_create_store(s: u32, z: u32) -> Weight {
        (102_384_000 as Weight)
            // Standard Error: 13_000
            .saturating_add((139_000 as Weight).saturating_mul(s as Weight))
            // Standard Error: 0
            .saturating_add((1_000 as Weight).saturating_mul(z as Weight))
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: MultiSig Multisigs (r:1 w:1)
    fn as_multi_approve(s: u32, z: u32) -> Weight {
        (45_192_000 as Weight)
            // Standard Error: 11_000
            .saturating_add((251_000 as Weight).saturating_mul(s as Weight))
            // Standard Error: 0
            .saturating_add((2_000 as Weight).saturating_mul(z as Weight))
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: MultiSig Multisigs (r:1 w:1)
    // Storage: MultiSig Calls (r:1 w:1)
    fn as_multi_approve_store(s: u32, z: u32) -> Weight {
        (56_845_000 as Weight)
            // Standard Error: 14_000
            .saturating_add((378_000 as Weight).saturating_mul(s as Weight))
            // Standard Error: 0
            .saturating_add((4_000 as Weight).saturating_mul(z as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: MultiSig Multisigs (r:1 w:1)
    // Storage: MultiSig Calls (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    fn as_multi_complete(s: u32, z: u32) -> Weight {
        (107_976_000 as Weight)
            // Standard Error: 16_000
            .saturating_add((272_000 as Weight).saturating_mul(s as Weight))
            // Standard Error: 0
            .saturating_add((4_000 as Weight).saturating_mul(z as Weight))
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    // Storage: MultiSig Multisigs (r:1 w:1)
    // Storage: unknown [0x3a65787472696e7369635f696e646578] (r:1 w:0)
    fn approve_as_multi_create(s: u32) -> Weight {
        (78_894_000 as Weight)
            // Standard Error: 12_000
            .saturating_add((221_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: MultiSig Multisigs (r:1 w:1)
    // Storage: MultiSig Calls (r:1 w:0)
    fn approve_as_multi_approve(s: u32) -> Weight {
        (50_776_000 as Weight)
            // Standard Error: 6_000
            .saturating_add((161_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: MultiSig Multisigs (r:1 w:1)
    // Storage: MultiSig Calls (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    fn approve_as_multi_complete(s: u32) -> Weight {
        (138_651_000 as Weight)
            // Standard Error: 18_000
            .saturating_add((191_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    // Storage: MultiSig Multisigs (r:1 w:1)
    // Storage: MultiSig Calls (r:1 w:1)
    fn cancel_as_multi(s: u32) -> Weight {
        (112_805_000 as Weight)
            // Standard Error: 23_000
            .saturating_add((238_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
}
