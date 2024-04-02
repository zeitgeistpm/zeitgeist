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

//! Autogenerated weights for zrml_swaps
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
// --pallet=zrml_swaps
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --template=./misc/weight_template.hbs
// --header=./HEADER_GPL3
// --output=./zrml/swaps/src/weights.rs

#![allow(unused_parens)]
#![allow(unused_imports)]

use core::marker::PhantomData;
use frame_support::{traits::Get, weights::Weight};

///  Trait containing the required functions for weight retrival within
/// zrml_swaps (automatically generated)
pub trait WeightInfoZeitgeist {
    fn pool_exit(a: u32) -> Weight;
    fn pool_exit_with_exact_asset_amount() -> Weight;
    fn pool_exit_with_exact_pool_amount() -> Weight;
    fn pool_join(a: u32) -> Weight;
    fn pool_join_with_exact_asset_amount() -> Weight;
    fn pool_join_with_exact_pool_amount() -> Weight;
    fn swap_exact_amount_in() -> Weight;
    fn swap_exact_amount_out() -> Weight;
    fn open_pool(a: u32) -> Weight;
    fn close_pool(a: u32) -> Weight;
    fn destroy_pool(a: u32) -> Weight;
}

/// Weight functions for zrml_swaps (automatically generated)
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfoZeitgeist for WeightInfo<T> {
    /// Storage: Swaps Pools (r:1 w:0)
    /// Proof: Swaps Pools (max_values: None, max_size: Some(3579), added: 6054, mode: MaxEncodedLen)
    /// Storage: MarketAssets Asset (r:66 w:0)
    /// Proof: MarketAssets Asset (max_values: None, max_size: Some(225), added: 2700, mode: MaxEncodedLen)
    /// Storage: Tokens TotalIssuance (r:1 w:1)
    /// Proof: Tokens TotalIssuance (max_values: None, max_size: Some(43), added: 2518, mode: MaxEncodedLen)
    /// Storage: Tokens Accounts (r:131 w:131)
    /// Proof: Tokens Accounts (max_values: None, max_size: Some(123), added: 2598, mode: MaxEncodedLen)
    /// Storage: System Account (r:1 w:0)
    /// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
    /// The range of component `a` is `[2, 65]`.
    fn pool_exit(a: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `848 + a * (176 ±0)`
        //  Estimated: `13777 + a * (5196 ±0)`
        // Minimum execution time: 129_570 nanoseconds.
        Weight::from_parts(128_926_395, 13777)
            // Standard Error: 377_426
            .saturating_add(Weight::from_parts(36_975_518, 0).saturating_mul(a.into()))
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().reads((2_u64).saturating_mul(a.into())))
            .saturating_add(T::DbWeight::get().writes(2))
            .saturating_add(T::DbWeight::get().writes((2_u64).saturating_mul(a.into())))
            .saturating_add(Weight::from_parts(0, 5196).saturating_mul(a.into()))
    }
    /// Storage: Swaps Pools (r:1 w:0)
    /// Proof: Swaps Pools (max_values: None, max_size: Some(3579), added: 6054, mode: MaxEncodedLen)
    /// Storage: MarketAssets Asset (r:2 w:0)
    /// Proof: MarketAssets Asset (max_values: None, max_size: Some(225), added: 2700, mode: MaxEncodedLen)
    /// Storage: Tokens Accounts (r:3 w:3)
    /// Proof: Tokens Accounts (max_values: None, max_size: Some(123), added: 2598, mode: MaxEncodedLen)
    /// Storage: Tokens TotalIssuance (r:1 w:1)
    /// Proof: Tokens TotalIssuance (max_values: None, max_size: Some(43), added: 2518, mode: MaxEncodedLen)
    /// Storage: System Account (r:1 w:0)
    /// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
    fn pool_exit_with_exact_asset_amount() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `5546`
        //  Estimated: `18973`
        // Minimum execution time: 119_830 nanoseconds.
        Weight::from_parts(147_671_000, 18973)
            .saturating_add(T::DbWeight::get().reads(6))
            .saturating_add(T::DbWeight::get().writes(4))
    }
    /// Storage: Swaps Pools (r:1 w:0)
    /// Proof: Swaps Pools (max_values: None, max_size: Some(3579), added: 6054, mode: MaxEncodedLen)
    /// Storage: MarketAssets Asset (r:2 w:0)
    /// Proof: MarketAssets Asset (max_values: None, max_size: Some(225), added: 2700, mode: MaxEncodedLen)
    /// Storage: Tokens TotalIssuance (r:1 w:1)
    /// Proof: Tokens TotalIssuance (max_values: None, max_size: Some(43), added: 2518, mode: MaxEncodedLen)
    /// Storage: Tokens Accounts (r:3 w:3)
    /// Proof: Tokens Accounts (max_values: None, max_size: Some(123), added: 2598, mode: MaxEncodedLen)
    /// Storage: System Account (r:1 w:0)
    /// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
    fn pool_exit_with_exact_pool_amount() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `5546`
        //  Estimated: `18973`
        // Minimum execution time: 141_210 nanoseconds.
        Weight::from_parts(147_801_000, 18973)
            .saturating_add(T::DbWeight::get().reads(6))
            .saturating_add(T::DbWeight::get().writes(4))
    }
    /// Storage: Swaps Pools (r:1 w:0)
    /// Proof: Swaps Pools (max_values: None, max_size: Some(3579), added: 6054, mode: MaxEncodedLen)
    /// Storage: MarketAssets Asset (r:66 w:0)
    /// Proof: MarketAssets Asset (max_values: None, max_size: Some(225), added: 2700, mode: MaxEncodedLen)
    /// Storage: Tokens TotalIssuance (r:1 w:1)
    /// Proof: Tokens TotalIssuance (max_values: None, max_size: Some(43), added: 2518, mode: MaxEncodedLen)
    /// Storage: Tokens Accounts (r:131 w:131)
    /// Proof: Tokens Accounts (max_values: None, max_size: Some(123), added: 2598, mode: MaxEncodedLen)
    /// The range of component `a` is `[2, 65]`.
    fn pool_join(a: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `726 + a * (289 ±0)`
        //  Estimated: `11170 + a * (5196 ±0)`
        // Minimum execution time: 125_590 nanoseconds.
        Weight::from_parts(63_107_045, 11170)
            // Standard Error: 353_437
            .saturating_add(Weight::from_parts(30_963_348, 0).saturating_mul(a.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().reads((2_u64).saturating_mul(a.into())))
            .saturating_add(T::DbWeight::get().writes(2))
            .saturating_add(T::DbWeight::get().writes((2_u64).saturating_mul(a.into())))
            .saturating_add(Weight::from_parts(0, 5196).saturating_mul(a.into()))
    }
    /// Storage: Swaps Pools (r:1 w:0)
    /// Proof: Swaps Pools (max_values: None, max_size: Some(3579), added: 6054, mode: MaxEncodedLen)
    /// Storage: MarketAssets Asset (r:2 w:0)
    /// Proof: MarketAssets Asset (max_values: None, max_size: Some(225), added: 2700, mode: MaxEncodedLen)
    /// Storage: Tokens TotalIssuance (r:1 w:1)
    /// Proof: Tokens TotalIssuance (max_values: None, max_size: Some(43), added: 2518, mode: MaxEncodedLen)
    /// Storage: Tokens Accounts (r:3 w:3)
    /// Proof: Tokens Accounts (max_values: None, max_size: Some(123), added: 2598, mode: MaxEncodedLen)
    fn pool_join_with_exact_asset_amount() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `6229`
        //  Estimated: `16366`
        // Minimum execution time: 120_300 nanoseconds.
        Weight::from_parts(138_451_000, 16366)
            .saturating_add(T::DbWeight::get().reads(5))
            .saturating_add(T::DbWeight::get().writes(4))
    }
    /// Storage: Swaps Pools (r:1 w:0)
    /// Proof: Swaps Pools (max_values: None, max_size: Some(3579), added: 6054, mode: MaxEncodedLen)
    /// Storage: MarketAssets Asset (r:2 w:0)
    /// Proof: MarketAssets Asset (max_values: None, max_size: Some(225), added: 2700, mode: MaxEncodedLen)
    /// Storage: Tokens TotalIssuance (r:1 w:1)
    /// Proof: Tokens TotalIssuance (max_values: None, max_size: Some(43), added: 2518, mode: MaxEncodedLen)
    /// Storage: Tokens Accounts (r:3 w:3)
    /// Proof: Tokens Accounts (max_values: None, max_size: Some(123), added: 2598, mode: MaxEncodedLen)
    fn pool_join_with_exact_pool_amount() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `6229`
        //  Estimated: `16366`
        // Minimum execution time: 101_230 nanoseconds.
        Weight::from_parts(126_591_000, 16366)
            .saturating_add(T::DbWeight::get().reads(5))
            .saturating_add(T::DbWeight::get().writes(4))
    }
    /// Storage: Swaps Pools (r:1 w:0)
    /// Proof: Swaps Pools (max_values: None, max_size: Some(3579), added: 6054, mode: MaxEncodedLen)
    /// Storage: MarketAssets Asset (r:2 w:0)
    /// Proof: MarketAssets Asset (max_values: None, max_size: Some(225), added: 2700, mode: MaxEncodedLen)
    /// Storage: Tokens Accounts (r:4 w:4)
    /// Proof: Tokens Accounts (max_values: None, max_size: Some(123), added: 2598, mode: MaxEncodedLen)
    /// Storage: System Account (r:1 w:0)
    /// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
    fn swap_exact_amount_in() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `5144`
        //  Estimated: `19053`
        // Minimum execution time: 163_771 nanoseconds.
        Weight::from_parts(201_801_000, 19053)
            .saturating_add(T::DbWeight::get().reads(6))
            .saturating_add(T::DbWeight::get().writes(4))
    }
    /// Storage: Swaps Pools (r:1 w:0)
    /// Proof: Swaps Pools (max_values: None, max_size: Some(3579), added: 6054, mode: MaxEncodedLen)
    /// Storage: MarketAssets Asset (r:2 w:0)
    /// Proof: MarketAssets Asset (max_values: None, max_size: Some(225), added: 2700, mode: MaxEncodedLen)
    /// Storage: Tokens Accounts (r:4 w:4)
    /// Proof: Tokens Accounts (max_values: None, max_size: Some(123), added: 2598, mode: MaxEncodedLen)
    /// Storage: System Account (r:1 w:0)
    /// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
    fn swap_exact_amount_out() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `5144`
        //  Estimated: `19053`
        // Minimum execution time: 166_981 nanoseconds.
        Weight::from_parts(202_971_000, 19053)
            .saturating_add(T::DbWeight::get().reads(6))
            .saturating_add(T::DbWeight::get().writes(4))
    }
    /// Storage: Swaps Pools (r:1 w:1)
    /// Proof: Swaps Pools (max_values: None, max_size: Some(3579), added: 6054, mode: MaxEncodedLen)
    /// The range of component `a` is `[2, 65]`.
    fn open_pool(a: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `167 + a * (54 ±0)`
        //  Estimated: `6054`
        // Minimum execution time: 18_030 nanoseconds.
        Weight::from_parts(19_889_995, 6054)
            // Standard Error: 12_386
            .saturating_add(Weight::from_parts(570_444, 0).saturating_mul(a.into()))
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    /// Storage: Swaps Pools (r:1 w:1)
    /// Proof: Swaps Pools (max_values: None, max_size: Some(3579), added: 6054, mode: MaxEncodedLen)
    /// The range of component `a` is `[2, 65]`.
    fn close_pool(a: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `167 + a * (54 ±0)`
        //  Estimated: `6054`
        // Minimum execution time: 17_140 nanoseconds.
        Weight::from_parts(19_467_788, 6054)
            // Standard Error: 3_011
            .saturating_add(Weight::from_parts(283_373, 0).saturating_mul(a.into()))
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    /// Storage: Swaps Pools (r:1 w:1)
    /// Proof: Swaps Pools (max_values: None, max_size: Some(3579), added: 6054, mode: MaxEncodedLen)
    /// Storage: MarketAssets Asset (r:65 w:0)
    /// Proof: MarketAssets Asset (max_values: None, max_size: Some(225), added: 2700, mode: MaxEncodedLen)
    /// Storage: Tokens Accounts (r:65 w:65)
    /// Proof: Tokens Accounts (max_values: None, max_size: Some(123), added: 2598, mode: MaxEncodedLen)
    /// Storage: System Account (r:1 w:1)
    /// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
    /// Storage: Tokens TotalIssuance (r:65 w:65)
    /// Proof: Tokens TotalIssuance (max_values: None, max_size: Some(43), added: 2518, mode: MaxEncodedLen)
    /// The range of component `a` is `[2, 65]`.
    fn destroy_pool(a: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `569 + a * (214 ±0)`
        //  Estimated: `8661 + a * (5116 ±0)`
        // Minimum execution time: 99_530 nanoseconds.
        Weight::from_parts(13_849_323, 8661)
            // Standard Error: 316_948
            .saturating_add(Weight::from_parts(31_161_021, 0).saturating_mul(a.into()))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().reads((2_u64).saturating_mul(a.into())))
            .saturating_add(T::DbWeight::get().writes(2))
            .saturating_add(T::DbWeight::get().writes((2_u64).saturating_mul(a.into())))
            .saturating_add(Weight::from_parts(0, 5116).saturating_mul(a.into()))
    }
}
