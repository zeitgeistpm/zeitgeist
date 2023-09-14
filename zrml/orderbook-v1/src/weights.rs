//! Autogenerated weights for zrml_orderbook_v1
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: `2023-09-14`, STEPS: `10`, REPEAT: `1000`, LOW RANGE: `[]`, HIGH RANGE: `[]`
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
// --pallet=zrml_orderbook_v1
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./zrml/orderbook-v1/src/weights.rs
// --template=./misc/weight_template.hbs

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
    /// Proof: Orderbook Orders (max_values: None, max_size: Some(175), added: 2650, mode: MaxEncodedLen)
    /// Storage: Tokens Reserves (r:1 w:1)
    /// Proof: Tokens Reserves (max_values: None, max_size: Some(1276), added: 3751, mode: MaxEncodedLen)
    /// Storage: Tokens Accounts (r:1 w:1)
    /// Proof: Tokens Accounts (max_values: None, max_size: Some(123), added: 2598, mode: MaxEncodedLen)
    fn remove_order_ask() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `635`
        //  Estimated: `8999`
        // Minimum execution time: 28_000 nanoseconds.
        Weight::from_parts(31_000_000, 8999)
            .saturating_add(T::DbWeight::get().reads(3_u64))
            .saturating_add(T::DbWeight::get().writes(3_u64))
    }
    /// Storage: Orderbook Orders (r:1 w:1)
    /// Proof: Orderbook Orders (max_values: None, max_size: Some(175), added: 2650, mode: MaxEncodedLen)
    /// Storage: Balances Reserves (r:1 w:1)
    /// Proof: Balances Reserves (max_values: None, max_size: Some(1249), added: 3724, mode: MaxEncodedLen)
    fn remove_order_bid() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `316`
        //  Estimated: `6374`
        // Minimum execution time: 27_000 nanoseconds.
        Weight::from_parts(29_000_000, 6374)
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    /// Storage: Orderbook Orders (r:1 w:1)
    /// Proof: Orderbook Orders (max_values: None, max_size: Some(175), added: 2650, mode: MaxEncodedLen)
    /// Storage: MarketCommons Markets (r:1 w:0)
    /// Proof: MarketCommons Markets (max_values: None, max_size: Some(541), added: 3016, mode: MaxEncodedLen)
    /// Storage: Tokens Reserves (r:1 w:1)
    /// Proof: Tokens Reserves (max_values: None, max_size: Some(1276), added: 3751, mode: MaxEncodedLen)
    /// Storage: Tokens Accounts (r:2 w:2)
    /// Proof: Tokens Accounts (max_values: None, max_size: Some(123), added: 2598, mode: MaxEncodedLen)
    /// Storage: System Account (r:1 w:1)
    /// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
    fn fill_order_ask() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1304`
        //  Estimated: `17220`
        // Minimum execution time: 70_000 nanoseconds.
        Weight::from_parts(73_000_000, 17220)
            .saturating_add(T::DbWeight::get().reads(6_u64))
            .saturating_add(T::DbWeight::get().writes(5_u64))
    }
    /// Storage: Orderbook Orders (r:1 w:1)
    /// Proof: Orderbook Orders (max_values: None, max_size: Some(175), added: 2650, mode: MaxEncodedLen)
    /// Storage: MarketCommons Markets (r:1 w:0)
    /// Proof: MarketCommons Markets (max_values: None, max_size: Some(541), added: 3016, mode: MaxEncodedLen)
    /// Storage: Tokens Accounts (r:2 w:2)
    /// Proof: Tokens Accounts (max_values: None, max_size: Some(123), added: 2598, mode: MaxEncodedLen)
    /// Storage: Balances Reserves (r:1 w:1)
    /// Proof: Balances Reserves (max_values: None, max_size: Some(1249), added: 3724, mode: MaxEncodedLen)
    /// Storage: System Account (r:1 w:1)
    /// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
    fn fill_order_bid() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1243`
        //  Estimated: `17193`
        // Minimum execution time: 62_000 nanoseconds.
        Weight::from_parts(64_000_000, 17193)
            .saturating_add(T::DbWeight::get().reads(6_u64))
            .saturating_add(T::DbWeight::get().writes(5_u64))
    }
    /// Storage: MarketCommons Markets (r:1 w:0)
    /// Proof: MarketCommons Markets (max_values: None, max_size: Some(541), added: 3016, mode: MaxEncodedLen)
    /// Storage: Orderbook NextOrderId (r:1 w:1)
    /// Proof: Orderbook NextOrderId (max_values: Some(1), max_size: Some(16), added: 511, mode: MaxEncodedLen)
    /// Storage: Tokens Reserves (r:1 w:1)
    /// Proof: Tokens Reserves (max_values: None, max_size: Some(1276), added: 3751, mode: MaxEncodedLen)
    /// Storage: Tokens Accounts (r:1 w:1)
    /// Proof: Tokens Accounts (max_values: None, max_size: Some(123), added: 2598, mode: MaxEncodedLen)
    /// Storage: Orderbook Orders (r:0 w:1)
    /// Proof: Orderbook Orders (max_values: None, max_size: Some(175), added: 2650, mode: MaxEncodedLen)
    fn place_order_ask() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `588`
        //  Estimated: `9876`
        // Minimum execution time: 35_000 nanoseconds.
        Weight::from_parts(38_000_000, 9876)
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().writes(4_u64))
    }
    /// Storage: MarketCommons Markets (r:1 w:0)
    /// Proof: MarketCommons Markets (max_values: None, max_size: Some(541), added: 3016, mode: MaxEncodedLen)
    /// Storage: Orderbook NextOrderId (r:1 w:1)
    /// Proof: Orderbook NextOrderId (max_values: Some(1), max_size: Some(16), added: 511, mode: MaxEncodedLen)
    /// Storage: Balances Reserves (r:1 w:1)
    /// Proof: Balances Reserves (max_values: None, max_size: Some(1249), added: 3724, mode: MaxEncodedLen)
    /// Storage: Orderbook Orders (r:0 w:1)
    /// Proof: Orderbook Orders (max_values: None, max_size: Some(175), added: 2650, mode: MaxEncodedLen)
    fn place_order_bid() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `330`
        //  Estimated: `7251`
        // Minimum execution time: 31_000 nanoseconds.
        Weight::from_parts(32_000_000, 7251)
            .saturating_add(T::DbWeight::get().reads(3_u64))
            .saturating_add(T::DbWeight::get().writes(3_u64))
    }
}
