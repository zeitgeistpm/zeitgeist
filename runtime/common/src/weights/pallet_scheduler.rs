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
//! DATE: 2023-06-16, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

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
    // Storage: Scheduler IncompleteSince (r:1 w:1)
    fn service_agendas_base() -> Weight {
        Weight::from_ref_time(8_730_000)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    // Storage: Scheduler Agenda (r:1 w:1)
    fn service_agenda_base(s: u32) -> Weight {
        Weight::from_ref_time(16_148_157)
            // Standard Error: 15_790
            .saturating_add(Weight::from_ref_time(602_294).saturating_mul(s.into()))
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    fn service_task_base() -> Weight {
        Weight::from_ref_time(17_290_000)
    }
    // Storage: Preimage PreimageFor (r:1 w:1)
    // Storage: Preimage StatusFor (r:1 w:1)
    fn service_task_fetched(s: u32) -> Weight {
        Weight::from_ref_time(38_440_000)
            // Standard Error: 7
            .saturating_add(Weight::from_ref_time(2_139).saturating_mul(s.into()))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    // Storage: Scheduler Lookup (r:0 w:1)
    fn service_task_named() -> Weight {
        Weight::from_ref_time(19_870_000).saturating_add(T::DbWeight::get().writes(1))
    }
    fn service_task_periodic() -> Weight {
        Weight::from_ref_time(17_530_000)
    }
    fn execute_dispatch_signed() -> Weight {
        Weight::from_ref_time(8_010_000)
    }
    fn execute_dispatch_unsigned() -> Weight {
        Weight::from_ref_time(7_960_000)
    }
    // Storage: Scheduler Agenda (r:1 w:1)
    fn schedule(s: u32) -> Weight {
        Weight::from_ref_time(36_389_558)
            // Standard Error: 17_862
            .saturating_add(Weight::from_ref_time(712_217).saturating_mul(s.into()))
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    // Storage: Scheduler Agenda (r:1 w:1)
    // Storage: Scheduler Lookup (r:0 w:1)
    fn cancel(s: u32) -> Weight {
        Weight::from_ref_time(38_453_612)
            // Standard Error: 9_437
            .saturating_add(Weight::from_ref_time(540_810).saturating_mul(s.into()))
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    // Storage: Scheduler Lookup (r:1 w:1)
    // Storage: Scheduler Agenda (r:1 w:1)
    fn schedule_named(s: u32) -> Weight {
        Weight::from_ref_time(45_792_369)
            // Standard Error: 14_702
            .saturating_add(Weight::from_ref_time(614_083).saturating_mul(s.into()))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    // Storage: Scheduler Lookup (r:1 w:1)
    // Storage: Scheduler Agenda (r:1 w:1)
    fn cancel_named(s: u32) -> Weight {
        Weight::from_ref_time(40_577_357)
            // Standard Error: 13_678
            .saturating_add(Weight::from_ref_time(632_570).saturating_mul(s.into()))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
}
