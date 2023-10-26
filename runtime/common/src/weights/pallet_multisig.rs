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

//! Autogenerated weights for pallet_multisig
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: `2023-10-25`, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
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
        // Minimum execution time: 17_090 nanoseconds.
        Weight::from_parts(22_547_170, 0)
            // Standard Error: 48
            .saturating_add(Weight::from_parts(769, 0).saturating_mul(z.into()))
    }
    /// Storage: Multisig Multisigs (r:1 w:1)
    /// Proof: Multisig Multisigs (max_values: None, max_size: Some(3350), added: 5825, mode: MaxEncodedLen)
    /// The range of component `s` is `[2, 100]`.
    /// The range of component `z` is `[0, 10000]`.
    fn as_multi_create(s: u32, z: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `339 + s * (1 ±0)`
        //  Estimated: `5825`
        // Minimum execution time: 50_160 nanoseconds.
        Weight::from_parts(52_808_971, 5825)
            // Standard Error: 10_853
            .saturating_add(Weight::from_parts(48_861, 0).saturating_mul(s.into()))
            // Standard Error: 106
            .saturating_add(Weight::from_parts(2_424, 0).saturating_mul(z.into()))
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    /// Storage: Multisig Multisigs (r:1 w:1)
    /// Proof: Multisig Multisigs (max_values: None, max_size: Some(3350), added: 5825, mode: MaxEncodedLen)
    /// The range of component `s` is `[3, 100]`.
    /// The range of component `z` is `[0, 10000]`.
    fn as_multi_approve(s: u32, z: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `283`
        //  Estimated: `5825`
        // Minimum execution time: 36_181 nanoseconds.
        Weight::from_parts(39_999_128, 5825)
            // Standard Error: 8_544
            .saturating_add(Weight::from_parts(15_566, 0).saturating_mul(s.into()))
            // Standard Error: 83
            .saturating_add(Weight::from_parts(2_593, 0).saturating_mul(z.into()))
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    /// Storage: Multisig Multisigs (r:1 w:1)
    /// Proof: Multisig Multisigs (max_values: None, max_size: Some(3350), added: 5825, mode: MaxEncodedLen)
    /// Storage: System Account (r:1 w:1)
    /// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
    /// The range of component `s` is `[2, 100]`.
    /// The range of component `z` is `[0, 10000]`.
    fn as_multi_complete(s: u32, z: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `425 + s * (33 ±0)`
        //  Estimated: `8432`
        // Minimum execution time: 57_120 nanoseconds.
        Weight::from_parts(43_350_397, 8432)
            // Standard Error: 15_458
            .saturating_add(Weight::from_parts(198_127, 0).saturating_mul(s.into()))
            // Standard Error: 151
            .saturating_add(Weight::from_parts(3_362, 0).saturating_mul(z.into()))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    /// Storage: Multisig Multisigs (r:1 w:1)
    /// Proof: Multisig Multisigs (max_values: None, max_size: Some(3350), added: 5825, mode: MaxEncodedLen)
    /// The range of component `s` is `[2, 100]`.
    fn approve_as_multi_create(s: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `343 + s * (1 ±0)`
        //  Estimated: `5825`
        // Minimum execution time: 38_630 nanoseconds.
        Weight::from_parts(45_470_511, 5825)
            // Standard Error: 13_700
            .saturating_add(Weight::from_parts(97_958, 0).saturating_mul(s.into()))
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    /// Storage: Multisig Multisigs (r:1 w:1)
    /// Proof: Multisig Multisigs (max_values: None, max_size: Some(3350), added: 5825, mode: MaxEncodedLen)
    /// The range of component `s` is `[2, 100]`.
    fn approve_as_multi_approve(s: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `283`
        //  Estimated: `5825`
        // Minimum execution time: 25_900 nanoseconds.
        Weight::from_parts(32_176_176, 5825)
            // Standard Error: 5_037
            .saturating_add(Weight::from_parts(66_127, 0).saturating_mul(s.into()))
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    /// Storage: Multisig Multisigs (r:1 w:1)
    /// Proof: Multisig Multisigs (max_values: None, max_size: Some(3350), added: 5825, mode: MaxEncodedLen)
    /// The range of component `s` is `[2, 100]`.
    fn cancel_as_multi(_s: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `491 + s * (1 ±0)`
        //  Estimated: `5825`
        // Minimum execution time: 41_270 nanoseconds.
        Weight::from_parts(56_230_114, 5825)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
}
