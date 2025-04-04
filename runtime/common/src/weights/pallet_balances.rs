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

//! Autogenerated weights for pallet_balances
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
// --pallet=pallet_balances
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

/// Weight functions for pallet_balances (automatically generated)
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_balances::weights::WeightInfo for WeightInfo<T> {
    /// Storage: `System::Account` (r:2 w:2)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(132), added: 2607, mode: `MaxEncodedLen`)
    fn transfer_allow_death() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `144`
        //  Estimated: `6204`
        // Minimum execution time: 79_931 nanoseconds.
        Weight::from_parts(82_021_000, 6204)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    /// Storage: `System::Account` (r:1 w:1)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(132), added: 2607, mode: `MaxEncodedLen`)
    fn transfer_keep_alive() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `3597`
        // Minimum execution time: 48_281 nanoseconds.
        Weight::from_parts(49_831_000, 3597)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    /// Storage: `System::Account` (r:1 w:1)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(132), added: 2607, mode: `MaxEncodedLen`)
    fn force_set_balance_creating() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `178`
        //  Estimated: `3597`
        // Minimum execution time: 20_000 nanoseconds.
        Weight::from_parts(20_650_000, 3597)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    /// Storage: `System::Account` (r:1 w:1)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(132), added: 2607, mode: `MaxEncodedLen`)
    fn force_set_balance_killing() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `178`
        //  Estimated: `3597`
        // Minimum execution time: 28_241 nanoseconds.
        Weight::from_parts(29_651_000, 3597)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    /// Storage: `System::Account` (r:3 w:3)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(132), added: 2607, mode: `MaxEncodedLen`)
    fn force_transfer() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `251`
        //  Estimated: `8811`
        // Minimum execution time: 82_702 nanoseconds.
        Weight::from_parts(84_292_000, 8811)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    /// Storage: `System::Account` (r:1 w:1)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(132), added: 2607, mode: `MaxEncodedLen`)
    fn transfer_all() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `3597`
        // Minimum execution time: 60_972 nanoseconds.
        Weight::from_parts(62_391_000, 3597)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    /// Storage: `System::Account` (r:1 w:1)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(132), added: 2607, mode: `MaxEncodedLen`)
    fn force_unreserve() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `178`
        //  Estimated: `3597`
        // Minimum execution time: 23_211 nanoseconds.
        Weight::from_parts(24_000_000, 3597)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    /// Storage: `System::Account` (r:999 w:999)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(132), added: 2607, mode: `MaxEncodedLen`)
    /// The range of component `u` is `[1, 1000]`.
    fn upgrade_accounts(u: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0 + u * (139 ±0)`
        //  Estimated: `990 + u * (2607 ±0)`
        // Minimum execution time: 22_612 nanoseconds.
        Weight::from_parts(23_020_000, 990)
            // Standard Error: 20_612
            .saturating_add(Weight::from_parts(18_203_443, 0).saturating_mul(u.into()))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(u.into())))
            .saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(u.into())))
            .saturating_add(Weight::from_parts(0, 2607).saturating_mul(u.into()))
    }
}
