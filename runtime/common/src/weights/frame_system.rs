// Copyright 2022-2024 Forecasting Technologies LTD.
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
//! DATE: `2024-08-27`, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `zeitgeist-benchmark`, CPU: `AMD EPYC 7601 32-Core Processor`
//! EXECUTION: ``, WASM-EXECUTION: `Compiled`, CHAIN: `Some("dev")`, DB CACHE: `1024`

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
        // Minimum execution time: 2_520 nanoseconds.
        Weight::from_parts(2_570_000, 0)
            // Standard Error: 0
            .saturating_add(Weight::from_parts(334, 0).saturating_mul(b.into()))
    }
    /// The range of component `b` is `[0, 3932160]`.
    fn remark_with_event(b: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 8_740 nanoseconds.
        Weight::from_parts(9_051_000, 0)
            // Standard Error: 1
            .saturating_add(Weight::from_parts(1_499, 0).saturating_mul(b.into()))
    }
    /// Storage: `System::Digest` (r:1 w:1)
    /// Proof: `System::Digest` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: UNKNOWN KEY `0x3a686561707061676573` (r:0 w:1)
    /// Proof: UNKNOWN KEY `0x3a686561707061676573` (r:0 w:1)
    fn set_heap_pages() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `1485`
        // Minimum execution time: 5_250 nanoseconds.
        Weight::from_parts(5_460_000, 1485)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    /// Storage: `System::Digest` (r:1 w:1)
    /// Proof: `System::Digest` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: UNKNOWN KEY `0x3a636f6465` (r:0 w:1)
    /// Proof: UNKNOWN KEY `0x3a636f6465` (r:0 w:1)
    fn set_code() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `1485`
        // Minimum execution time: 84_704_867 nanoseconds.
        Weight::from_parts(92_806_077_000, 1485)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    /// Storage: `Skipped::Metadata` (r:0 w:0)
    /// Proof: `Skipped::Metadata` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// The range of component `i` is `[0, 1000]`.
    fn set_storage(i: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 2_430 nanoseconds.
        Weight::from_parts(2_521_000, 0)
            // Standard Error: 1_257
            .saturating_add(Weight::from_parts(960_659, 0).saturating_mul(i.into()))
            .saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(i.into())))
    }
    /// Storage: `Skipped::Metadata` (r:0 w:0)
    /// Proof: `Skipped::Metadata` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// The range of component `i` is `[0, 1000]`.
    fn kill_storage(i: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 2_410 nanoseconds.
        Weight::from_parts(2_540_000, 0)
            // Standard Error: 1_438
            .saturating_add(Weight::from_parts(717_594, 0).saturating_mul(i.into()))
            .saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(i.into())))
    }
    /// Storage: `Skipped::Metadata` (r:0 w:0)
    /// Proof: `Skipped::Metadata` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// The range of component `p` is `[0, 1000]`.
    fn kill_prefix(p: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `48 + p * (69 ±0)`
        //  Estimated: `52 + p * (70 ±0)`
        // Minimum execution time: 4_460 nanoseconds.
        Weight::from_parts(4_640_000, 52)
            // Standard Error: 1_445
            .saturating_add(Weight::from_parts(1_389_405, 0).saturating_mul(p.into()))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(p.into())))
            .saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(p.into())))
            .saturating_add(Weight::from_parts(0, 70).saturating_mul(p.into()))
    }
}
