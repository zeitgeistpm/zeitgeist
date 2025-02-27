// Copyright 2022-2025 Forecasting Technologies LTD.
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

//! Autogenerated weights for cumulus_pallet_xcmp_queue
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: `2025-02-27`, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
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
// --pallet=cumulus_pallet_xcmp_queue
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

/// Weight functions for cumulus_pallet_xcmp_queue (automatically generated)
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> cumulus_pallet_xcmp_queue::weights::WeightInfo for WeightInfo<T> {
    /// Storage: `XcmpQueue::QueueConfig` (r:1 w:1)
    /// Proof: `XcmpQueue::QueueConfig` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    fn set_config_with_u32() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `142`
        //  Estimated: `1627`
        // Minimum execution time: 6_091 nanoseconds.
        Weight::from_parts(6_390_000, 1627)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    /// Storage: `XcmpQueue::QueueConfig` (r:1 w:1)
    /// Proof: `XcmpQueue::QueueConfig` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    fn set_config_with_weight() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `142`
        //  Estimated: `1627`
        // Minimum execution time: 6_170 nanoseconds.
        Weight::from_parts(6_571_000, 1627)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
}
