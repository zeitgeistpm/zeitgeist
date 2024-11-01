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

//! Autogenerated weights for zrml_neo_swaps
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: `2024-10-31`, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
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
    fn sell(n: u32) -> Weight;
    fn join_in_place(n: u32) -> Weight;
    fn join_reassigned(n: u32) -> Weight;
    fn join_leaf(n: u32) -> Weight;
    fn exit(n: u32) -> Weight;
    fn withdraw_fees() -> Weight;
    fn deploy_pool(n: u32) -> Weight;
    fn combo_buy(n: u32) -> Weight;
    fn combo_sell(n: u32) -> Weight;
    fn deploy_combinatorial_pool(n: u32) -> Weight;
    fn decision_market_oracle_evaluate() -> Weight;
}

/// Weight functions for zrml_neo_swaps (automatically generated)
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfoZeitgeist for WeightInfo<T> {
    /// Storage: `NeoSwaps::Pools` (r:1 w:1)
    /// Proof: `NeoSwaps::Pools` (`max_values`: None, `max_size`: Some(152829), added: 155304, mode: `MaxEncodedLen`)
    /// Storage: `MarketCommons::Markets` (r:1 w:0)
    /// Proof: `MarketCommons::Markets` (`max_values`: None, `max_size`: Some(708), added: 3183, mode: `MaxEncodedLen`)
    /// Storage: `System::Account` (r:3 w:3)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(132), added: 2607, mode: `MaxEncodedLen`)
    /// Storage: `Tokens::Accounts` (r:129 w:129)
    /// Proof: `Tokens::Accounts` (`max_values`: None, `max_size`: Some(137), added: 2612, mode: `MaxEncodedLen`)
    /// Storage: `Tokens::TotalIssuance` (r:128 w:128)
    /// Proof: `Tokens::TotalIssuance` (`max_values`: None, `max_size`: Some(57), added: 2532, mode: `MaxEncodedLen`)
    /// The range of component `n` is `[2, 128]`.
    fn buy(n: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1372 + n * (182 ±0)`
        //  Estimated: `156294 + n * (2612 ±0)`
        // Minimum execution time: 402_539 nanoseconds.
        Weight::from_parts(302_340_193, 156294)
            // Standard Error: 92_254
            .saturating_add(Weight::from_parts(53_376_748, 0).saturating_mul(n.into()))
            .saturating_add(T::DbWeight::get().reads(6))
            .saturating_add(T::DbWeight::get().reads((2_u64).saturating_mul(n.into())))
            .saturating_add(T::DbWeight::get().writes(5))
            .saturating_add(T::DbWeight::get().writes((2_u64).saturating_mul(n.into())))
            .saturating_add(Weight::from_parts(0, 2612).saturating_mul(n.into()))
    }
    /// Storage: `NeoSwaps::Pools` (r:1 w:1)
    /// Proof: `NeoSwaps::Pools` (`max_values`: None, `max_size`: Some(152829), added: 155304, mode: `MaxEncodedLen`)
    /// Storage: `MarketCommons::Markets` (r:1 w:0)
    /// Proof: `MarketCommons::Markets` (`max_values`: None, `max_size`: Some(708), added: 3183, mode: `MaxEncodedLen`)
    /// Storage: `Tokens::Accounts` (r:129 w:129)
    /// Proof: `Tokens::Accounts` (`max_values`: None, `max_size`: Some(137), added: 2612, mode: `MaxEncodedLen`)
    /// Storage: `System::Account` (r:3 w:3)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(132), added: 2607, mode: `MaxEncodedLen`)
    /// Storage: `Tokens::TotalIssuance` (r:128 w:128)
    /// Proof: `Tokens::TotalIssuance` (`max_values`: None, `max_size`: Some(57), added: 2532, mode: `MaxEncodedLen`)
    /// The range of component `n` is `[2, 128]`.
    fn sell(n: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1503 + n * (182 ±0)`
        //  Estimated: `156294 + n * (2612 ±0)`
        // Minimum execution time: 337_007 nanoseconds.
        Weight::from_parts(250_443_677, 156294)
            // Standard Error: 116_699
            .saturating_add(Weight::from_parts(60_461_360, 0).saturating_mul(n.into()))
            .saturating_add(T::DbWeight::get().reads(6))
            .saturating_add(T::DbWeight::get().reads((2_u64).saturating_mul(n.into())))
            .saturating_add(T::DbWeight::get().writes(5))
            .saturating_add(T::DbWeight::get().writes((2_u64).saturating_mul(n.into())))
            .saturating_add(Weight::from_parts(0, 2612).saturating_mul(n.into()))
    }
    /// Storage: `NeoSwaps::Pools` (r:1 w:1)
    /// Proof: `NeoSwaps::Pools` (`max_values`: None, `max_size`: Some(152829), added: 155304, mode: `MaxEncodedLen`)
    /// Storage: `MarketCommons::Markets` (r:1 w:0)
    /// Proof: `MarketCommons::Markets` (`max_values`: None, `max_size`: Some(708), added: 3183, mode: `MaxEncodedLen`)
    /// Storage: `Tokens::Accounts` (r:256 w:256)
    /// Proof: `Tokens::Accounts` (`max_values`: None, `max_size`: Some(137), added: 2612, mode: `MaxEncodedLen`)
    /// Storage: `System::Account` (r:1 w:0)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(132), added: 2607, mode: `MaxEncodedLen`)
    /// The range of component `n` is `[2, 128]`.
    fn join_in_place(n: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `139400 + n * (216 ±0)`
        //  Estimated: `156294 + n * (5224 ±0)`
        // Minimum execution time: 396_540 nanoseconds.
        Weight::from_parts(350_023_672, 156294)
            // Standard Error: 157_275
            .saturating_add(Weight::from_parts(31_812_304, 0).saturating_mul(n.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().reads((2_u64).saturating_mul(n.into())))
            .saturating_add(T::DbWeight::get().writes(1))
            .saturating_add(T::DbWeight::get().writes((2_u64).saturating_mul(n.into())))
            .saturating_add(Weight::from_parts(0, 5224).saturating_mul(n.into()))
    }
    /// Storage: `NeoSwaps::Pools` (r:1 w:1)
    /// Proof: `NeoSwaps::Pools` (`max_values`: None, `max_size`: Some(152829), added: 155304, mode: `MaxEncodedLen`)
    /// Storage: `MarketCommons::Markets` (r:1 w:0)
    /// Proof: `MarketCommons::Markets` (`max_values`: None, `max_size`: Some(708), added: 3183, mode: `MaxEncodedLen`)
    /// Storage: `Tokens::Accounts` (r:256 w:256)
    /// Proof: `Tokens::Accounts` (`max_values`: None, `max_size`: Some(137), added: 2612, mode: `MaxEncodedLen`)
    /// Storage: `System::Account` (r:1 w:0)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(132), added: 2607, mode: `MaxEncodedLen`)
    /// The range of component `n` is `[2, 128]`.
    fn join_reassigned(n: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `139196 + n * (216 ±0)`
        //  Estimated: `156294 + n * (5224 ±0)`
        // Minimum execution time: 405_608 nanoseconds.
        Weight::from_parts(376_463_342, 156294)
            // Standard Error: 158_653
            .saturating_add(Weight::from_parts(32_337_731, 0).saturating_mul(n.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().reads((2_u64).saturating_mul(n.into())))
            .saturating_add(T::DbWeight::get().writes(1))
            .saturating_add(T::DbWeight::get().writes((2_u64).saturating_mul(n.into())))
            .saturating_add(Weight::from_parts(0, 5224).saturating_mul(n.into()))
    }
    /// Storage: `NeoSwaps::Pools` (r:1 w:1)
    /// Proof: `NeoSwaps::Pools` (`max_values`: None, `max_size`: Some(152829), added: 155304, mode: `MaxEncodedLen`)
    /// Storage: `MarketCommons::Markets` (r:1 w:0)
    /// Proof: `MarketCommons::Markets` (`max_values`: None, `max_size`: Some(708), added: 3183, mode: `MaxEncodedLen`)
    /// Storage: `Tokens::Accounts` (r:256 w:256)
    /// Proof: `Tokens::Accounts` (`max_values`: None, `max_size`: Some(137), added: 2612, mode: `MaxEncodedLen`)
    /// Storage: `System::Account` (r:1 w:0)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(132), added: 2607, mode: `MaxEncodedLen`)
    /// The range of component `n` is `[2, 128]`.
    fn join_leaf(n: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `138700 + n * (216 ±0)`
        //  Estimated: `156294 + n * (5224 ±0)`
        // Minimum execution time: 470_430 nanoseconds.
        Weight::from_parts(404_406_469, 156294)
            // Standard Error: 162_346
            .saturating_add(Weight::from_parts(31_654_906, 0).saturating_mul(n.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().reads((2_u64).saturating_mul(n.into())))
            .saturating_add(T::DbWeight::get().writes(1))
            .saturating_add(T::DbWeight::get().writes((2_u64).saturating_mul(n.into())))
            .saturating_add(Weight::from_parts(0, 5224).saturating_mul(n.into()))
    }
    /// Storage: `NeoSwaps::Pools` (r:1 w:1)
    /// Proof: `NeoSwaps::Pools` (`max_values`: None, `max_size`: Some(152829), added: 155304, mode: `MaxEncodedLen`)
    /// Storage: `MarketCommons::Markets` (r:1 w:0)
    /// Proof: `MarketCommons::Markets` (`max_values`: None, `max_size`: Some(708), added: 3183, mode: `MaxEncodedLen`)
    /// Storage: `Tokens::Accounts` (r:256 w:256)
    /// Proof: `Tokens::Accounts` (`max_values`: None, `max_size`: Some(137), added: 2612, mode: `MaxEncodedLen`)
    /// Storage: `System::Account` (r:1 w:0)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(132), added: 2607, mode: `MaxEncodedLen`)
    /// The range of component `n` is `[2, 128]`.
    fn exit(n: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `139297 + n * (216 ±0)`
        //  Estimated: `156294 + n * (5224 ±0)`
        // Minimum execution time: 433_429 nanoseconds.
        Weight::from_parts(445_783_938, 156294)
            // Standard Error: 148_856
            .saturating_add(Weight::from_parts(31_235_322, 0).saturating_mul(n.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().reads((2_u64).saturating_mul(n.into())))
            .saturating_add(T::DbWeight::get().writes(1))
            .saturating_add(T::DbWeight::get().writes((2_u64).saturating_mul(n.into())))
            .saturating_add(Weight::from_parts(0, 5224).saturating_mul(n.into()))
    }
    /// Storage: `NeoSwaps::Pools` (r:1 w:1)
    /// Proof: `NeoSwaps::Pools` (`max_values`: None, `max_size`: Some(152829), added: 155304, mode: `MaxEncodedLen`)
    /// Storage: `System::Account` (r:2 w:2)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(132), added: 2607, mode: `MaxEncodedLen`)
    fn withdraw_fees() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `137883`
        //  Estimated: `156294`
        // Minimum execution time: 319_797 nanoseconds.
        Weight::from_parts(365_799_000, 156294)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    /// Storage: `MarketCommons::Markets` (r:1 w:0)
    /// Proof: `MarketCommons::Markets` (`max_values`: None, `max_size`: Some(708), added: 3183, mode: `MaxEncodedLen`)
    /// Storage: `NeoSwaps::MarketIdToPoolId` (r:1 w:1)
    /// Proof: `NeoSwaps::MarketIdToPoolId` (`max_values`: None, `max_size`: Some(40), added: 2515, mode: `MaxEncodedLen`)
    /// Storage: `Tokens::Accounts` (r:256 w:256)
    /// Proof: `Tokens::Accounts` (`max_values`: None, `max_size`: Some(137), added: 2612, mode: `MaxEncodedLen`)
    /// Storage: `System::Account` (r:1 w:1)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(132), added: 2607, mode: `MaxEncodedLen`)
    /// Storage: `NeoSwaps::PoolCount` (r:1 w:1)
    /// Proof: `NeoSwaps::PoolCount` (`max_values`: Some(1), `max_size`: Some(16), added: 511, mode: `MaxEncodedLen`)
    /// Storage: `NeoSwaps::Pools` (r:0 w:1)
    /// Proof: `NeoSwaps::Pools` (`max_values`: None, `max_size`: Some(152829), added: 155304, mode: `MaxEncodedLen`)
    /// The range of component `n` is `[2, 128]`.
    fn deploy_pool(n: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `611 + n * (81 ±0)`
        //  Estimated: `4173 + n * (5224 ±0)`
        // Minimum execution time: 159_294 nanoseconds.
        Weight::from_parts(100_340_149, 4173)
            // Standard Error: 67_332
            .saturating_add(Weight::from_parts(33_452_544, 0).saturating_mul(n.into()))
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().reads((2_u64).saturating_mul(n.into())))
            .saturating_add(T::DbWeight::get().writes(4))
            .saturating_add(T::DbWeight::get().writes((2_u64).saturating_mul(n.into())))
            .saturating_add(Weight::from_parts(0, 5224).saturating_mul(n.into()))
    }
    /// Storage: `NeoSwaps::Pools` (r:1 w:1)
    /// Proof: `NeoSwaps::Pools` (`max_values`: None, `max_size`: Some(152829), added: 155304, mode: `MaxEncodedLen`)
    /// Storage: `MarketCommons::Markets` (r:7 w:0)
    /// Proof: `MarketCommons::Markets` (`max_values`: None, `max_size`: Some(708), added: 3183, mode: `MaxEncodedLen`)
    /// Storage: `System::Account` (r:3 w:3)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(132), added: 2607, mode: `MaxEncodedLen`)
    /// Storage: `Tokens::Accounts` (r:256 w:256)
    /// Proof: `Tokens::Accounts` (`max_values`: None, `max_size`: Some(137), added: 2612, mode: `MaxEncodedLen`)
    /// Storage: `Tokens::TotalIssuance` (r:128 w:128)
    /// Proof: `Tokens::TotalIssuance` (`max_values`: None, `max_size`: Some(57), added: 2532, mode: `MaxEncodedLen`)
    /// The range of component `n` is `[1, 7]`.
    fn combo_buy(n: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0 + n * (2721 ±0)`
        //  Estimated: `156294 + n * (38153 ±999)`
        // Minimum execution time: 462_410 nanoseconds.
        Weight::from_parts(465_820_000, 156294)
            // Standard Error: 23_261_249
            .saturating_add(Weight::from_parts(901_905_964, 0).saturating_mul(n.into()))
            .saturating_add(T::DbWeight::get().reads(11))
            .saturating_add(T::DbWeight::get().reads((23_u64).saturating_mul(n.into())))
            .saturating_add(T::DbWeight::get().writes(10))
            .saturating_add(T::DbWeight::get().writes((22_u64).saturating_mul(n.into())))
            .saturating_add(Weight::from_parts(0, 38153).saturating_mul(n.into()))
    }
    /// Storage: `NeoSwaps::Pools` (r:1 w:1)
    /// Proof: `NeoSwaps::Pools` (`max_values`: None, `max_size`: Some(152829), added: 155304, mode: `MaxEncodedLen`)
    /// Storage: `MarketCommons::Markets` (r:7 w:0)
    /// Proof: `MarketCommons::Markets` (`max_values`: None, `max_size`: Some(708), added: 3183, mode: `MaxEncodedLen`)
    /// Storage: `Tokens::Accounts` (r:255 w:255)
    /// Proof: `Tokens::Accounts` (`max_values`: None, `max_size`: Some(137), added: 2612, mode: `MaxEncodedLen`)
    /// Storage: `System::Account` (r:3 w:3)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(132), added: 2607, mode: `MaxEncodedLen`)
    /// Storage: `Tokens::TotalIssuance` (r:128 w:128)
    /// Proof: `Tokens::TotalIssuance` (`max_values`: None, `max_size`: Some(57), added: 2532, mode: `MaxEncodedLen`)
    /// The range of component `n` is `[1, 7]`.
    fn combo_sell(n: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0 + n * (3627 ±0)`
        //  Estimated: `156294 + n * (38153 ±484)`
        // Minimum execution time: 523_942 nanoseconds.
        Weight::from_parts(527_472_000, 156294)
            // Standard Error: 38_920_367
            .saturating_add(Weight::from_parts(1_535_695_671, 0).saturating_mul(n.into()))
            .saturating_add(T::DbWeight::get().reads(10))
            .saturating_add(T::DbWeight::get().reads((23_u64).saturating_mul(n.into())))
            .saturating_add(T::DbWeight::get().writes(9))
            .saturating_add(T::DbWeight::get().writes((22_u64).saturating_mul(n.into())))
            .saturating_add(Weight::from_parts(0, 38153).saturating_mul(n.into()))
    }
    /// Storage: `MarketCommons::Markets` (r:7 w:0)
    /// Proof: `MarketCommons::Markets` (`max_values`: None, `max_size`: Some(708), added: 3183, mode: `MaxEncodedLen`)
    /// Storage: `System::Account` (r:2 w:2)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(132), added: 2607, mode: `MaxEncodedLen`)
    /// Storage: `Tokens::Accounts` (r:382 w:382)
    /// Proof: `Tokens::Accounts` (`max_values`: None, `max_size`: Some(137), added: 2612, mode: `MaxEncodedLen`)
    /// Storage: `Tokens::TotalIssuance` (r:254 w:254)
    /// Proof: `Tokens::TotalIssuance` (`max_values`: None, `max_size`: Some(57), added: 2532, mode: `MaxEncodedLen`)
    /// Storage: `NeoSwaps::PoolCount` (r:1 w:1)
    /// Proof: `NeoSwaps::PoolCount` (`max_values`: Some(1), `max_size`: Some(16), added: 511, mode: `MaxEncodedLen`)
    /// Storage: `NeoSwaps::Pools` (r:0 w:1)
    /// Proof: `NeoSwaps::Pools` (`max_values`: None, `max_size`: Some(152829), added: 155304, mode: `MaxEncodedLen`)
    /// The range of component `n` is `[1, 7]`.
    fn deploy_combinatorial_pool(n: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `357 + n * (185 ±0)`
        //  Estimated: `11438 + n * (57229 ±969)`
        // Minimum execution time: 7_103_129 nanoseconds.
        Weight::from_parts(7_159_730_000, 11438)
            // Standard Error: 1_623_272_081
            .saturating_add(Weight::from_parts(61_965_055_407, 0).saturating_mul(n.into()))
            .saturating_add(T::DbWeight::get().reads(10))
            .saturating_add(T::DbWeight::get().reads((37_u64).saturating_mul(n.into())))
            .saturating_add(T::DbWeight::get().writes(10))
            .saturating_add(T::DbWeight::get().writes((37_u64).saturating_mul(n.into())))
            .saturating_add(Weight::from_parts(0, 57229).saturating_mul(n.into()))
    }
    /// Storage: `NeoSwaps::Pools` (r:1 w:0)
    /// Proof: `NeoSwaps::Pools` (`max_values`: None, `max_size`: Some(152829), added: 155304, mode: `MaxEncodedLen`)
    fn decision_market_oracle_evaluate() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `492`
        //  Estimated: `156294`
        // Minimum execution time: 91_762 nanoseconds.
        Weight::from_parts(93_152_000, 156294).saturating_add(T::DbWeight::get().reads(1))
    }
}
