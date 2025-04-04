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

//! Autogenerated weights for zrml_hybrid_router
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
// --pallet=zrml_hybrid_router
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --template=./misc/weight_template.hbs
// --header=./HEADER_GPL3
// --output=./zrml/hybrid-router/src/weights.rs

#![allow(unused_parens)]
#![allow(unused_imports)]

use core::marker::PhantomData;
use frame_support::{traits::Get, weights::Weight};

///  Trait containing the required functions for weight retrival within
/// zrml_hybrid_router (automatically generated)
pub trait WeightInfoZeitgeist {
    fn buy(n: u32, o: u32) -> Weight;
    fn sell(n: u32, o: u32) -> Weight;
}

/// Weight functions for zrml_hybrid_router (automatically generated)
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfoZeitgeist for WeightInfo<T> {
    /// Storage: `MarketCommons::Markets` (r:1 w:0)
    /// Proof: `MarketCommons::Markets` (`max_values`: None, `max_size`: Some(708), added: 3183, mode: `MaxEncodedLen`)
    /// Storage: `Orderbook::Orders` (r:10 w:11)
    /// Proof: `Orderbook::Orders` (`max_values`: None, `max_size`: Some(170), added: 2645, mode: `MaxEncodedLen`)
    /// Storage: `NeoSwaps::MarketIdToPoolId` (r:1 w:0)
    /// Proof: `NeoSwaps::MarketIdToPoolId` (`max_values`: None, `max_size`: Some(40), added: 2515, mode: `MaxEncodedLen`)
    /// Storage: `NeoSwaps::Pools` (r:1 w:1)
    /// Proof: `NeoSwaps::Pools` (`max_values`: None, `max_size`: Some(152829), added: 155304, mode: `MaxEncodedLen`)
    /// Storage: `System::Account` (r:12 w:12)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(132), added: 2607, mode: `MaxEncodedLen`)
    /// Storage: `Tokens::Accounts` (r:27 w:27)
    /// Proof: `Tokens::Accounts` (`max_values`: None, `max_size`: Some(137), added: 2612, mode: `MaxEncodedLen`)
    /// Storage: `Tokens::TotalIssuance` (r:16 w:16)
    /// Proof: `Tokens::TotalIssuance` (`max_values`: None, `max_size`: Some(57), added: 2532, mode: `MaxEncodedLen`)
    /// Storage: `Tokens::Reserves` (r:10 w:10)
    /// Proof: `Tokens::Reserves` (`max_values`: None, `max_size`: Some(1290), added: 3765, mode: `MaxEncodedLen`)
    /// Storage: `Orderbook::NextOrderId` (r:1 w:1)
    /// Proof: `Orderbook::NextOrderId` (`max_values`: Some(1), `max_size`: Some(16), added: 511, mode: `MaxEncodedLen`)
    /// Storage: `Balances::Reserves` (r:1 w:1)
    /// Proof: `Balances::Reserves` (`max_values`: None, `max_size`: Some(1249), added: 3724, mode: `MaxEncodedLen`)
    /// The range of component `n` is `[2, 16]`.
    /// The range of component `o` is `[0, 10]`.
    fn buy(n: u32, o: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1365 + n * (185 ±0) + o * (497 ±0)`
        //  Estimated: `156294 + n * (2612 ±0) + o * (3765 ±0)`
        // Minimum execution time: 1_230_755 nanoseconds.
        Weight::from_parts(1_241_677_000, 156294)
            // Standard Error: 9_261_068
            .saturating_add(Weight::from_parts(239_255_448, 0).saturating_mul(n.into()))
            // Standard Error: 15_188_685
            .saturating_add(Weight::from_parts(715_470_714, 0).saturating_mul(o.into()))
            .saturating_add(T::DbWeight::get().reads(8))
            .saturating_add(T::DbWeight::get().reads((2_u64).saturating_mul(n.into())))
            .saturating_add(T::DbWeight::get().reads((4_u64).saturating_mul(o.into())))
            .saturating_add(T::DbWeight::get().writes(7))
            .saturating_add(T::DbWeight::get().writes((2_u64).saturating_mul(n.into())))
            .saturating_add(T::DbWeight::get().writes((4_u64).saturating_mul(o.into())))
            .saturating_add(Weight::from_parts(0, 2612).saturating_mul(n.into()))
            .saturating_add(Weight::from_parts(0, 3765).saturating_mul(o.into()))
    }
    /// Storage: `MarketCommons::Markets` (r:1 w:0)
    /// Proof: `MarketCommons::Markets` (`max_values`: None, `max_size`: Some(708), added: 3183, mode: `MaxEncodedLen`)
    /// Storage: `Tokens::Accounts` (r:21 w:21)
    /// Proof: `Tokens::Accounts` (`max_values`: None, `max_size`: Some(137), added: 2612, mode: `MaxEncodedLen`)
    /// Storage: `Orderbook::Orders` (r:10 w:11)
    /// Proof: `Orderbook::Orders` (`max_values`: None, `max_size`: Some(170), added: 2645, mode: `MaxEncodedLen`)
    /// Storage: `NeoSwaps::MarketIdToPoolId` (r:1 w:0)
    /// Proof: `NeoSwaps::MarketIdToPoolId` (`max_values`: None, `max_size`: Some(40), added: 2515, mode: `MaxEncodedLen`)
    /// Storage: `NeoSwaps::Pools` (r:1 w:1)
    /// Proof: `NeoSwaps::Pools` (`max_values`: None, `max_size`: Some(152829), added: 155304, mode: `MaxEncodedLen`)
    /// Storage: `System::Account` (r:12 w:12)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(132), added: 2607, mode: `MaxEncodedLen`)
    /// Storage: `Tokens::TotalIssuance` (r:10 w:10)
    /// Proof: `Tokens::TotalIssuance` (`max_values`: None, `max_size`: Some(57), added: 2532, mode: `MaxEncodedLen`)
    /// Storage: `Balances::Reserves` (r:10 w:10)
    /// Proof: `Balances::Reserves` (`max_values`: None, `max_size`: Some(1249), added: 3724, mode: `MaxEncodedLen`)
    /// Storage: `Orderbook::NextOrderId` (r:1 w:1)
    /// Proof: `Orderbook::NextOrderId` (`max_values`: Some(1), `max_size`: Some(16), added: 511, mode: `MaxEncodedLen`)
    /// Storage: `Tokens::Reserves` (r:1 w:1)
    /// Proof: `Tokens::Reserves` (`max_values`: None, `max_size`: Some(1290), added: 3765, mode: `MaxEncodedLen`)
    /// The range of component `n` is `[2, 10]`.
    /// The range of component `o` is `[0, 10]`.
    fn sell(n: u32, o: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1441 + n * (183 ±0) + o * (339 ±0)`
        //  Estimated: `156294 + n * (2612 ±0) + o * (3724 ±0)`
        // Minimum execution time: 916_440 nanoseconds.
        Weight::from_parts(927_929_000, 156294)
            // Standard Error: 4_814_974
            .saturating_add(Weight::from_parts(136_360_462, 0).saturating_mul(n.into()))
            // Standard Error: 4_949_097
            .saturating_add(Weight::from_parts(592_482_482, 0).saturating_mul(o.into()))
            .saturating_add(T::DbWeight::get().reads(8))
            .saturating_add(T::DbWeight::get().reads((2_u64).saturating_mul(n.into())))
            .saturating_add(T::DbWeight::get().reads((4_u64).saturating_mul(o.into())))
            .saturating_add(T::DbWeight::get().writes(7))
            .saturating_add(T::DbWeight::get().writes((2_u64).saturating_mul(n.into())))
            .saturating_add(T::DbWeight::get().writes((4_u64).saturating_mul(o.into())))
            .saturating_add(Weight::from_parts(0, 2612).saturating_mul(n.into()))
            .saturating_add(Weight::from_parts(0, 3724).saturating_mul(o.into()))
    }
}
