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

//! Autogenerated weights for pallet_vesting
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: `2024-01-15`, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
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
// --pallet=pallet_vesting
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

/// Weight functions for pallet_vesting (automatically generated)
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_vesting::weights::WeightInfo for WeightInfo<T> {
    /// Storage: Vesting Vesting (r:1 w:1)
    /// Proof: Vesting Vesting (max_values: None, max_size: Some(1169), added: 3644, mode: MaxEncodedLen)
    /// Storage: Balances Locks (r:1 w:1)
    /// Proof: Balances Locks (max_values: None, max_size: Some(1299), added: 3774, mode: MaxEncodedLen)
    /// The range of component `l` is `[0, 49]`.
    /// The range of component `s` is `[1, 28]`.
    fn vest_locked(l: u32, s: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `405 + l * (25 ±0) + s * (40 ±0)`
        //  Estimated: `7418`
        // Minimum execution time: 38_830 nanoseconds.
        Weight::from_parts(40_243_387, 7418)
            // Standard Error: 13_742
            .saturating_add(Weight::from_parts(93_058, 0).saturating_mul(l.into()))
            // Standard Error: 24_449
            .saturating_add(Weight::from_parts(163_917, 0).saturating_mul(s.into()))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    /// Storage: Vesting Vesting (r:1 w:1)
    /// Proof: Vesting Vesting (max_values: None, max_size: Some(1169), added: 3644, mode: MaxEncodedLen)
    /// Storage: Balances Locks (r:1 w:1)
    /// Proof: Balances Locks (max_values: None, max_size: Some(1299), added: 3774, mode: MaxEncodedLen)
    /// The range of component `l` is `[0, 49]`.
    /// The range of component `s` is `[1, 28]`.
    fn vest_unlocked(l: u32, s: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `405 + l * (25 ±0) + s * (40 ±0)`
        //  Estimated: `7418`
        // Minimum execution time: 38_450 nanoseconds.
        Weight::from_parts(43_347_252, 7418)
            // Standard Error: 9_766
            .saturating_add(Weight::from_parts(48_570, 0).saturating_mul(l.into()))
            // Standard Error: 17_375
            .saturating_add(Weight::from_parts(17_934, 0).saturating_mul(s.into()))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    /// Storage: Vesting Vesting (r:1 w:1)
    /// Proof: Vesting Vesting (max_values: None, max_size: Some(1169), added: 3644, mode: MaxEncodedLen)
    /// Storage: Balances Locks (r:1 w:1)
    /// Proof: Balances Locks (max_values: None, max_size: Some(1299), added: 3774, mode: MaxEncodedLen)
    /// Storage: System Account (r:1 w:1)
    /// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
    /// The range of component `l` is `[0, 49]`.
    /// The range of component `s` is `[1, 28]`.
    fn vest_other_locked(l: u32, s: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `544 + l * (25 ±0) + s * (40 ±0)`
        //  Estimated: `10025`
        // Minimum execution time: 43_880 nanoseconds.
        Weight::from_parts(46_021_890, 10025)
            // Standard Error: 12_235
            .saturating_add(Weight::from_parts(93_769, 0).saturating_mul(l.into()))
            // Standard Error: 21_768
            .saturating_add(Weight::from_parts(115_349, 0).saturating_mul(s.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    /// Storage: Vesting Vesting (r:1 w:1)
    /// Proof: Vesting Vesting (max_values: None, max_size: Some(1169), added: 3644, mode: MaxEncodedLen)
    /// Storage: Balances Locks (r:1 w:1)
    /// Proof: Balances Locks (max_values: None, max_size: Some(1299), added: 3774, mode: MaxEncodedLen)
    /// Storage: System Account (r:1 w:1)
    /// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
    /// The range of component `l` is `[0, 49]`.
    /// The range of component `s` is `[1, 28]`.
    fn vest_other_unlocked(l: u32, s: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `544 + l * (25 ±0) + s * (40 ±0)`
        //  Estimated: `10025`
        // Minimum execution time: 43_890 nanoseconds.
        Weight::from_parts(45_171_083, 10025)
            // Standard Error: 14_381
            .saturating_add(Weight::from_parts(100_199, 0).saturating_mul(l.into()))
            // Standard Error: 25_587
            .saturating_add(Weight::from_parts(158_229, 0).saturating_mul(s.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    /// Storage: Vesting Vesting (r:1 w:1)
    /// Proof: Vesting Vesting (max_values: None, max_size: Some(1169), added: 3644, mode: MaxEncodedLen)
    /// Storage: System Account (r:1 w:1)
    /// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
    /// Storage: Balances Locks (r:1 w:1)
    /// Proof: Balances Locks (max_values: None, max_size: Some(1299), added: 3774, mode: MaxEncodedLen)
    /// The range of component `l` is `[0, 49]`.
    /// The range of component `s` is `[0, 27]`.
    fn vested_transfer(l: u32, _s: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `615 + l * (25 ±0) + s * (40 ±0)`
        //  Estimated: `10025`
        // Minimum execution time: 62_820 nanoseconds.
        Weight::from_parts(74_828_952, 10025)
            // Standard Error: 20_587
            .saturating_add(Weight::from_parts(27_325, 0).saturating_mul(l.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    /// Storage: Vesting Vesting (r:1 w:1)
    /// Proof: Vesting Vesting (max_values: None, max_size: Some(1169), added: 3644, mode: MaxEncodedLen)
    /// Storage: System Account (r:2 w:2)
    /// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
    /// Storage: Balances Locks (r:1 w:1)
    /// Proof: Balances Locks (max_values: None, max_size: Some(1299), added: 3774, mode: MaxEncodedLen)
    /// The range of component `l` is `[0, 49]`.
    /// The range of component `s` is `[0, 27]`.
    fn force_vested_transfer(l: u32, _s: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `754 + l * (25 ±0) + s * (40 ±0)`
        //  Estimated: `12632`
        // Minimum execution time: 66_680 nanoseconds.
        Weight::from_parts(78_411_765, 12632)
            // Standard Error: 16_120
            .saturating_add(Weight::from_parts(6_898, 0).saturating_mul(l.into()))
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(4))
    }
    /// Storage: Vesting Vesting (r:1 w:1)
    /// Proof: Vesting Vesting (max_values: None, max_size: Some(1169), added: 3644, mode: MaxEncodedLen)
    /// Storage: Balances Locks (r:1 w:1)
    /// Proof: Balances Locks (max_values: None, max_size: Some(1299), added: 3774, mode: MaxEncodedLen)
    /// Storage: System Account (r:1 w:1)
    /// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
    /// The range of component `l` is `[0, 49]`.
    /// The range of component `s` is `[2, 28]`.
    fn not_unlocking_merge_schedules(_l: u32, _s: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `542 + l * (25 ±0) + s * (40 ±0)`
        //  Estimated: `10025`
        // Minimum execution time: 45_190 nanoseconds.
        Weight::from_parts(54_486_163, 10025)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    /// Storage: Vesting Vesting (r:1 w:1)
    /// Proof: Vesting Vesting (max_values: None, max_size: Some(1169), added: 3644, mode: MaxEncodedLen)
    /// Storage: Balances Locks (r:1 w:1)
    /// Proof: Balances Locks (max_values: None, max_size: Some(1299), added: 3774, mode: MaxEncodedLen)
    /// Storage: System Account (r:1 w:1)
    /// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
    /// The range of component `l` is `[0, 49]`.
    /// The range of component `s` is `[2, 28]`.
    fn unlocking_merge_schedules(l: u32, s: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `542 + l * (25 ±0) + s * (40 ±0)`
        //  Estimated: `10025`
        // Minimum execution time: 45_100 nanoseconds.
        Weight::from_parts(46_158_877, 10025)
            // Standard Error: 14_211
            .saturating_add(Weight::from_parts(105_475, 0).saturating_mul(l.into()))
            // Standard Error: 26_245
            .saturating_add(Weight::from_parts(168_563, 0).saturating_mul(s.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }
}
