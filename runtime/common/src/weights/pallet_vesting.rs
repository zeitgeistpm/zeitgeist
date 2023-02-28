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

//! Autogenerated weights for pallet_vesting
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-01-20, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/production/zeitgeist
// benchmark
// pallet
// --chain=dev
// --steps=50
// --repeat=20
// --pallet=pallet_vesting
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

/// Weight functions for pallet_vesting (automatically generated)
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_vesting::weights::WeightInfo for WeightInfo<T> {
    // Storage: Vesting Vesting (r:1 w:1)
    // Storage: Balances Locks (r:1 w:1)
    fn vest_locked(_l: u32, _s: u32) -> Weight {
        Weight::from_ref_time(74_193_000)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    // Storage: Vesting Vesting (r:1 w:1)
    // Storage: Balances Locks (r:1 w:1)
    fn vest_unlocked(l: u32, s: u32) -> Weight {
        Weight::from_ref_time(59_403_000)
            // Standard Error: 14_000
            .saturating_add(Weight::from_ref_time(142_000).saturating_mul(l.into()))
            // Standard Error: 25_000
            .saturating_add(Weight::from_ref_time(76_000).saturating_mul(s.into()))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    // Storage: Vesting Vesting (r:1 w:1)
    // Storage: Balances Locks (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    fn vest_other_locked(l: u32, _s: u32) -> Weight {
        Weight::from_ref_time(69_282_000)
            // Standard Error: 17_000
            .saturating_add(Weight::from_ref_time(33_000).saturating_mul(l.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    // Storage: Vesting Vesting (r:1 w:1)
    // Storage: Balances Locks (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    fn vest_other_unlocked(l: u32, _s: u32) -> Weight {
        Weight::from_ref_time(64_138_000)
            // Standard Error: 10_000
            .saturating_add(Weight::from_ref_time(101_000).saturating_mul(l.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    // Storage: Vesting Vesting (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    // Storage: Balances Locks (r:1 w:1)
    fn vested_transfer(l: u32, s: u32) -> Weight {
        Weight::from_ref_time(89_063_000)
            // Standard Error: 24_000
            .saturating_add(Weight::from_ref_time(74_000).saturating_mul(l.into()))
            // Standard Error: 43_000
            .saturating_add(Weight::from_ref_time(240_000).saturating_mul(s.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    // Storage: Vesting Vesting (r:1 w:1)
    // Storage: System Account (r:2 w:2)
    // Storage: Balances Locks (r:1 w:1)
    fn force_vested_transfer(_l: u32, s: u32) -> Weight {
        Weight::from_ref_time(95_489_000)
            // Standard Error: 61_000
            .saturating_add(Weight::from_ref_time(44_000).saturating_mul(s.into()))
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(4))
    }
    // Storage: Vesting Vesting (r:1 w:1)
    // Storage: Balances Locks (r:1 w:1)
    fn not_unlocking_merge_schedules(l: u32, _s: u32) -> Weight {
        Weight::from_ref_time(67_438_000)
            // Standard Error: 14_000
            .saturating_add(Weight::from_ref_time(158_000).saturating_mul(l.into()))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    // Storage: Vesting Vesting (r:1 w:1)
    // Storage: Balances Locks (r:1 w:1)
    fn unlocking_merge_schedules(l: u32, s: u32) -> Weight {
        Weight::from_ref_time(67_798_000)
            // Standard Error: 22_000
            .saturating_add(Weight::from_ref_time(48_000).saturating_mul(l.into()))
            // Standard Error: 41_000
            .saturating_add(Weight::from_ref_time(115_000).saturating_mul(s.into()))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
}
