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

//! Autogenerated weights for pallet_democracy
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: `2024-04-15`, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
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
// --pallet=pallet_democracy
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

/// Weight functions for pallet_democracy (automatically generated)
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_democracy::weights::WeightInfo for WeightInfo<T> {
    /// Storage: Democracy PublicPropCount (r:1 w:1)
    /// Proof: Democracy PublicPropCount (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
    /// Storage: Democracy PublicProps (r:1 w:1)
    /// Proof: Democracy PublicProps (max_values: Some(1), max_size: Some(16702), added: 17197, mode: MaxEncodedLen)
    /// Storage: Democracy Blacklist (r:1 w:0)
    /// Proof: Democracy Blacklist (max_values: None, max_size: Some(3238), added: 5713, mode: MaxEncodedLen)
    /// Storage: Democracy DepositOf (r:0 w:1)
    /// Proof: Democracy DepositOf (max_values: None, max_size: Some(3230), added: 5705, mode: MaxEncodedLen)
    fn propose() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `4801`
        //  Estimated: `18187`
        // Minimum execution time: 49_339_000 picoseconds.
        Weight::from_parts(50_942_000, 18187)
            .saturating_add(T::DbWeight::get().reads(3_u64))
            .saturating_add(T::DbWeight::get().writes(3_u64))
    }
    /// Storage: Democracy DepositOf (r:1 w:1)
    /// Proof: Democracy DepositOf (max_values: None, max_size: Some(3230), added: 5705, mode: MaxEncodedLen)
    fn second() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `3556`
        //  Estimated: `6695`
        // Minimum execution time: 43_291_000 picoseconds.
        Weight::from_parts(44_856_000, 6695)
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
    /// Storage: Democracy ReferendumInfoOf (r:1 w:1)
    /// Proof: Democracy ReferendumInfoOf (max_values: None, max_size: Some(201), added: 2676, mode: MaxEncodedLen)
    /// Storage: Democracy VotingOf (r:1 w:1)
    /// Proof: Democracy VotingOf (max_values: None, max_size: Some(3795), added: 6270, mode: MaxEncodedLen)
    /// Storage: Balances Locks (r:1 w:1)
    /// Proof: Balances Locks (max_values: None, max_size: Some(1299), added: 3774, mode: MaxEncodedLen)
    /// Storage: Balances Freezes (r:1 w:0)
    /// Proof: Balances Freezes (max_values: None, max_size: Some(49), added: 2524, mode: MaxEncodedLen)
    fn vote_new() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `3470`
        //  Estimated: `7260`
        // Minimum execution time: 61_890_000 picoseconds.
        Weight::from_parts(63_626_000, 7260)
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().writes(3_u64))
    }
    /// Storage: Democracy ReferendumInfoOf (r:1 w:1)
    /// Proof: Democracy ReferendumInfoOf (max_values: None, max_size: Some(201), added: 2676, mode: MaxEncodedLen)
    /// Storage: Democracy VotingOf (r:1 w:1)
    /// Proof: Democracy VotingOf (max_values: None, max_size: Some(3795), added: 6270, mode: MaxEncodedLen)
    /// Storage: Balances Locks (r:1 w:1)
    /// Proof: Balances Locks (max_values: None, max_size: Some(1299), added: 3774, mode: MaxEncodedLen)
    /// Storage: Balances Freezes (r:1 w:0)
    /// Proof: Balances Freezes (max_values: None, max_size: Some(49), added: 2524, mode: MaxEncodedLen)
    fn vote_existing() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `3492`
        //  Estimated: `7260`
        // Minimum execution time: 67_802_000 picoseconds.
        Weight::from_parts(69_132_000, 7260)
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().writes(3_u64))
    }
    /// Storage: Democracy ReferendumInfoOf (r:1 w:1)
    /// Proof: Democracy ReferendumInfoOf (max_values: None, max_size: Some(201), added: 2676, mode: MaxEncodedLen)
    /// Storage: Democracy Cancellations (r:1 w:1)
    /// Proof: Democracy Cancellations (max_values: None, max_size: Some(33), added: 2508, mode: MaxEncodedLen)
    /// Storage: Democracy MetadataOf (r:1 w:1)
    /// Proof: Democracy MetadataOf (max_values: None, max_size: Some(53), added: 2528, mode: MaxEncodedLen)
    fn emergency_cancel() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `366`
        //  Estimated: `3666`
        // Minimum execution time: 25_757_000 picoseconds.
        Weight::from_parts(27_226_000, 3666)
            .saturating_add(T::DbWeight::get().reads(3_u64))
            .saturating_add(T::DbWeight::get().writes(3_u64))
    }
    /// Storage: Democracy PublicProps (r:1 w:1)
    /// Proof: Democracy PublicProps (max_values: Some(1), max_size: Some(16702), added: 17197, mode: MaxEncodedLen)
    /// Storage: Democracy DepositOf (r:1 w:1)
    /// Proof: Democracy DepositOf (max_values: None, max_size: Some(3230), added: 5705, mode: MaxEncodedLen)
    /// Storage: System Account (r:1 w:1)
    /// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
    /// Storage: Democracy MetadataOf (r:3 w:1)
    /// Proof: Democracy MetadataOf (max_values: None, max_size: Some(53), added: 2528, mode: MaxEncodedLen)
    /// Storage: Democracy NextExternal (r:1 w:1)
    /// Proof: Democracy NextExternal (max_values: Some(1), max_size: Some(132), added: 627, mode: MaxEncodedLen)
    /// Storage: Democracy ReferendumInfoOf (r:1 w:1)
    /// Proof: Democracy ReferendumInfoOf (max_values: None, max_size: Some(201), added: 2676, mode: MaxEncodedLen)
    /// Storage: Democracy Blacklist (r:0 w:1)
    /// Proof: Democracy Blacklist (max_values: None, max_size: Some(3238), added: 5713, mode: MaxEncodedLen)
    fn blacklist() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `5910`
        //  Estimated: `18187`
        // Minimum execution time: 113_060_000 picoseconds.
        Weight::from_parts(114_813_000, 18187)
            .saturating_add(T::DbWeight::get().reads(8_u64))
            .saturating_add(T::DbWeight::get().writes(7_u64))
    }
    /// Storage: Democracy NextExternal (r:1 w:1)
    /// Proof: Democracy NextExternal (max_values: Some(1), max_size: Some(132), added: 627, mode: MaxEncodedLen)
    /// Storage: Democracy Blacklist (r:1 w:0)
    /// Proof: Democracy Blacklist (max_values: None, max_size: Some(3238), added: 5713, mode: MaxEncodedLen)
    fn external_propose() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `3416`
        //  Estimated: `6703`
        // Minimum execution time: 13_413_000 picoseconds.
        Weight::from_parts(13_794_000, 6703)
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
    /// Storage: Democracy NextExternal (r:0 w:1)
    /// Proof: Democracy NextExternal (max_values: Some(1), max_size: Some(132), added: 627, mode: MaxEncodedLen)
    fn external_propose_majority() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 3_213_000 picoseconds.
        Weight::from_parts(3_429_000, 0).saturating_add(T::DbWeight::get().writes(1_u64))
    }
    /// Storage: Democracy NextExternal (r:0 w:1)
    /// Proof: Democracy NextExternal (max_values: Some(1), max_size: Some(132), added: 627, mode: MaxEncodedLen)
    fn external_propose_default() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 3_280_000 picoseconds.
        Weight::from_parts(3_389_000, 0).saturating_add(T::DbWeight::get().writes(1_u64))
    }
    /// Storage: Democracy NextExternal (r:1 w:1)
    /// Proof: Democracy NextExternal (max_values: Some(1), max_size: Some(132), added: 627, mode: MaxEncodedLen)
    /// Storage: Democracy ReferendumCount (r:1 w:1)
    /// Proof: Democracy ReferendumCount (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
    /// Storage: Democracy MetadataOf (r:1 w:2)
    /// Proof: Democracy MetadataOf (max_values: None, max_size: Some(53), added: 2528, mode: MaxEncodedLen)
    /// Storage: Democracy ReferendumInfoOf (r:0 w:1)
    /// Proof: Democracy ReferendumInfoOf (max_values: None, max_size: Some(201), added: 2676, mode: MaxEncodedLen)
    fn fast_track() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `286`
        //  Estimated: `3518`
        // Minimum execution time: 28_142_000 picoseconds.
        Weight::from_parts(28_862_000, 3518)
            .saturating_add(T::DbWeight::get().reads(3_u64))
            .saturating_add(T::DbWeight::get().writes(5_u64))
    }
    /// Storage: Democracy NextExternal (r:1 w:1)
    /// Proof: Democracy NextExternal (max_values: Some(1), max_size: Some(132), added: 627, mode: MaxEncodedLen)
    /// Storage: Democracy Blacklist (r:1 w:1)
    /// Proof: Democracy Blacklist (max_values: None, max_size: Some(3238), added: 5713, mode: MaxEncodedLen)
    /// Storage: Democracy MetadataOf (r:1 w:1)
    /// Proof: Democracy MetadataOf (max_values: None, max_size: Some(53), added: 2528, mode: MaxEncodedLen)
    fn veto_external() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `3519`
        //  Estimated: `6703`
        // Minimum execution time: 32_395_000 picoseconds.
        Weight::from_parts(33_617_000, 6703)
            .saturating_add(T::DbWeight::get().reads(3_u64))
            .saturating_add(T::DbWeight::get().writes(3_u64))
    }
    /// Storage: Democracy PublicProps (r:1 w:1)
    /// Proof: Democracy PublicProps (max_values: Some(1), max_size: Some(16702), added: 17197, mode: MaxEncodedLen)
    /// Storage: Democracy DepositOf (r:1 w:1)
    /// Proof: Democracy DepositOf (max_values: None, max_size: Some(3230), added: 5705, mode: MaxEncodedLen)
    /// Storage: System Account (r:1 w:1)
    /// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
    /// Storage: Democracy MetadataOf (r:1 w:1)
    /// Proof: Democracy MetadataOf (max_values: None, max_size: Some(53), added: 2528, mode: MaxEncodedLen)
    fn cancel_proposal() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `5821`
        //  Estimated: `18187`
        // Minimum execution time: 92_255_000 picoseconds.
        Weight::from_parts(93_704_000, 18187)
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().writes(4_u64))
    }
    /// Storage: Democracy MetadataOf (r:1 w:1)
    /// Proof: Democracy MetadataOf (max_values: None, max_size: Some(53), added: 2528, mode: MaxEncodedLen)
    /// Storage: Democracy ReferendumInfoOf (r:0 w:1)
    /// Proof: Democracy ReferendumInfoOf (max_values: None, max_size: Some(201), added: 2676, mode: MaxEncodedLen)
    fn cancel_referendum() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `271`
        //  Estimated: `3518`
        // Minimum execution time: 19_623_000 picoseconds.
        Weight::from_parts(20_545_000, 3518)
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    /// Storage: Democracy LowestUnbaked (r:1 w:1)
    /// Proof: Democracy LowestUnbaked (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
    /// Storage: Democracy ReferendumCount (r:1 w:0)
    /// Proof: Democracy ReferendumCount (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
    /// Storage: Democracy ReferendumInfoOf (r:99 w:0)
    /// Proof: Democracy ReferendumInfoOf (max_values: None, max_size: Some(201), added: 2676, mode: MaxEncodedLen)
    /// The range of component `r` is `[0, 99]`.
    fn on_initialize_base(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `244 + r * (86 ±0)`
        //  Estimated: `1489 + r * (2676 ±0)`
        // Minimum execution time: 7_032_000 picoseconds.
        Weight::from_parts(7_931_421, 1489)
            // Standard Error: 7_395
            .saturating_add(Weight::from_parts(3_236_964, 0).saturating_mul(r.into()))
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(r.into())))
            .saturating_add(T::DbWeight::get().writes(1_u64))
            .saturating_add(Weight::from_parts(0, 2676).saturating_mul(r.into()))
    }
    /// Storage: Democracy LowestUnbaked (r:1 w:1)
    /// Proof: Democracy LowestUnbaked (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
    /// Storage: Democracy ReferendumCount (r:1 w:0)
    /// Proof: Democracy ReferendumCount (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
    /// Storage: Democracy LastTabledWasExternal (r:1 w:0)
    /// Proof: Democracy LastTabledWasExternal (max_values: Some(1), max_size: Some(1), added: 496, mode: MaxEncodedLen)
    /// Storage: Democracy NextExternal (r:1 w:0)
    /// Proof: Democracy NextExternal (max_values: Some(1), max_size: Some(132), added: 627, mode: MaxEncodedLen)
    /// Storage: Democracy PublicProps (r:1 w:0)
    /// Proof: Democracy PublicProps (max_values: Some(1), max_size: Some(16702), added: 17197, mode: MaxEncodedLen)
    /// Storage: Democracy ReferendumInfoOf (r:99 w:0)
    /// Proof: Democracy ReferendumInfoOf (max_values: None, max_size: Some(201), added: 2676, mode: MaxEncodedLen)
    /// The range of component `r` is `[0, 99]`.
    fn on_initialize_base_with_launch_period(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `244 + r * (86 ±0)`
        //  Estimated: `18187 + r * (2676 ±0)`
        // Minimum execution time: 10_524_000 picoseconds.
        Weight::from_parts(10_369_064, 18187)
            // Standard Error: 8_385
            .saturating_add(Weight::from_parts(3_242_334, 0).saturating_mul(r.into()))
            .saturating_add(T::DbWeight::get().reads(5_u64))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(r.into())))
            .saturating_add(T::DbWeight::get().writes(1_u64))
            .saturating_add(Weight::from_parts(0, 2676).saturating_mul(r.into()))
    }
    /// Storage: Democracy VotingOf (r:3 w:3)
    /// Proof: Democracy VotingOf (max_values: None, max_size: Some(3795), added: 6270, mode: MaxEncodedLen)
    /// Storage: Democracy ReferendumInfoOf (r:99 w:99)
    /// Proof: Democracy ReferendumInfoOf (max_values: None, max_size: Some(201), added: 2676, mode: MaxEncodedLen)
    /// Storage: Balances Locks (r:1 w:1)
    /// Proof: Balances Locks (max_values: None, max_size: Some(1299), added: 3774, mode: MaxEncodedLen)
    /// Storage: Balances Freezes (r:1 w:0)
    /// Proof: Balances Freezes (max_values: None, max_size: Some(49), added: 2524, mode: MaxEncodedLen)
    /// The range of component `r` is `[0, 99]`.
    fn delegate(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `830 + r * (108 ±0)`
        //  Estimated: `19800 + r * (2676 ±0)`
        // Minimum execution time: 46_106_000 picoseconds.
        Weight::from_parts(48_936_654, 19800)
            // Standard Error: 8_879
            .saturating_add(Weight::from_parts(4_708_141, 0).saturating_mul(r.into()))
            .saturating_add(T::DbWeight::get().reads(5_u64))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(r.into())))
            .saturating_add(T::DbWeight::get().writes(4_u64))
            .saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(r.into())))
            .saturating_add(Weight::from_parts(0, 2676).saturating_mul(r.into()))
    }
    /// Storage: Democracy VotingOf (r:2 w:2)
    /// Proof: Democracy VotingOf (max_values: None, max_size: Some(3795), added: 6270, mode: MaxEncodedLen)
    /// Storage: Democracy ReferendumInfoOf (r:99 w:99)
    /// Proof: Democracy ReferendumInfoOf (max_values: None, max_size: Some(201), added: 2676, mode: MaxEncodedLen)
    /// The range of component `r` is `[0, 99]`.
    fn undelegate(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `493 + r * (108 ±0)`
        //  Estimated: `13530 + r * (2676 ±0)`
        // Minimum execution time: 21_078_000 picoseconds.
        Weight::from_parts(22_732_737, 13530)
            // Standard Error: 7_969
            .saturating_add(Weight::from_parts(4_626_458, 0).saturating_mul(r.into()))
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(r.into())))
            .saturating_add(T::DbWeight::get().writes(2_u64))
            .saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(r.into())))
            .saturating_add(Weight::from_parts(0, 2676).saturating_mul(r.into()))
    }
    /// Storage: Democracy PublicProps (r:0 w:1)
    /// Proof: Democracy PublicProps (max_values: Some(1), max_size: Some(16702), added: 17197, mode: MaxEncodedLen)
    fn clear_public_proposals() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 3_229_000 picoseconds.
        Weight::from_parts(3_415_000, 0).saturating_add(T::DbWeight::get().writes(1_u64))
    }
    /// Storage: Democracy VotingOf (r:1 w:1)
    /// Proof: Democracy VotingOf (max_values: None, max_size: Some(3795), added: 6270, mode: MaxEncodedLen)
    /// Storage: Balances Locks (r:1 w:1)
    /// Proof: Balances Locks (max_values: None, max_size: Some(1299), added: 3774, mode: MaxEncodedLen)
    /// Storage: Balances Freezes (r:1 w:0)
    /// Proof: Balances Freezes (max_values: None, max_size: Some(49), added: 2524, mode: MaxEncodedLen)
    /// Storage: System Account (r:1 w:1)
    /// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
    /// The range of component `r` is `[0, 99]`.
    fn unlock_remove(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `563`
        //  Estimated: `7260`
        // Minimum execution time: 25_735_000 picoseconds.
        Weight::from_parts(41_341_468, 7260)
            // Standard Error: 3_727
            .saturating_add(Weight::from_parts(94_755, 0).saturating_mul(r.into()))
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().writes(3_u64))
    }
    /// Storage: Democracy VotingOf (r:1 w:1)
    /// Proof: Democracy VotingOf (max_values: None, max_size: Some(3795), added: 6270, mode: MaxEncodedLen)
    /// Storage: Balances Locks (r:1 w:1)
    /// Proof: Balances Locks (max_values: None, max_size: Some(1299), added: 3774, mode: MaxEncodedLen)
    /// Storage: Balances Freezes (r:1 w:0)
    /// Proof: Balances Freezes (max_values: None, max_size: Some(49), added: 2524, mode: MaxEncodedLen)
    /// Storage: System Account (r:1 w:1)
    /// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
    /// The range of component `r` is `[0, 99]`.
    fn unlock_set(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `564 + r * (22 ±0)`
        //  Estimated: `7260`
        // Minimum execution time: 36_233_000 picoseconds.
        Weight::from_parts(39_836_017, 7260)
            // Standard Error: 1_791
            .saturating_add(Weight::from_parts(132_158, 0).saturating_mul(r.into()))
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().writes(3_u64))
    }
    /// Storage: Democracy ReferendumInfoOf (r:1 w:1)
    /// Proof: Democracy ReferendumInfoOf (max_values: None, max_size: Some(201), added: 2676, mode: MaxEncodedLen)
    /// Storage: Democracy VotingOf (r:1 w:1)
    /// Proof: Democracy VotingOf (max_values: None, max_size: Some(3795), added: 6270, mode: MaxEncodedLen)
    /// The range of component `r` is `[1, 100]`.
    fn remove_vote(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `728 + r * (26 ±0)`
        //  Estimated: `7260`
        // Minimum execution time: 16_081_000 picoseconds.
        Weight::from_parts(19_624_101, 7260)
            // Standard Error: 1_639
            .saturating_add(Weight::from_parts(133_630, 0).saturating_mul(r.into()))
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    /// Storage: Democracy ReferendumInfoOf (r:1 w:1)
    /// Proof: Democracy ReferendumInfoOf (max_values: None, max_size: Some(201), added: 2676, mode: MaxEncodedLen)
    /// Storage: Democracy VotingOf (r:1 w:1)
    /// Proof: Democracy VotingOf (max_values: None, max_size: Some(3795), added: 6270, mode: MaxEncodedLen)
    /// The range of component `r` is `[1, 100]`.
    fn remove_other_vote(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `728 + r * (26 ±0)`
        //  Estimated: `7260`
        // Minimum execution time: 15_634_000 picoseconds.
        Weight::from_parts(19_573_407, 7260)
            // Standard Error: 1_790
            .saturating_add(Weight::from_parts(139_707, 0).saturating_mul(r.into()))
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    /// Storage: Democracy NextExternal (r:1 w:0)
    /// Proof: Democracy NextExternal (max_values: Some(1), max_size: Some(132), added: 627, mode: MaxEncodedLen)
    /// Storage: Preimage StatusFor (r:1 w:0)
    /// Proof: Preimage StatusFor (max_values: None, max_size: Some(91), added: 2566, mode: MaxEncodedLen)
    /// Storage: Democracy MetadataOf (r:0 w:1)
    /// Proof: Democracy MetadataOf (max_values: None, max_size: Some(53), added: 2528, mode: MaxEncodedLen)
    fn set_external_metadata() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `356`
        //  Estimated: `3556`
        // Minimum execution time: 18_344_000 picoseconds.
        Weight::from_parts(18_727_000, 3556)
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
    /// Storage: Democracy NextExternal (r:1 w:0)
    /// Proof: Democracy NextExternal (max_values: Some(1), max_size: Some(132), added: 627, mode: MaxEncodedLen)
    /// Storage: Democracy MetadataOf (r:1 w:1)
    /// Proof: Democracy MetadataOf (max_values: None, max_size: Some(53), added: 2528, mode: MaxEncodedLen)
    fn clear_external_metadata() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `286`
        //  Estimated: `3518`
        // Minimum execution time: 16_497_000 picoseconds.
        Weight::from_parts(16_892_000, 3518)
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
    /// Storage: Democracy PublicProps (r:1 w:0)
    /// Proof: Democracy PublicProps (max_values: Some(1), max_size: Some(16702), added: 17197, mode: MaxEncodedLen)
    /// Storage: Preimage StatusFor (r:1 w:0)
    /// Proof: Preimage StatusFor (max_values: None, max_size: Some(91), added: 2566, mode: MaxEncodedLen)
    /// Storage: Democracy MetadataOf (r:0 w:1)
    /// Proof: Democracy MetadataOf (max_values: None, max_size: Some(53), added: 2528, mode: MaxEncodedLen)
    fn set_proposal_metadata() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `4888`
        //  Estimated: `18187`
        // Minimum execution time: 39_517_000 picoseconds.
        Weight::from_parts(40_632_000, 18187)
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
    /// Storage: Democracy PublicProps (r:1 w:0)
    /// Proof: Democracy PublicProps (max_values: Some(1), max_size: Some(16702), added: 17197, mode: MaxEncodedLen)
    /// Storage: Democracy MetadataOf (r:1 w:1)
    /// Proof: Democracy MetadataOf (max_values: None, max_size: Some(53), added: 2528, mode: MaxEncodedLen)
    fn clear_proposal_metadata() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `4822`
        //  Estimated: `18187`
        // Minimum execution time: 37_108_000 picoseconds.
        Weight::from_parts(37_599_000, 18187)
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
    /// Storage: Preimage StatusFor (r:1 w:0)
    /// Proof: Preimage StatusFor (max_values: None, max_size: Some(91), added: 2566, mode: MaxEncodedLen)
    /// Storage: Democracy MetadataOf (r:0 w:1)
    /// Proof: Democracy MetadataOf (max_values: None, max_size: Some(53), added: 2528, mode: MaxEncodedLen)
    fn set_referendum_metadata() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `144`
        //  Estimated: `3556`
        // Minimum execution time: 13_997_000 picoseconds.
        Weight::from_parts(14_298_000, 3556)
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
    /// Storage: Democracy ReferendumInfoOf (r:1 w:0)
    /// Proof: Democracy ReferendumInfoOf (max_values: None, max_size: Some(201), added: 2676, mode: MaxEncodedLen)
    /// Storage: Democracy MetadataOf (r:1 w:1)
    /// Proof: Democracy MetadataOf (max_values: None, max_size: Some(53), added: 2528, mode: MaxEncodedLen)
    fn clear_referendum_metadata() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `302`
        //  Estimated: `3666`
        // Minimum execution time: 18_122_000 picoseconds.
        Weight::from_parts(18_655_000, 3666)
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
}
