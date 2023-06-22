// Copyright 2022-2023 Forecasting Technologies LTD.
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

//! Autogenerated weights for frame_system
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-06-16, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/production/zeitgeist
// benchmark
// pallet
// --chain=dev
// --steps=50
// --repeat=20
// --pallet=frame_system
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

/// Weight functions for frame_system (automatically generated)
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> frame_system::weights::WeightInfo for WeightInfo<T> {
    fn remark(b: u32) -> Weight {
        Weight::from_ref_time(7_750_000)
            // Standard Error: 1
            .saturating_add(Weight::from_ref_time(692).saturating_mul(b.into()))
    }
    fn remark_with_event(b: u32) -> Weight {
        Weight::from_ref_time(23_520_000)
            // Standard Error: 4
            .saturating_add(Weight::from_ref_time(2_341).saturating_mul(b.into()))
    }
    // Storage: System Digest (r:1 w:1)
    // Storage: unknown [0x3a686561707061676573] (r:0 w:1)
    fn set_heap_pages() -> Weight {
        Weight::from_ref_time(16_260_000)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    // Storage: Skipped Metadata (r:0 w:0)
    fn set_storage(i: u32) -> Weight {
        Weight::from_ref_time(7_690_000)
            // Standard Error: 4_435
            .saturating_add(Weight::from_ref_time(1_133_019).saturating_mul(i.into()))
            .saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(i.into())))
    }
    // Storage: Skipped Metadata (r:0 w:0)
    fn kill_storage(i: u32) -> Weight {
        Weight::from_ref_time(589_951)
            // Standard Error: 8_232
            .saturating_add(Weight::from_ref_time(925_898).saturating_mul(i.into()))
            .saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(i.into())))
    }
    // Storage: Skipped Metadata (r:0 w:0)
    fn kill_prefix(p: u32) -> Weight {
        Weight::from_ref_time(10_930_000)
            // Standard Error: 10_457
            .saturating_add(Weight::from_ref_time(2_090_894).saturating_mul(p.into()))
            .saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(p.into())))
    }
}
