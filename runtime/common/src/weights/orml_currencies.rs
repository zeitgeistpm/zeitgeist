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

//! Autogenerated weights for orml_currencies
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
// --pallet=orml_currencies
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --template=./misc/orml_weight_template.hbs
// --header=./HEADER_GPL3
// --output=./runtime/common/src/weights/

#![allow(unused_parens)]
#![allow(unused_imports)]

use core::marker::PhantomData;
use frame_support::{traits::Get, weights::Weight};

/// Weight functions for orml_currencies (automatically generated)
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> orml_currencies::WeightInfo for WeightInfo<T> {
    /// Storage: `Tokens::Accounts` (r:2 w:2)
    /// Proof: `Tokens::Accounts` (`max_values`: None, `max_size`: Some(123), added: 2598, mode: `MaxEncodedLen`)
    /// Storage: `System::Account` (r:1 w:1)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(132), added: 2607, mode: `MaxEncodedLen`)
    fn transfer_non_native_currency() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1690`
        //  Estimated: `6186`
        // Minimum execution time: 61_662 nanoseconds.
        Weight::from_parts(63_092_000, 6186)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    /// Storage: `System::Account` (r:1 w:1)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(132), added: 2607, mode: `MaxEncodedLen`)
    fn transfer_native_currency() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1545`
        //  Estimated: `3597`
        // Minimum execution time: 82_882 nanoseconds.
        Weight::from_parts(84_542_000, 3597)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    /// Storage: `Tokens::Accounts` (r:1 w:1)
    /// Proof: `Tokens::Accounts` (`max_values`: None, `max_size`: Some(123), added: 2598, mode: `MaxEncodedLen`)
    /// Storage: `Tokens::TotalIssuance` (r:1 w:1)
    /// Proof: `Tokens::TotalIssuance` (`max_values`: None, `max_size`: Some(43), added: 2518, mode: `MaxEncodedLen`)
    /// Storage: `System::Account` (r:1 w:1)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(132), added: 2607, mode: `MaxEncodedLen`)
    fn update_balance_non_native_currency() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1361`
        //  Estimated: `3597`
        // Minimum execution time: 41_682 nanoseconds.
        Weight::from_parts(43_061_000, 3597)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    /// Storage: `System::Account` (r:1 w:1)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(132), added: 2607, mode: `MaxEncodedLen`)
    fn update_balance_native_currency_creating() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1401`
        //  Estimated: `3597`
        // Minimum execution time: 44_301 nanoseconds.
        Weight::from_parts(45_441_000, 3597)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    /// Storage: `System::Account` (r:1 w:1)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(132), added: 2607, mode: `MaxEncodedLen`)
    fn update_balance_native_currency_killing() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1493`
        //  Estimated: `3597`
        // Minimum execution time: 45_101 nanoseconds.
        Weight::from_parts(46_071_000, 3597)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
}
