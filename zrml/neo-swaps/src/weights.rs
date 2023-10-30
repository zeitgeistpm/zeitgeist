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

//! Autogenerated weights for zrml_neo_swaps
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: `2023-10-12`, STEPS: `10`, REPEAT: `1000`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `zeitgeist-benchmark`, CPU: `AMD EPYC 7601 32-Core Processor`
//! EXECUTION: `Some(Wasm)`, WASM-EXECUTION: `Compiled`, CHAIN: `Some("dev")`, DB CACHE: `1024`

// Executed Command:
// ./target/production/zeitgeist
// benchmark
// pallet
// --chain=dev
// --steps=10
// --repeat=1000
// --pallet=zrml_neo_swaps
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --template=./misc/weight_template.hbs
// --header=./HEADER_GPL3
// --output=./zrml/neo-swaps/src/weights.rs

#![allow(unused_parens)]
#![allow(unused_imports)]

use core::marker::PhantomData;
use frame_support::{traits::Get, weights::Weight};

///  Trait containing the required functions for weight retrival within
/// zrml_neo_swaps (automatically generated)
pub trait WeightInfoZeitgeist {
    fn buy(n: u32) -> Weight;
    fn sell() -> Weight;
    fn join() -> Weight;
    fn exit() -> Weight;
    fn withdraw_fees() -> Weight;
    fn deploy_pool() -> Weight;
}

/// Weight functions for zrml_neo_swaps (automatically generated)
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfoZeitgeist for WeightInfo<T> {
    /// Storage: MarketCommons Markets (r:1 w:0)
    /// Proof: MarketCommons Markets (max_values: None, max_size: Some(542), added: 3017, mode: MaxEncodedLen)
    /// Storage: NeoSwaps Pools (r:1 w:1)
    /// Proof: NeoSwaps Pools (max_values: None, max_size: Some(4652), added: 7127, mode: MaxEncodedLen)
    /// Storage: System Account (r:2 w:2)
    /// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
    /// Storage: Tokens Accounts (r:3 w:3)
    /// Proof: Tokens Accounts (max_values: None, max_size: Some(123), added: 2598, mode: MaxEncodedLen)
    /// Storage: Tokens TotalIssuance (r:2 w:2)
    /// Proof: Tokens TotalIssuance (max_values: None, max_size: Some(43), added: 2518, mode: MaxEncodedLen)
    fn buy(_n: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `2868`
        //  Estimated: `28188`
        // Minimum execution time: 383_942 nanoseconds.
        Weight::from_parts(452_972_000, 28188)
            .saturating_add(T::DbWeight::get().reads(9))
            .saturating_add(T::DbWeight::get().writes(8))
    }
    /// Storage: MarketCommons Markets (r:1 w:0)
    /// Proof: MarketCommons Markets (max_values: None, max_size: Some(542), added: 3017, mode: MaxEncodedLen)
    /// Storage: NeoSwaps Pools (r:1 w:1)
    /// Proof: NeoSwaps Pools (max_values: None, max_size: Some(4652), added: 7127, mode: MaxEncodedLen)
    /// Storage: Tokens Accounts (r:3 w:3)
    /// Proof: Tokens Accounts (max_values: None, max_size: Some(123), added: 2598, mode: MaxEncodedLen)
    /// Storage: System Account (r:2 w:2)
    /// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
    /// Storage: Tokens TotalIssuance (r:2 w:2)
    /// Proof: Tokens TotalIssuance (max_values: None, max_size: Some(43), added: 2518, mode: MaxEncodedLen)
    fn sell() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `3034`
        //  Estimated: `28188`
        // Minimum execution time: 401_532 nanoseconds.
        Weight::from_parts(491_103_000, 28188)
            .saturating_add(T::DbWeight::get().reads(9))
            .saturating_add(T::DbWeight::get().writes(8))
    }
    /// Storage: MarketCommons Markets (r:1 w:0)
    /// Proof: MarketCommons Markets (max_values: None, max_size: Some(542), added: 3017, mode: MaxEncodedLen)
    /// Storage: NeoSwaps Pools (r:1 w:1)
    /// Proof: NeoSwaps Pools (max_values: None, max_size: Some(4652), added: 7127, mode: MaxEncodedLen)
    /// Storage: Tokens Accounts (r:4 w:4)
    /// Proof: Tokens Accounts (max_values: None, max_size: Some(123), added: 2598, mode: MaxEncodedLen)
    fn join() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `2756`
        //  Estimated: `20536`
        // Minimum execution time: 122_280 nanoseconds.
        Weight::from_parts(149_371_000, 20536)
            .saturating_add(T::DbWeight::get().reads(6))
            .saturating_add(T::DbWeight::get().writes(5))
    }
    /// Storage: MarketCommons Markets (r:1 w:0)
    /// Proof: MarketCommons Markets (max_values: None, max_size: Some(542), added: 3017, mode: MaxEncodedLen)
    /// Storage: NeoSwaps Pools (r:1 w:1)
    /// Proof: NeoSwaps Pools (max_values: None, max_size: Some(4652), added: 7127, mode: MaxEncodedLen)
    /// Storage: Tokens Accounts (r:4 w:4)
    /// Proof: Tokens Accounts (max_values: None, max_size: Some(123), added: 2598, mode: MaxEncodedLen)
    /// Storage: System Account (r:1 w:0)
    /// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
    fn exit() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `2524`
        //  Estimated: `23143`
        // Minimum execution time: 124_590 nanoseconds.
        Weight::from_parts(151_600_000, 23143)
            .saturating_add(T::DbWeight::get().reads(7))
            .saturating_add(T::DbWeight::get().writes(5))
    }
    /// Storage: NeoSwaps Pools (r:1 w:1)
    /// Proof: NeoSwaps Pools (max_values: None, max_size: Some(4652), added: 7127, mode: MaxEncodedLen)
    /// Storage: System Account (r:1 w:1)
    /// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
    fn withdraw_fees() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1819`
        //  Estimated: `9734`
        // Minimum execution time: 77_180 nanoseconds.
        Weight::from_parts(93_340_000, 9734)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    /// Storage: MarketCommons Markets (r:1 w:0)
    /// Proof: MarketCommons Markets (max_values: None, max_size: Some(542), added: 3017, mode: MaxEncodedLen)
    /// Storage: NeoSwaps Pools (r:1 w:1)
    /// Proof: NeoSwaps Pools (max_values: None, max_size: Some(4652), added: 7127, mode: MaxEncodedLen)
    /// Storage: Tokens Accounts (r:4 w:4)
    /// Proof: Tokens Accounts (max_values: None, max_size: Some(123), added: 2598, mode: MaxEncodedLen)
    /// Storage: System Account (r:1 w:1)
    /// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
    fn deploy_pool() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `2241`
        //  Estimated: `23143`
        // Minimum execution time: 166_011 nanoseconds.
        Weight::from_parts(201_540_000, 23143)
            .saturating_add(T::DbWeight::get().reads(7))
            .saturating_add(T::DbWeight::get().writes(6))
    }
}
