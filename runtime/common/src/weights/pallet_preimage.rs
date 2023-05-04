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
//! DATE: 2023-02-08, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

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
    // Storage: Preimage StatusFor (r:1 w:1)
    // Storage: Preimage PreimageFor (r:0 w:1)
    fn note_preimage(s: u32) -> Weight {
        Weight::from_ref_time(21_280_000)
            // Standard Error: 1
            .saturating_add(Weight::from_ref_time(1_231).saturating_mul(s.into()))
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    // Storage: Preimage StatusFor (r:1 w:1)
    // Storage: Preimage PreimageFor (r:0 w:1)
    fn note_requested_preimage(s: u32) -> Weight {
        Weight::from_ref_time(14_404_000)
            // Standard Error: 1
            .saturating_add(Weight::from_ref_time(1_233).saturating_mul(s.into()))
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    // Storage: Preimage StatusFor (r:1 w:1)
    // Storage: Preimage PreimageFor (r:0 w:1)
    fn note_no_deposit_preimage(s: u32) -> Weight {
        Weight::from_ref_time(13_424_000)
            // Standard Error: 1
            .saturating_add(Weight::from_ref_time(1_231).saturating_mul(s.into()))
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    // Storage: Preimage StatusFor (r:1 w:1)
    // Storage: Preimage PreimageFor (r:0 w:1)
    fn unnote_preimage() -> Weight {
        Weight::from_ref_time(28_640_000)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    // Storage: Preimage StatusFor (r:1 w:1)
    // Storage: Preimage PreimageFor (r:0 w:1)
    fn unnote_no_deposit_preimage() -> Weight {
        Weight::from_ref_time(16_748_000)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    // Storage: Preimage StatusFor (r:1 w:1)
    fn request_preimage() -> Weight {
        Weight::from_ref_time(15_219_000)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    // Storage: Preimage StatusFor (r:1 w:1)
    fn request_no_deposit_preimage() -> Weight {
        Weight::from_ref_time(6_886_000)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    // Storage: Preimage StatusFor (r:1 w:1)
    fn request_unnoted_preimage() -> Weight {
        Weight::from_ref_time(13_170_000)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    // Storage: Preimage StatusFor (r:1 w:1)
    fn request_requested_preimage() -> Weight {
        Weight::from_ref_time(6_794_000)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    // Storage: Preimage StatusFor (r:1 w:1)
    // Storage: Preimage PreimageFor (r:0 w:1)
    fn unrequest_preimage() -> Weight {
        Weight::from_ref_time(16_478_000)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    // Storage: Preimage StatusFor (r:1 w:1)
    fn unrequest_unnoted_preimage() -> Weight {
        Weight::from_ref_time(6_985_000)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    // Storage: Preimage StatusFor (r:1 w:1)
    fn unrequest_multi_referenced_preimage() -> Weight {
        Weight::from_ref_time(6_886_000)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
}
