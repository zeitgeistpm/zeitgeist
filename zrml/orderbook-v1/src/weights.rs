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

//! Autogenerated weights for zrml_orderbook_v1
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: `2023-10-26`, STEPS: `10`, REPEAT: `1000`, LOW RANGE: `[]`, HIGH RANGE: `[]`
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
// --pallet=zrml_orderbook_v1
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --template=./misc/weight_template.hbs
// --header=./HEADER_GPL3
// --output=./zrml/orderbook-v1/src/weights.rs

#![allow(unused_parens)]
#![allow(unused_imports)]

use core::marker::PhantomData;
use frame_support::{traits::Get, weights::Weight};

///  Trait containing the required functions for weight retrival within
/// zrml_orderbook_v1 (automatically generated)
pub trait WeightInfoZeitgeist {
    fn remove_order_ask() -> Weight;
    fn remove_order_bid() -> Weight;
    fn fill_order_ask() -> Weight;
    fn fill_order_bid() -> Weight;
    fn place_order_ask() -> Weight;
    fn place_order_bid() -> Weight;
}

/// Weight functions for zrml_orderbook_v1 (automatically generated)
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfoZeitgeist for WeightInfo<T> {
    /// Storage: Orderbook Orders (r:1 w:1)
    /// Proof: Orderbook Orders (max_values: None, max_size: Some(143), added: 2618, mode: MaxEncodedLen)
    /// Storage: Tokens Reserves (r:1 w:1)
    /// Proof: Tokens Reserves (max_values: None, max_size: Some(1276), added: 3751, mode: MaxEncodedLen)
    /// Storage: Tokens Accounts (r:1 w:1)
    /// Proof: Tokens Accounts (max_values: None, max_size: Some(123), added: 2598, mode: MaxEncodedLen)
    fn remove_order_ask() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `675`
        //  Estimated: `8967`
        // Minimum execution time: 41_210 nanoseconds.
        Weight::from_parts(51_230_000, 8967)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    /// Storage: Orderbook Orders (r:1 w:1)
    /// Proof: Orderbook Orders (max_values: None, max_size: Some(143), added: 2618, mode: MaxEncodedLen)
    /// Storage: Balances Reserves (r:1 w:1)
    /// Proof: Balances Reserves (max_values: None, max_size: Some(1249), added: 3724, mode: MaxEncodedLen)
    fn remove_order_bid() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `323`
        //  Estimated: `6342`
        // Minimum execution time: 37_110 nanoseconds.
        Weight::from_parts(45_930_000, 6342)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    /// Storage: Orderbook Orders (r:1 w:1)
    /// Proof: Orderbook Orders (max_values: None, max_size: Some(143), added: 2618, mode: MaxEncodedLen)
    /// Storage: MarketCommons Markets (r:1 w:0)
    /// Proof: MarketCommons Markets (max_values: None, max_size: Some(678), added: 3153, mode: MaxEncodedLen)
    /// Storage: Tokens Reserves (r:1 w:1)
    /// Proof: Tokens Reserves (max_values: None, max_size: Some(1276), added: 3751, mode: MaxEncodedLen)
    /// Storage: Tokens Accounts (r:2 w:2)
    /// Proof: Tokens Accounts (max_values: None, max_size: Some(123), added: 2598, mode: MaxEncodedLen)
    /// Storage: System Account (r:1 w:1)
    /// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
    fn fill_order_ask() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1348`
        //  Estimated: `17325`
        // Minimum execution time: 97_511 nanoseconds.
        Weight::from_parts(120_261_000, 17325)
            .saturating_add(T::DbWeight::get().reads(6))
            .saturating_add(T::DbWeight::get().writes(5))
    }
    /// Storage: Orderbook Orders (r:1 w:1)
    /// Proof: Orderbook Orders (max_values: None, max_size: Some(143), added: 2618, mode: MaxEncodedLen)
    /// Storage: MarketCommons Markets (r:1 w:0)
    /// Proof: MarketCommons Markets (max_values: None, max_size: Some(678), added: 3153, mode: MaxEncodedLen)
    /// Storage: Tokens Accounts (r:2 w:2)
    /// Proof: Tokens Accounts (max_values: None, max_size: Some(123), added: 2598, mode: MaxEncodedLen)
    /// Storage: Balances Reserves (r:1 w:1)
    /// Proof: Balances Reserves (max_values: None, max_size: Some(1249), added: 3724, mode: MaxEncodedLen)
    /// Storage: System Account (r:1 w:1)
    /// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
    fn fill_order_bid() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1288`
        //  Estimated: `17298`
        // Minimum execution time: 86_600 nanoseconds.
        Weight::from_parts(106_891_000, 17298)
            .saturating_add(T::DbWeight::get().reads(6))
            .saturating_add(T::DbWeight::get().writes(5))
    }
    /// Storage: MarketCommons Markets (r:1 w:0)
    /// Proof: MarketCommons Markets (max_values: None, max_size: Some(678), added: 3153, mode: MaxEncodedLen)
    /// Storage: Orderbook NextOrderId (r:1 w:1)
    /// Proof: Orderbook NextOrderId (max_values: Some(1), max_size: Some(16), added: 511, mode: MaxEncodedLen)
    /// Storage: Tokens Reserves (r:1 w:1)
    /// Proof: Tokens Reserves (max_values: None, max_size: Some(1276), added: 3751, mode: MaxEncodedLen)
    /// Storage: Tokens Accounts (r:1 w:1)
    /// Proof: Tokens Accounts (max_values: None, max_size: Some(123), added: 2598, mode: MaxEncodedLen)
    /// Storage: Orderbook Orders (r:0 w:1)
    /// Proof: Orderbook Orders (max_values: None, max_size: Some(143), added: 2618, mode: MaxEncodedLen)
    fn place_order_ask() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `664`
        //  Estimated: `10013`
        // Minimum execution time: 55_910 nanoseconds.
        Weight::from_parts(69_621_000, 10013)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(4))
    }
    /// Storage: MarketCommons Markets (r:1 w:0)
    /// Proof: MarketCommons Markets (max_values: None, max_size: Some(678), added: 3153, mode: MaxEncodedLen)
    /// Storage: Orderbook NextOrderId (r:1 w:1)
    /// Proof: Orderbook NextOrderId (max_values: Some(1), max_size: Some(16), added: 511, mode: MaxEncodedLen)
    /// Storage: Balances Reserves (r:1 w:1)
    /// Proof: Balances Reserves (max_values: None, max_size: Some(1249), added: 3724, mode: MaxEncodedLen)
    /// Storage: Orderbook Orders (r:0 w:1)
    /// Proof: Orderbook Orders (max_values: None, max_size: Some(143), added: 2618, mode: MaxEncodedLen)
    fn place_order_bid() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `372`
        //  Estimated: `7388`
        // Minimum execution time: 44_780 nanoseconds.
        Weight::from_parts(55_840_000, 7388)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }
}
