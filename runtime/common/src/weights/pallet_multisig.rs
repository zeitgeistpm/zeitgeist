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

//! Autogenerated weights for pallet_multisig
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: `2024-08-12`, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
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
// --pallet=pallet_multisig
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

/// Weight functions for pallet_multisig (automatically generated)
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_multisig::weights::WeightInfo for WeightInfo<T> {
    /// The range of component `z` is `[0, 10000]`.
    fn as_multi_threshold_1(z: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 14_540 nanoseconds.
        Weight::from_parts(16_492_945, 0)
            // Standard Error: 8
            .saturating_add(Weight::from_parts(467, 0).saturating_mul(z.into()))
    }
    /// Storage: `Multisig::Multisigs` (r:1 w:1)
    /// Proof: `Multisig::Multisigs` (`max_values`: None, `max_size`: Some(3350), added: 5825, mode: `MaxEncodedLen`)
    /// The range of component `s` is `[2, 100]`.
    /// The range of component `z` is `[0, 10000]`.
    fn as_multi_create(s: u32, z: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `275 + s * (1 ±0)`
        //  Estimated: `6815`
        // Minimum execution time: 47_391 nanoseconds.
        Weight::from_parts(40_119_455, 6815)
            // Standard Error: 1_972
            .saturating_add(Weight::from_parts(109_190, 0).saturating_mul(s.into()))
            // Standard Error: 19
            .saturating_add(Weight::from_parts(1_655, 0).saturating_mul(z.into()))
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    /// Storage: `Multisig::Multisigs` (r:1 w:1)
    /// Proof: `Multisig::Multisigs` (`max_values`: None, `max_size`: Some(3350), added: 5825, mode: `MaxEncodedLen`)
    /// The range of component `s` is `[3, 100]`.
    /// The range of component `z` is `[0, 10000]`.
    fn as_multi_approve(s: u32, z: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `252`
        //  Estimated: `6815`
        // Minimum execution time: 30_861 nanoseconds.
        Weight::from_parts(24_416_204, 6815)
            // Standard Error: 1_236
            .saturating_add(Weight::from_parts(93_258, 0).saturating_mul(s.into()))
            // Standard Error: 12
            .saturating_add(Weight::from_parts(1_570, 0).saturating_mul(z.into()))
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    /// Storage: `Multisig::Multisigs` (r:1 w:1)
    /// Proof: `Multisig::Multisigs` (`max_values`: None, `max_size`: Some(3350), added: 5825, mode: `MaxEncodedLen`)
    /// Storage: `System::Account` (r:1 w:1)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(132), added: 2607, mode: `MaxEncodedLen`)
    /// The range of component `s` is `[2, 100]`.
    /// The range of component `z` is `[0, 10000]`.
    fn as_multi_complete(s: u32, z: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `362 + s * (33 ±0)`
        //  Estimated: `6815`
        // Minimum execution time: 55_211 nanoseconds.
        Weight::from_parts(45_366_942, 6815)
            // Standard Error: 1_775
            .saturating_add(Weight::from_parts(127_546, 0).saturating_mul(s.into()))
            // Standard Error: 17
            .saturating_add(Weight::from_parts(1_690, 0).saturating_mul(z.into()))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    /// Storage: `Multisig::Multisigs` (r:1 w:1)
    /// Proof: `Multisig::Multisigs` (`max_values`: None, `max_size`: Some(3350), added: 5825, mode: `MaxEncodedLen`)
    /// The range of component `s` is `[2, 100]`.
    fn approve_as_multi_create(s: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `279 + s * (1 ±0)`
        //  Estimated: `6815`
        // Minimum execution time: 36_421 nanoseconds.
        Weight::from_parts(39_635_679, 6815)
            // Standard Error: 1_998
            .saturating_add(Weight::from_parts(102_944, 0).saturating_mul(s.into()))
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    /// Storage: `Multisig::Multisigs` (r:1 w:1)
    /// Proof: `Multisig::Multisigs` (`max_values`: None, `max_size`: Some(3350), added: 5825, mode: `MaxEncodedLen`)
    /// The range of component `s` is `[2, 100]`.
    fn approve_as_multi_approve(s: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `252`
        //  Estimated: `6815`
        // Minimum execution time: 21_391 nanoseconds.
        Weight::from_parts(22_425_605, 6815)
            // Standard Error: 1_445
            .saturating_add(Weight::from_parts(101_371, 0).saturating_mul(s.into()))
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    /// Storage: `Multisig::Multisigs` (r:1 w:1)
    /// Proof: `Multisig::Multisigs` (`max_values`: None, `max_size`: Some(3350), added: 5825, mode: `MaxEncodedLen`)
    /// The range of component `s` is `[2, 100]`.
    fn cancel_as_multi(s: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `428 + s * (1 ±0)`
        //  Estimated: `6815`
        // Minimum execution time: 38_660 nanoseconds.
        Weight::from_parts(40_879_796, 6815)
            // Standard Error: 1_474
            .saturating_add(Weight::from_parts(87_791, 0).saturating_mul(s.into()))
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
}
