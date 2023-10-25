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
//! DATE: `2023-10-11`, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `zeitgeist-benchmark`, CPU: `AMD EPYC 7601 32-Core Processor`
//! EXECUTION: `Some(Wasm)`, WASM-EXECUTION: `Compiled`, CHAIN: `Some("dev")`, DB CACHE: `1024`

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
// --header=./HEADER_GPL3
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
    /// The range of component `b` is `[0, 3932160]`.
    fn remark(b: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 3_330 nanoseconds.
        Weight::from_parts(3_530_000, 0)
            // Standard Error: 1
            .saturating_add(Weight::from_parts(832, 0).saturating_mul(b.into()))
    }
    /// The range of component `b` is `[0, 3932160]`.
    fn remark_with_event(b: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 11_890 nanoseconds.
        Weight::from_parts(12_270_000, 0)
            // Standard Error: 5
            .saturating_add(Weight::from_parts(2_541, 0).saturating_mul(b.into()))
    }
    /// Storage: System Digest (r:1 w:1)
    /// Proof Skipped: System Digest (max_values: Some(1), max_size: None, mode: Measured)
    /// Storage: unknown `0x3a686561707061676573` (r:0 w:1)
    /// Proof Skipped: unknown `0x3a686561707061676573` (r:0 w:1)
    fn set_heap_pages() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `495`
        // Minimum execution time: 7_030 nanoseconds.
        Weight::from_parts(7_850_000, 495)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    /// Storage: Skipped Metadata (r:0 w:0)
    /// Proof Skipped: Skipped Metadata (max_values: None, max_size: None, mode: Measured)
    /// The range of component `i` is `[0, 1000]`.
    fn set_storage(i: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 3_570 nanoseconds.
        Weight::from_parts(3_640_000, 0)
            // Standard Error: 4_801
            .saturating_add(Weight::from_parts(1_148_373, 0).saturating_mul(i.into()))
            .saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(i.into())))
    }
    /// Storage: Skipped Metadata (r:0 w:0)
    /// Proof Skipped: Skipped Metadata (max_values: None, max_size: None, mode: Measured)
    /// The range of component `i` is `[0, 1000]`.
    fn kill_storage(i: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 3_430 nanoseconds.
        Weight::from_parts(3_640_000, 0)
            // Standard Error: 4_580
            .saturating_add(Weight::from_parts(943_402, 0).saturating_mul(i.into()))
            .saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(i.into())))
    }
    /// Storage: Skipped Metadata (r:0 w:0)
    /// Proof Skipped: Skipped Metadata (max_values: None, max_size: None, mode: Measured)
    /// The range of component `p` is `[0, 1000]`.
    fn kill_prefix(p: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `50 + p * (69 ±0)`
        //  Estimated: `53 + p * (70 ±0)`
        // Minimum execution time: 7_060 nanoseconds.
        Weight::from_parts(7_320_000, 53)
            // Standard Error: 8_444
            .saturating_add(Weight::from_parts(2_133_080, 0).saturating_mul(p.into()))
            .saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(p.into())))
            .saturating_add(Weight::from_parts(0, 70).saturating_mul(p.into()))
    }
}
