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

//! Autogenerated weights for pallet_balances
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: `2023-09-19`, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
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
    /// Storage: System Account (r:2 w:2)
    /// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
    fn transfer() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1585`
        //  Estimated: `5214`
        // Minimum execution time: 110_920 nanoseconds.
        Weight::from_parts(112_730_000, 5214)
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    /// Storage: System Account (r:1 w:1)
    /// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
    fn transfer_keep_alive() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1409`
        //  Estimated: `2607`
        // Minimum execution time: 61_750 nanoseconds.
        Weight::from_parts(74_220_000, 2607)
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
    /// Storage: System Account (r:1 w:1)
    /// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
    fn set_balance_creating() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1559`
        //  Estimated: `2607`
        // Minimum execution time: 53_450 nanoseconds.
        Weight::from_parts(59_371_000, 2607)
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
    /// Storage: System Account (r:1 w:1)
    /// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
    fn set_balance_killing() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1559`
        //  Estimated: `2607`
        // Minimum execution time: 65_040 nanoseconds.
        Weight::from_parts(65_980_000, 2607)
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
    /// Storage: System Account (r:3 w:3)
    /// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
    fn force_transfer() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1581`
        //  Estimated: `7821`
        // Minimum execution time: 110_150 nanoseconds.
        Weight::from_parts(111_860_000, 7821)
            .saturating_add(T::DbWeight::get().reads(3_u64))
            .saturating_add(T::DbWeight::get().writes(3_u64))
    }
    /// Storage: System Account (r:1 w:1)
    /// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
    fn transfer_all() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1409`
        //  Estimated: `2607`
        // Minimum execution time: 84_951 nanoseconds.
        Weight::from_parts(85_671_000, 2607)
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
    /// Storage: System Account (r:1 w:1)
    /// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
    fn force_unreserve() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1443`
        //  Estimated: `2607`
        // Minimum execution time: 42_121 nanoseconds.
        Weight::from_parts(51_680_000, 2607)
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
}
