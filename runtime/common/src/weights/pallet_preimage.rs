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

//! Autogenerated weights for pallet_preimage
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: `2023-09-11`, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
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
// --pallet=pallet_preimage
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

/// Weight functions for pallet_preimage (automatically generated)
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_preimage::weights::WeightInfo for WeightInfo<T> {
    /// Storage: Preimage StatusFor (r:1 w:1)
    /// Proof: Preimage StatusFor (max_values: None, max_size: Some(91), added: 2566, mode: MaxEncodedLen)
    /// Storage: Preimage PreimageFor (r:0 w:1)
    /// Proof: Preimage PreimageFor (max_values: None, max_size: Some(4194344), added: 4196819, mode: MaxEncodedLen)
    fn note_preimage(s: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `179`
        //  Estimated: `2566`
        // Minimum execution time: 39_770 nanoseconds.
        Weight::from_parts(47_381_000, 2566)
            // Standard Error: 6
            .saturating_add(Weight::from_ref_time(3_373).saturating_mul(s.into()))
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    /// Storage: Preimage StatusFor (r:1 w:1)
    /// Proof: Preimage StatusFor (max_values: None, max_size: Some(91), added: 2566, mode: MaxEncodedLen)
    /// Storage: Preimage PreimageFor (r:0 w:1)
    /// Proof: Preimage PreimageFor (max_values: None, max_size: Some(4194344), added: 4196819, mode: MaxEncodedLen)
    fn note_requested_preimage(s: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `106`
        //  Estimated: `2566`
        // Minimum execution time: 28_380 nanoseconds.
        Weight::from_parts(28_881_000, 2566)
            // Standard Error: 5
            .saturating_add(Weight::from_ref_time(3_385).saturating_mul(s.into()))
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    /// Storage: Preimage StatusFor (r:1 w:1)
    /// Proof: Preimage StatusFor (max_values: None, max_size: Some(91), added: 2566, mode: MaxEncodedLen)
    /// Storage: Preimage PreimageFor (r:0 w:1)
    /// Proof: Preimage PreimageFor (max_values: None, max_size: Some(4194344), added: 4196819, mode: MaxEncodedLen)
    fn note_no_deposit_preimage(s: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `106`
        //  Estimated: `2566`
        // Minimum execution time: 22_250 nanoseconds.
        Weight::from_parts(22_930_000, 2566)
            // Standard Error: 5
            .saturating_add(Weight::from_ref_time(3_353).saturating_mul(s.into()))
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    /// Storage: Preimage StatusFor (r:1 w:1)
    /// Proof: Preimage StatusFor (max_values: None, max_size: Some(91), added: 2566, mode: MaxEncodedLen)
    /// Storage: Preimage PreimageFor (r:0 w:1)
    /// Proof: Preimage PreimageFor (max_values: None, max_size: Some(4194344), added: 4196819, mode: MaxEncodedLen)
    fn unnote_preimage() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `357`
        //  Estimated: `2566`
        // Minimum execution time: 65_570 nanoseconds.
        Weight::from_parts(74_720_000, 2566)
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    /// Storage: Preimage StatusFor (r:1 w:1)
    /// Proof: Preimage StatusFor (max_values: None, max_size: Some(91), added: 2566, mode: MaxEncodedLen)
    /// Storage: Preimage PreimageFor (r:0 w:1)
    /// Proof: Preimage PreimageFor (max_values: None, max_size: Some(4194344), added: 4196819, mode: MaxEncodedLen)
    fn unnote_no_deposit_preimage() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `144`
        //  Estimated: `2566`
        // Minimum execution time: 46_820 nanoseconds.
        Weight::from_parts(53_090_000, 2566)
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    /// Storage: Preimage StatusFor (r:1 w:1)
    /// Proof: Preimage StatusFor (max_values: None, max_size: Some(91), added: 2566, mode: MaxEncodedLen)
    fn request_preimage() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `220`
        //  Estimated: `2566`
        // Minimum execution time: 43_940 nanoseconds.
        Weight::from_parts(54_850_000, 2566)
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
    /// Storage: Preimage StatusFor (r:1 w:1)
    /// Proof: Preimage StatusFor (max_values: None, max_size: Some(91), added: 2566, mode: MaxEncodedLen)
    fn request_no_deposit_preimage() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `144`
        //  Estimated: `2566`
        // Minimum execution time: 29_810 nanoseconds.
        Weight::from_parts(32_701_000, 2566)
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
    /// Storage: Preimage StatusFor (r:1 w:1)
    /// Proof: Preimage StatusFor (max_values: None, max_size: Some(91), added: 2566, mode: MaxEncodedLen)
    fn request_unnoted_preimage() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `42`
        //  Estimated: `2566`
        // Minimum execution time: 34_820 nanoseconds.
        Weight::from_parts(38_670_000, 2566)
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
    /// Storage: Preimage StatusFor (r:1 w:1)
    /// Proof: Preimage StatusFor (max_values: None, max_size: Some(91), added: 2566, mode: MaxEncodedLen)
    fn request_requested_preimage() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `106`
        //  Estimated: `2566`
        // Minimum execution time: 16_410 nanoseconds.
        Weight::from_parts(20_020_000, 2566)
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
    /// Storage: Preimage StatusFor (r:1 w:1)
    /// Proof: Preimage StatusFor (max_values: None, max_size: Some(91), added: 2566, mode: MaxEncodedLen)
    /// Storage: Preimage PreimageFor (r:0 w:1)
    /// Proof: Preimage PreimageFor (max_values: None, max_size: Some(4194344), added: 4196819, mode: MaxEncodedLen)
    fn unrequest_preimage() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `144`
        //  Estimated: `2566`
        // Minimum execution time: 44_190 nanoseconds.
        Weight::from_parts(52_690_000, 2566)
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    /// Storage: Preimage StatusFor (r:1 w:1)
    /// Proof: Preimage StatusFor (max_values: None, max_size: Some(91), added: 2566, mode: MaxEncodedLen)
    fn unrequest_unnoted_preimage() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `106`
        //  Estimated: `2566`
        // Minimum execution time: 16_190 nanoseconds.
        Weight::from_parts(19_450_000, 2566)
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
    /// Storage: Preimage StatusFor (r:1 w:1)
    /// Proof: Preimage StatusFor (max_values: None, max_size: Some(91), added: 2566, mode: MaxEncodedLen)
    fn unrequest_multi_referenced_preimage() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `106`
        //  Estimated: `2566`
        // Minimum execution time: 16_310 nanoseconds.
        Weight::from_parts(20_150_000, 2566)
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
}
