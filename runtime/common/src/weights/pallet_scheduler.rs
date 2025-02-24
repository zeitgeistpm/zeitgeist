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

//! Autogenerated weights for pallet_scheduler
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: `2024-12-06`, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
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
// --pallet=pallet_scheduler
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

/// Weight functions for pallet_scheduler (automatically generated)
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_scheduler::weights::WeightInfo for WeightInfo<T> {
    /// Storage: `Scheduler::IncompleteSince` (r:1 w:1)
    /// Proof: `Scheduler::IncompleteSince` (`max_values`: Some(1), `max_size`: Some(8), added: 503, mode: `MaxEncodedLen`)
    fn service_agendas_base() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `35`
        //  Estimated: `1493`
        // Minimum execution time: 7_670 nanoseconds.
        Weight::from_parts(8_601_000, 1493)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    /// Storage: `Scheduler::Agenda` (r:1 w:1)
    /// Proof: `Scheduler::Agenda` (`max_values`: None, `max_size`: Some(109074), added: 111549, mode: `MaxEncodedLen`)
    /// The range of component `s` is `[0, 512]`.
    fn service_agenda_base(s: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `85 + s * (181 ±0)`
        //  Estimated: `112539`
        // Minimum execution time: 7_280 nanoseconds.
        Weight::from_parts(10_877_615, 112539)
            // Standard Error: 461
            .saturating_add(Weight::from_parts(375_179, 0).saturating_mul(s.into()))
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    fn service_task_base() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 6_070 nanoseconds.
        Weight::from_parts(6_371_000, 0)
    }
    /// Storage: `Preimage::PreimageFor` (r:1 w:1)
    /// Proof: `Preimage::PreimageFor` (`max_values`: None, `max_size`: Some(4194344), added: 4196819, mode: `Measured`)
    /// Storage: `Preimage::StatusFor` (r:1 w:1)
    /// Proof: `Preimage::StatusFor` (`max_values`: None, `max_size`: Some(91), added: 2566, mode: `MaxEncodedLen`)
    /// The range of component `s` is `[128, 4194304]`.
    fn service_task_fetched(s: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `179 + s * (1 ±0)`
        //  Estimated: `3644 + s * (1 ±0)`
        // Minimum execution time: 24_341 nanoseconds.
        Weight::from_parts(24_800_000, 3644)
            // Standard Error: 2
            .saturating_add(Weight::from_parts(1_026, 0).saturating_mul(s.into()))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
            .saturating_add(Weight::from_parts(0, 1).saturating_mul(s.into()))
    }
    /// Storage: `Scheduler::Lookup` (r:0 w:1)
    /// Proof: `Scheduler::Lookup` (`max_values`: None, `max_size`: Some(52), added: 2527, mode: `MaxEncodedLen`)
    fn service_task_named() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 7_640 nanoseconds.
        Weight::from_parts(8_191_000, 0).saturating_add(T::DbWeight::get().writes(1))
    }
    fn service_task_periodic() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 6_050 nanoseconds.
        Weight::from_parts(6_480_000, 0)
    }
    fn execute_dispatch_signed() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 2_650 nanoseconds.
        Weight::from_parts(2_790_000, 0)
    }
    fn execute_dispatch_unsigned() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 2_570 nanoseconds.
        Weight::from_parts(2_790_000, 0)
    }
    /// Storage: `Scheduler::Agenda` (r:1 w:1)
    /// Proof: `Scheduler::Agenda` (`max_values`: None, `max_size`: Some(109074), added: 111549, mode: `MaxEncodedLen`)
    /// The range of component `s` is `[0, 511]`.
    fn schedule(s: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `85 + s * (181 ±0)`
        //  Estimated: `112539`
        // Minimum execution time: 14_830 nanoseconds.
        Weight::from_parts(21_341_301, 112539)
            // Standard Error: 1_609
            .saturating_add(Weight::from_parts(415_643, 0).saturating_mul(s.into()))
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    /// Storage: `Scheduler::Agenda` (r:1 w:1)
    /// Proof: `Scheduler::Agenda` (`max_values`: None, `max_size`: Some(109074), added: 111549, mode: `MaxEncodedLen`)
    /// Storage: `Scheduler::Lookup` (r:0 w:1)
    /// Proof: `Scheduler::Lookup` (`max_values`: None, `max_size`: Some(52), added: 2527, mode: `MaxEncodedLen`)
    /// The range of component `s` is `[1, 512]`.
    fn cancel(s: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `85 + s * (181 ±0)`
        //  Estimated: `112539`
        // Minimum execution time: 20_420 nanoseconds.
        Weight::from_parts(22_383_928, 112539)
            // Standard Error: 2_287
            .saturating_add(Weight::from_parts(630_724, 0).saturating_mul(s.into()))
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    /// Storage: `Scheduler::Lookup` (r:1 w:1)
    /// Proof: `Scheduler::Lookup` (`max_values`: None, `max_size`: Some(52), added: 2527, mode: `MaxEncodedLen`)
    /// Storage: `Scheduler::Agenda` (r:1 w:1)
    /// Proof: `Scheduler::Agenda` (`max_values`: None, `max_size`: Some(109074), added: 111549, mode: `MaxEncodedLen`)
    /// The range of component `s` is `[0, 511]`.
    fn schedule_named(s: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `601 + s * (182 ±0)`
        //  Estimated: `112539`
        // Minimum execution time: 18_110 nanoseconds.
        Weight::from_parts(25_735_298, 112539)
            // Standard Error: 1_565
            .saturating_add(Weight::from_parts(432_864, 0).saturating_mul(s.into()))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    /// Storage: `Scheduler::Lookup` (r:1 w:1)
    /// Proof: `Scheduler::Lookup` (`max_values`: None, `max_size`: Some(52), added: 2527, mode: `MaxEncodedLen`)
    /// Storage: `Scheduler::Agenda` (r:1 w:1)
    /// Proof: `Scheduler::Agenda` (`max_values`: None, `max_size`: Some(109074), added: 111549, mode: `MaxEncodedLen`)
    /// The range of component `s` is `[1, 512]`.
    fn cancel_named(s: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `717 + s * (181 ±0)`
        //  Estimated: `112539`
        // Minimum execution time: 21_871 nanoseconds.
        Weight::from_parts(24_964_890, 112539)
            // Standard Error: 1_607
            .saturating_add(Weight::from_parts(631_932, 0).saturating_mul(s.into()))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
}
