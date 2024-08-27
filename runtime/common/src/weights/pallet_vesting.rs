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
//! DATE: `2024-08-12`, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `zeitgeist-benchmark`, CPU: `AMD EPYC 7601 32-Core Processor`
//! EXECUTION: ``, WASM-EXECUTION: `Compiled`, CHAIN: `Some("dev")`, DB CACHE: `1024`

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
    /// Storage: `Vesting::Vesting` (r:1 w:1)
    /// Proof: `Vesting::Vesting` (`max_values`: None, `max_size`: Some(1169), added: 3644, mode: `MaxEncodedLen`)
    /// Storage: `Balances::Locks` (r:1 w:1)
    /// Proof: `Balances::Locks` (`max_values`: None, `max_size`: Some(1299), added: 3774, mode: `MaxEncodedLen`)
    /// Storage: `Balances::Freezes` (r:1 w:0)
    /// Proof: `Balances::Freezes` (`max_values`: None, `max_size`: Some(65), added: 2540, mode: `MaxEncodedLen`)
    /// The range of component `l` is `[0, 49]`.
    /// The range of component `s` is `[1, 28]`.
    fn vest_locked(l: u32, s: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `343 + l * (25 ±0) + s * (40 ±0)`
        //  Estimated: `4764`
        // Minimum execution time: 38_841 nanoseconds.
        Weight::from_parts(39_177_842, 4764)
            // Standard Error: 3_062
            .saturating_add(Weight::from_parts(45_586, 0).saturating_mul(l.into()))
            // Standard Error: 5_449
            .saturating_add(Weight::from_parts(104_115, 0).saturating_mul(s.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    /// Storage: `Vesting::Vesting` (r:1 w:1)
    /// Proof: `Vesting::Vesting` (`max_values`: None, `max_size`: Some(1169), added: 3644, mode: `MaxEncodedLen`)
    /// Storage: `Balances::Locks` (r:1 w:1)
    /// Proof: `Balances::Locks` (`max_values`: None, `max_size`: Some(1299), added: 3774, mode: `MaxEncodedLen`)
    /// Storage: `Balances::Freezes` (r:1 w:0)
    /// Proof: `Balances::Freezes` (`max_values`: None, `max_size`: Some(65), added: 2540, mode: `MaxEncodedLen`)
    /// The range of component `l` is `[0, 49]`.
    /// The range of component `s` is `[1, 28]`.
    fn vest_unlocked(l: u32, s: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `343 + l * (25 ±0) + s * (40 ±0)`
        //  Estimated: `4764`
        // Minimum execution time: 43_011 nanoseconds.
        Weight::from_parts(42_994_675, 4764)
            // Standard Error: 5_463
            .saturating_add(Weight::from_parts(62_936, 0).saturating_mul(l.into()))
            // Standard Error: 9_721
            .saturating_add(Weight::from_parts(105_034, 0).saturating_mul(s.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    /// Storage: `Vesting::Vesting` (r:1 w:1)
    /// Proof: `Vesting::Vesting` (`max_values`: None, `max_size`: Some(1169), added: 3644, mode: `MaxEncodedLen`)
    /// Storage: `Balances::Locks` (r:1 w:1)
    /// Proof: `Balances::Locks` (`max_values`: None, `max_size`: Some(1299), added: 3774, mode: `MaxEncodedLen`)
    /// Storage: `Balances::Freezes` (r:1 w:0)
    /// Proof: `Balances::Freezes` (`max_values`: None, `max_size`: Some(65), added: 2540, mode: `MaxEncodedLen`)
    /// Storage: `System::Account` (r:1 w:1)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(132), added: 2607, mode: `MaxEncodedLen`)
    /// The range of component `l` is `[0, 49]`.
    /// The range of component `s` is `[1, 28]`.
    fn vest_other_locked(l: u32, s: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `450 + l * (25 ±0) + s * (40 ±0)`
        //  Estimated: `4764`
        // Minimum execution time: 42_491 nanoseconds.
        Weight::from_parts(43_248_191, 4764)
            // Standard Error: 3_258
            .saturating_add(Weight::from_parts(45_647, 0).saturating_mul(l.into()))
            // Standard Error: 5_797
            .saturating_add(Weight::from_parts(96_446, 0).saturating_mul(s.into()))
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    /// Storage: `Vesting::Vesting` (r:1 w:1)
    /// Proof: `Vesting::Vesting` (`max_values`: None, `max_size`: Some(1169), added: 3644, mode: `MaxEncodedLen`)
    /// Storage: `Balances::Locks` (r:1 w:1)
    /// Proof: `Balances::Locks` (`max_values`: None, `max_size`: Some(1299), added: 3774, mode: `MaxEncodedLen`)
    /// Storage: `Balances::Freezes` (r:1 w:0)
    /// Proof: `Balances::Freezes` (`max_values`: None, `max_size`: Some(65), added: 2540, mode: `MaxEncodedLen`)
    /// Storage: `System::Account` (r:1 w:1)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(132), added: 2607, mode: `MaxEncodedLen`)
    /// The range of component `l` is `[0, 49]`.
    /// The range of component `s` is `[1, 28]`.
    fn vest_other_unlocked(l: u32, s: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `450 + l * (25 ±0) + s * (40 ±0)`
        //  Estimated: `4764`
        // Minimum execution time: 46_221 nanoseconds.
        Weight::from_parts(47_350_351, 4764)
            // Standard Error: 3_939
            .saturating_add(Weight::from_parts(45_864, 0).saturating_mul(l.into()))
            // Standard Error: 7_008
            .saturating_add(Weight::from_parts(98_661, 0).saturating_mul(s.into()))
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    /// Storage: `Vesting::Vesting` (r:1 w:1)
    /// Proof: `Vesting::Vesting` (`max_values`: None, `max_size`: Some(1169), added: 3644, mode: `MaxEncodedLen`)
    /// Storage: `System::Account` (r:1 w:1)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(132), added: 2607, mode: `MaxEncodedLen`)
    /// Storage: `Balances::Locks` (r:1 w:1)
    /// Proof: `Balances::Locks` (`max_values`: None, `max_size`: Some(1299), added: 3774, mode: `MaxEncodedLen`)
    /// Storage: `Balances::Freezes` (r:1 w:0)
    /// Proof: `Balances::Freezes` (`max_values`: None, `max_size`: Some(65), added: 2540, mode: `MaxEncodedLen`)
    /// The range of component `l` is `[0, 49]`.
    /// The range of component `s` is `[0, 27]`.
    fn vested_transfer(l: u32, s: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `521 + l * (25 ±0) + s * (40 ±0)`
        //  Estimated: `4764`
        // Minimum execution time: 82_762 nanoseconds.
        Weight::from_parts(85_391_940, 4764)
            // Standard Error: 7_126
            .saturating_add(Weight::from_parts(59_265, 0).saturating_mul(l.into()))
            // Standard Error: 12_680
            .saturating_add(Weight::from_parts(189_674, 0).saturating_mul(s.into()))
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    /// Storage: `Vesting::Vesting` (r:1 w:1)
    /// Proof: `Vesting::Vesting` (`max_values`: None, `max_size`: Some(1169), added: 3644, mode: `MaxEncodedLen`)
    /// Storage: `System::Account` (r:2 w:2)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(132), added: 2607, mode: `MaxEncodedLen`)
    /// Storage: `Balances::Locks` (r:1 w:1)
    /// Proof: `Balances::Locks` (`max_values`: None, `max_size`: Some(1299), added: 3774, mode: `MaxEncodedLen`)
    /// Storage: `Balances::Freezes` (r:1 w:0)
    /// Proof: `Balances::Freezes` (`max_values`: None, `max_size`: Some(65), added: 2540, mode: `MaxEncodedLen`)
    /// The range of component `l` is `[0, 49]`.
    /// The range of component `s` is `[0, 27]`.
    fn force_vested_transfer(l: u32, s: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `628 + l * (25 ±0) + s * (40 ±0)`
        //  Estimated: `6204`
        // Minimum execution time: 87_132 nanoseconds.
        Weight::from_parts(86_414_750, 6204)
            // Standard Error: 16_931
            .saturating_add(Weight::from_parts(119_327, 0).saturating_mul(l.into()))
            // Standard Error: 30_124
            .saturating_add(Weight::from_parts(266_388, 0).saturating_mul(s.into()))
            .saturating_add(T::DbWeight::get().reads(5))
            .saturating_add(T::DbWeight::get().writes(4))
    }
    /// Storage: `Vesting::Vesting` (r:1 w:1)
    /// Proof: `Vesting::Vesting` (`max_values`: None, `max_size`: Some(1169), added: 3644, mode: `MaxEncodedLen`)
    /// Storage: `Balances::Locks` (r:1 w:1)
    /// Proof: `Balances::Locks` (`max_values`: None, `max_size`: Some(1299), added: 3774, mode: `MaxEncodedLen`)
    /// Storage: `Balances::Freezes` (r:1 w:0)
    /// Proof: `Balances::Freezes` (`max_values`: None, `max_size`: Some(65), added: 2540, mode: `MaxEncodedLen`)
    /// Storage: `System::Account` (r:1 w:1)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(132), added: 2607, mode: `MaxEncodedLen`)
    /// The range of component `l` is `[0, 49]`.
    /// The range of component `s` is `[2, 28]`.
    fn not_unlocking_merge_schedules(l: u32, s: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `448 + l * (25 ±0) + s * (40 ±0)`
        //  Estimated: `4764`
        // Minimum execution time: 42_830 nanoseconds.
        Weight::from_parts(43_773_712, 4764)
            // Standard Error: 2_903
            .saturating_add(Weight::from_parts(40_756, 0).saturating_mul(l.into()))
            // Standard Error: 5_361
            .saturating_add(Weight::from_parts(116_495, 0).saturating_mul(s.into()))
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    /// Storage: `Vesting::Vesting` (r:1 w:1)
    /// Proof: `Vesting::Vesting` (`max_values`: None, `max_size`: Some(1169), added: 3644, mode: `MaxEncodedLen`)
    /// Storage: `Balances::Locks` (r:1 w:1)
    /// Proof: `Balances::Locks` (`max_values`: None, `max_size`: Some(1299), added: 3774, mode: `MaxEncodedLen`)
    /// Storage: `Balances::Freezes` (r:1 w:0)
    /// Proof: `Balances::Freezes` (`max_values`: None, `max_size`: Some(65), added: 2540, mode: `MaxEncodedLen`)
    /// Storage: `System::Account` (r:1 w:1)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(132), added: 2607, mode: `MaxEncodedLen`)
    /// The range of component `l` is `[0, 49]`.
    /// The range of component `s` is `[2, 28]`.
    fn unlocking_merge_schedules(l: u32, s: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `448 + l * (25 ±0) + s * (40 ±0)`
        //  Estimated: `4764`
        // Minimum execution time: 47_501 nanoseconds.
        Weight::from_parts(48_177_622, 4764)
            // Standard Error: 4_244
            .saturating_add(Weight::from_parts(48_727, 0).saturating_mul(l.into()))
            // Standard Error: 7_839
            .saturating_add(Weight::from_parts(121_884, 0).saturating_mul(s.into()))
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(3))
    }
}
