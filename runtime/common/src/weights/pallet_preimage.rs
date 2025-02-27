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

//! Autogenerated weights for pallet_preimage
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: `2025-02-26`, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `ztg-benchmark`, CPU: `AMD EPYC 7601 32-Core Processor`
//! EXECUTION: ``, WASM-EXECUTION: `Compiled`, CHAIN: `Some("dev")`, DB CACHE: `1024`

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
    /// Storage: `Preimage::StatusFor` (r:1 w:1)
    /// Proof: `Preimage::StatusFor` (`max_values`: None, `max_size`: Some(91), added: 2566, mode: `MaxEncodedLen`)
    /// Storage: `Preimage::PreimageFor` (r:0 w:1)
    /// Proof: `Preimage::PreimageFor` (`max_values`: None, `max_size`: Some(4194344), added: 4196819, mode: `MaxEncodedLen`)
    /// The range of component `s` is `[0, 4194304]`.
    fn note_preimage(s: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `147`
        //  Estimated: `3556`
        // Minimum execution time: 34_831 nanoseconds.
        Weight::from_parts(35_661_000, 3556)
            // Standard Error: 2
            .saturating_add(Weight::from_parts(1_833, 0).saturating_mul(s.into()))
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    /// Storage: `Preimage::StatusFor` (r:1 w:1)
    /// Proof: `Preimage::StatusFor` (`max_values`: None, `max_size`: Some(91), added: 2566, mode: `MaxEncodedLen`)
    /// Storage: `Preimage::PreimageFor` (r:0 w:1)
    /// Proof: `Preimage::PreimageFor` (`max_values`: None, `max_size`: Some(4194344), added: 4196819, mode: `MaxEncodedLen`)
    /// The range of component `s` is `[0, 4194304]`.
    fn note_requested_preimage(s: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `106`
        //  Estimated: `3556`
        // Minimum execution time: 19_520 nanoseconds.
        Weight::from_parts(19_981_000, 3556)
            // Standard Error: 2
            .saturating_add(Weight::from_parts(1_885, 0).saturating_mul(s.into()))
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    /// Storage: `Preimage::StatusFor` (r:1 w:1)
    /// Proof: `Preimage::StatusFor` (`max_values`: None, `max_size`: Some(91), added: 2566, mode: `MaxEncodedLen`)
    /// Storage: `Preimage::PreimageFor` (r:0 w:1)
    /// Proof: `Preimage::PreimageFor` (`max_values`: None, `max_size`: Some(4194344), added: 4196819, mode: `MaxEncodedLen`)
    /// The range of component `s` is `[0, 4194304]`.
    fn note_no_deposit_preimage(s: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `106`
        //  Estimated: `3556`
        // Minimum execution time: 19_271 nanoseconds.
        Weight::from_parts(19_480_000, 3556)
            // Standard Error: 2
            .saturating_add(Weight::from_parts(1_822, 0).saturating_mul(s.into()))
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    /// Storage: `Preimage::StatusFor` (r:1 w:1)
    /// Proof: `Preimage::StatusFor` (`max_values`: None, `max_size`: Some(91), added: 2566, mode: `MaxEncodedLen`)
    /// Storage: `Preimage::PreimageFor` (r:0 w:1)
    /// Proof: `Preimage::PreimageFor` (`max_values`: None, `max_size`: Some(4194344), added: 4196819, mode: `MaxEncodedLen`)
    fn unnote_preimage() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `293`
        //  Estimated: `3556`
        // Minimum execution time: 49_121 nanoseconds.
        Weight::from_parts(51_371_000, 3556)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    /// Storage: `Preimage::StatusFor` (r:1 w:1)
    /// Proof: `Preimage::StatusFor` (`max_values`: None, `max_size`: Some(91), added: 2566, mode: `MaxEncodedLen`)
    /// Storage: `Preimage::PreimageFor` (r:0 w:1)
    /// Proof: `Preimage::PreimageFor` (`max_values`: None, `max_size`: Some(4194344), added: 4196819, mode: `MaxEncodedLen`)
    fn unnote_no_deposit_preimage() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `144`
        //  Estimated: `3556`
        // Minimum execution time: 30_730 nanoseconds.
        Weight::from_parts(32_750_000, 3556)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    /// Storage: `Preimage::StatusFor` (r:1 w:1)
    /// Proof: `Preimage::StatusFor` (`max_values`: None, `max_size`: Some(91), added: 2566, mode: `MaxEncodedLen`)
    fn request_preimage() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `188`
        //  Estimated: `3556`
        // Minimum execution time: 26_391 nanoseconds.
        Weight::from_parts(27_691_000, 3556)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    /// Storage: `Preimage::StatusFor` (r:1 w:1)
    /// Proof: `Preimage::StatusFor` (`max_values`: None, `max_size`: Some(91), added: 2566, mode: `MaxEncodedLen`)
    fn request_no_deposit_preimage() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `144`
        //  Estimated: `3556`
        // Minimum execution time: 17_691 nanoseconds.
        Weight::from_parts(18_441_000, 3556)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    /// Storage: `Preimage::StatusFor` (r:1 w:1)
    /// Proof: `Preimage::StatusFor` (`max_values`: None, `max_size`: Some(91), added: 2566, mode: `MaxEncodedLen`)
    fn request_unnoted_preimage() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `42`
        //  Estimated: `3556`
        // Minimum execution time: 19_551 nanoseconds.
        Weight::from_parts(20_350_000, 3556)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    /// Storage: `Preimage::StatusFor` (r:1 w:1)
    /// Proof: `Preimage::StatusFor` (`max_values`: None, `max_size`: Some(91), added: 2566, mode: `MaxEncodedLen`)
    fn request_requested_preimage() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `106`
        //  Estimated: `3556`
        // Minimum execution time: 12_521 nanoseconds.
        Weight::from_parts(12_920_000, 3556)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    /// Storage: `Preimage::StatusFor` (r:1 w:1)
    /// Proof: `Preimage::StatusFor` (`max_values`: None, `max_size`: Some(91), added: 2566, mode: `MaxEncodedLen`)
    /// Storage: `Preimage::PreimageFor` (r:0 w:1)
    /// Proof: `Preimage::PreimageFor` (`max_values`: None, `max_size`: Some(4194344), added: 4196819, mode: `MaxEncodedLen`)
    fn unrequest_preimage() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `144`
        //  Estimated: `3556`
        // Minimum execution time: 27_980 nanoseconds.
        Weight::from_parts(30_071_000, 3556)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    /// Storage: `Preimage::StatusFor` (r:1 w:1)
    /// Proof: `Preimage::StatusFor` (`max_values`: None, `max_size`: Some(91), added: 2566, mode: `MaxEncodedLen`)
    fn unrequest_unnoted_preimage() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `106`
        //  Estimated: `3556`
        // Minimum execution time: 12_130 nanoseconds.
        Weight::from_parts(12_761_000, 3556)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    /// Storage: `Preimage::StatusFor` (r:1 w:1)
    /// Proof: `Preimage::StatusFor` (`max_values`: None, `max_size`: Some(91), added: 2566, mode: `MaxEncodedLen`)
    fn unrequest_multi_referenced_preimage() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `106`
        //  Estimated: `3556`
        // Minimum execution time: 12_490 nanoseconds.
        Weight::from_parts(12_860_000, 3556)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
}
