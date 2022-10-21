// Copyright 2021-2022 Zeitgeist PM LLC.
//
// This file is part of Zeitgeist.
//
// Zeitgeist is free software: you can redistribute it and/or modify it
// under the terms of the GNU General Public License as published by the
// Free Software Foundation, either version 3 of the License, or (at
// your option) any later version.
//
// Zeitgeist is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Zeitgeist. If not, see <https://www.gnu.org/licenses/>.

//! Autogenerated weights for pallet_multisig
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-10-17, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/production/zeitgeist
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
    fn as_multi_threshold_1(_z: u32) -> Weight {
        (21_403_000 as Weight)
    }
    // Storage: MultiSig Multisigs (r:1 w:1)
    // Storage: unknown [0x3a65787472696e7369635f696e646578] (r:1 w:0)
    fn as_multi_create(s: u32, z: u32) -> Weight {
        (49_442_000 as Weight)
            // Standard Error: 15_000
            .saturating_add((249_000 as Weight).saturating_mul(s as Weight))
            // Standard Error: 0
            .saturating_add((2_000 as Weight).saturating_mul(z as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: MultiSig Multisigs (r:1 w:1)
    // Storage: MultiSig Calls (r:1 w:1)
    // Storage: unknown [0x3a65787472696e7369635f696e646578] (r:1 w:0)
    fn as_multi_create_store(s: u32, z: u32) -> Weight {
        (69_514_000 as Weight)
            // Standard Error: 8_000
            .saturating_add((84_000 as Weight).saturating_mul(s as Weight))
            // Standard Error: 0
            .saturating_add((1_000 as Weight).saturating_mul(z as Weight))
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: MultiSig Multisigs (r:1 w:1)
    fn as_multi_approve(s: u32, z: u32) -> Weight {
        (37_432_000 as Weight)
            // Standard Error: 6_000
            .saturating_add((133_000 as Weight).saturating_mul(s as Weight))
            // Standard Error: 0
            .saturating_add((1_000 as Weight).saturating_mul(z as Weight))
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: MultiSig Multisigs (r:1 w:1)
    // Storage: MultiSig Calls (r:1 w:1)
    fn as_multi_approve_store(s: u32, z: u32) -> Weight {
        (60_493_000 as Weight)
            // Standard Error: 11_000
            .saturating_add((105_000 as Weight).saturating_mul(s as Weight))
            // Standard Error: 0
            .saturating_add((2_000 as Weight).saturating_mul(z as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: MultiSig Multisigs (r:1 w:1)
    // Storage: MultiSig Calls (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    fn as_multi_complete(s: u32, z: u32) -> Weight {
        (64_232_000 as Weight)
            // Standard Error: 15_000
            .saturating_add((303_000 as Weight).saturating_mul(s as Weight))
            // Standard Error: 0
            .saturating_add((3_000 as Weight).saturating_mul(z as Weight))
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    // Storage: MultiSig Multisigs (r:1 w:1)
    // Storage: unknown [0x3a65787472696e7369635f696e646578] (r:1 w:0)
    fn approve_as_multi_create(s: u32) -> Weight {
        (52_061_000 as Weight)
            // Standard Error: 8_000
            .saturating_add((125_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: MultiSig Multisigs (r:1 w:1)
    // Storage: MultiSig Calls (r:1 w:0)
    fn approve_as_multi_approve(s: u32) -> Weight {
        (33_511_000 as Weight)
            // Standard Error: 6_000
            .saturating_add((137_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: MultiSig Multisigs (r:1 w:1)
    // Storage: MultiSig Calls (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    fn approve_as_multi_complete(s: u32) -> Weight {
        (88_898_000 as Weight)
            // Standard Error: 17_000
            .saturating_add((310_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    // Storage: MultiSig Multisigs (r:1 w:1)
    // Storage: MultiSig Calls (r:1 w:1)
    fn cancel_as_multi(s: u32) -> Weight {
        (77_910_000 as Weight)
            // Standard Error: 10_000
            .saturating_add((105_000 as Weight).saturating_mul(s as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
}
