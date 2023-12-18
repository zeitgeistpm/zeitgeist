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

//! Autogenerated weights for zrml_orderbook
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: `2023-11-03`, STEPS: `10`, REPEAT: `10`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `chralt`, CPU: `<UNKNOWN>`
//! EXECUTION: `Some(Wasm)`, WASM-EXECUTION: `Compiled`, CHAIN: `Some("dev")`, DB CACHE: `1024`

// Executed Command:
// ./target/release/zeitgeist
// benchmark
// pallet
// --chain=dev
// --steps=10
// --repeat=1000
// --pallet=zrml_orderbook
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --template=./misc/weight_template.hbs
// --header=./HEADER_GPL3
// --output=./zrml/orderbook/src/weights.rs

#![allow(unused_parens)]
#![allow(unused_imports)]

use core::marker::PhantomData;
use frame_support::{traits::Get, weights::Weight};

///  Trait containing the required functions for weight retrival within
/// zrml_orderbook (automatically generated)
pub trait WeightInfoZeitgeist {
    fn remove_order() -> Weight;
    fn fill_order() -> Weight;
    fn place_order() -> Weight;
}

/// Weight functions for zrml_orderbook (automatically generated)
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfoZeitgeist for WeightInfo<T> {
    /// Storage: Orderbook Orders (r:1 w:1)
    /// Proof: Orderbook Orders (max_values: None, max_size: Some(142), added: 2617, mode: MaxEncodedLen)
    /// Storage: Balances Reserves (r:1 w:1)
    /// Proof: Balances Reserves (max_values: None, max_size: Some(1249), added: 3724, mode: MaxEncodedLen)
    fn remove_order() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `283`
        //  Estimated: `6341`
        // Minimum execution time: 26_000 nanoseconds.
        Weight::from_parts(27_000_000, 6341)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    /// Storage: Orderbook Orders (r:1 w:1)
    /// Proof: Orderbook Orders (max_values: None, max_size: Some(142), added: 2617, mode: MaxEncodedLen)
    /// Storage: MarketCommons Markets (r:1 w:0)
    /// Proof: MarketCommons Markets (max_values: None, max_size: Some(678), added: 3153, mode: MaxEncodedLen)
    /// Storage: Balances Reserves (r:1 w:1)
    /// Proof: Balances Reserves (max_values: None, max_size: Some(1249), added: 3724, mode: MaxEncodedLen)
    /// Storage: System Account (r:1 w:1)
    /// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
    /// Storage: Tokens Accounts (r:2 w:2)
    /// Proof: Tokens Accounts (max_values: None, max_size: Some(123), added: 2598, mode: MaxEncodedLen)
    fn fill_order() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1081`
        //  Estimated: `17297`
        // Minimum execution time: 67_000 nanoseconds.
        Weight::from_parts(68_000_000, 17297)
            .saturating_add(T::DbWeight::get().reads(6))
            .saturating_add(T::DbWeight::get().writes(5))
    }
    /// Storage: MarketCommons Markets (r:1 w:0)
    /// Proof: MarketCommons Markets (max_values: None, max_size: Some(678), added: 3153, mode: MaxEncodedLen)
    /// Storage: Orderbook NextOrderId (r:1 w:1)
    /// Proof: Orderbook NextOrderId (max_values: Some(1), max_size: Some(16), added: 511, mode: MaxEncodedLen)
    /// Storage: Balances Reserves (r:1 w:1)
    /// Proof: Balances Reserves (max_values: None, max_size: Some(1249), added: 3724, mode: MaxEncodedLen)
    /// Storage: Orderbook Orders (r:0 w:1)
    /// Proof: Orderbook Orders (max_values: None, max_size: Some(142), added: 2617, mode: MaxEncodedLen)
    fn place_order() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `372`
        //  Estimated: `7388`
        // Minimum execution time: 33_000 nanoseconds.
        Weight::from_parts(34_000_000, 7388)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }
}
