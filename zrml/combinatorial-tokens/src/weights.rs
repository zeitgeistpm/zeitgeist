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

//! Autogenerated weights for zrml_combinatorial_tokens
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: `2024-10-24`, STEPS: `2`, REPEAT: `0`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `blackbird`, CPU: `<UNKNOWN>`
//! EXECUTION: ``, WASM-EXECUTION: `Compiled`, CHAIN: `Some("dev")`, DB CACHE: `1024`

// Executed Command:
// ./target/release/zeitgeist
// benchmark
// pallet
// --chain=dev
// --steps=2
// --repeat=0
// --pallet=zrml_combinatorial_tokens
// --extrinsic=*
// --execution=native
// --wasm-execution=compiled
// --heap-pages=4096
// --template=./misc/weight_template.hbs
// --header=./HEADER_GPL3
// --output=./zrml/combinatorial-tokens/src/weights.rs

#![allow(unused_parens)]
#![allow(unused_imports)]

use core::marker::PhantomData;
use frame_support::{traits::Get, weights::Weight};

///  Trait containing the required functions for weight retrival within
/// zrml_combinatorial_tokens (automatically generated)
pub trait WeightInfoZeitgeist {
    fn split_position_vertical_sans_parent(n: u32, ) -> Weight;
    fn split_position_vertical_with_parent(n: u32, ) -> Weight;
    fn split_position_horizontal(n: u32, ) -> Weight;
    fn merge_position_vertical_sans_parent(n: u32, ) -> Weight;
    fn merge_position_vertical_with_parent(n: u32, ) -> Weight;
    fn merge_position_horizontal(n: u32, ) -> Weight;
    fn redeem_position_sans_parent(n: u32, ) -> Weight;
    fn redeem_position_with_parent(n: u32, ) -> Weight;
}

/// Weight functions for zrml_combinatorial_tokens (automatically generated)
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfoZeitgeist for WeightInfo<T> {
    /// Storage: `MarketCommons::Markets` (r:1 w:0)
    /// Proof: `MarketCommons::Markets` (`max_values`: None, `max_size`: Some(708), added: 3183, mode: `MaxEncodedLen`)
    /// Storage: `System::Account` (r:1 w:1)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(132), added: 2607, mode: `MaxEncodedLen`)
    /// Storage: `Tokens::Accounts` (r:32 w:32)
    /// Proof: `Tokens::Accounts` (`max_values`: None, `max_size`: Some(137), added: 2612, mode: `MaxEncodedLen`)
    /// Storage: `Tokens::TotalIssuance` (r:32 w:32)
    /// Proof: `Tokens::TotalIssuance` (`max_values`: None, `max_size`: Some(57), added: 2532, mode: `MaxEncodedLen`)
    /// The range of component `n` is `[2, 32]`.
    fn split_position_vertical_sans_parent(_n: u32, ) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `441`
        //  Estimated: `84574`
        // Minimum execution time: 1_923_000 nanoseconds.
        Weight::from_parts(29_365_000_000, 84574)
            .saturating_add(T::DbWeight::get().reads(66))
            .saturating_add(T::DbWeight::get().writes(65))
    }
    /// Storage: `MarketCommons::Markets` (r:1 w:0)
    /// Proof: `MarketCommons::Markets` (`max_values`: None, `max_size`: Some(708), added: 3183, mode: `MaxEncodedLen`)
    /// Storage: `Tokens::Accounts` (r:33 w:33)
    /// Proof: `Tokens::Accounts` (`max_values`: None, `max_size`: Some(137), added: 2612, mode: `MaxEncodedLen`)
    /// Storage: `Tokens::TotalIssuance` (r:33 w:33)
    /// Proof: `Tokens::TotalIssuance` (`max_values`: None, `max_size`: Some(57), added: 2532, mode: `MaxEncodedLen`)
    /// The range of component `n` is `[2, 32]`.
    fn split_position_vertical_with_parent(_n: u32, ) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `671`
        //  Estimated: `87186`
        // Minimum execution time: 2_353_000 nanoseconds.
        Weight::from_parts(37_193_000_000, 87186)
            .saturating_add(T::DbWeight::get().reads(67))
            .saturating_add(T::DbWeight::get().writes(66))
    }
    /// Storage: `MarketCommons::Markets` (r:1 w:0)
    /// Proof: `MarketCommons::Markets` (`max_values`: None, `max_size`: Some(708), added: 3183, mode: `MaxEncodedLen`)
    /// Storage: `Tokens::Accounts` (r:33 w:33)
    /// Proof: `Tokens::Accounts` (`max_values`: None, `max_size`: Some(137), added: 2612, mode: `MaxEncodedLen`)
    /// Storage: `Tokens::TotalIssuance` (r:33 w:33)
    /// Proof: `Tokens::TotalIssuance` (`max_values`: None, `max_size`: Some(57), added: 2532, mode: `MaxEncodedLen`)
    /// The range of component `n` is `[2, 32]`.
    fn split_position_horizontal(_n: u32, ) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `633`
        //  Estimated: `87186`
        // Minimum execution time: 2_773_000 nanoseconds.
        Weight::from_parts(30_303_000_000, 87186)
            .saturating_add(T::DbWeight::get().reads(67))
            .saturating_add(T::DbWeight::get().writes(66))
    }
    /// Storage: `MarketCommons::Markets` (r:1 w:0)
    /// Proof: `MarketCommons::Markets` (`max_values`: None, `max_size`: Some(708), added: 3183, mode: `MaxEncodedLen`)
    /// Storage: `Tokens::Accounts` (r:32 w:32)
    /// Proof: `Tokens::Accounts` (`max_values`: None, `max_size`: Some(137), added: 2612, mode: `MaxEncodedLen`)
    /// Storage: `Tokens::TotalIssuance` (r:32 w:32)
    /// Proof: `Tokens::TotalIssuance` (`max_values`: None, `max_size`: Some(57), added: 2532, mode: `MaxEncodedLen`)
    /// Storage: `System::Account` (r:1 w:1)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(132), added: 2607, mode: `MaxEncodedLen`)
    /// The range of component `n` is `[2, 32]`.
    fn merge_position_vertical_sans_parent(_n: u32, ) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `624 + n * (160 ±0)`
        //  Estimated: `84574`
        // Minimum execution time: 1_889_000 nanoseconds.
        Weight::from_parts(29_394_000_000, 84574)
            .saturating_add(T::DbWeight::get().reads(66))
            .saturating_add(T::DbWeight::get().writes(65))
    }
    /// Storage: `MarketCommons::Markets` (r:1 w:0)
    /// Proof: `MarketCommons::Markets` (`max_values`: None, `max_size`: Some(708), added: 3183, mode: `MaxEncodedLen`)
    /// Storage: `Tokens::Accounts` (r:33 w:33)
    /// Proof: `Tokens::Accounts` (`max_values`: None, `max_size`: Some(137), added: 2612, mode: `MaxEncodedLen`)
    /// Storage: `Tokens::TotalIssuance` (r:33 w:33)
    /// Proof: `Tokens::TotalIssuance` (`max_values`: None, `max_size`: Some(57), added: 2532, mode: `MaxEncodedLen`)
    /// The range of component `n` is `[2, 32]`.
    fn merge_position_vertical_with_parent(_n: u32, ) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `518 + n * (160 ±0)`
        //  Estimated: `87186`
        // Minimum execution time: 2_376_000 nanoseconds.
        Weight::from_parts(37_564_000_000, 87186)
            .saturating_add(T::DbWeight::get().reads(67))
            .saturating_add(T::DbWeight::get().writes(66))
    }
    /// Storage: `MarketCommons::Markets` (r:1 w:0)
    /// Proof: `MarketCommons::Markets` (`max_values`: None, `max_size`: Some(708), added: 3183, mode: `MaxEncodedLen`)
    /// Storage: `Tokens::Accounts` (r:33 w:33)
    /// Proof: `Tokens::Accounts` (`max_values`: None, `max_size`: Some(137), added: 2612, mode: `MaxEncodedLen`)
    /// Storage: `Tokens::TotalIssuance` (r:33 w:33)
    /// Proof: `Tokens::TotalIssuance` (`max_values`: None, `max_size`: Some(57), added: 2532, mode: `MaxEncodedLen`)
    /// The range of component `n` is `[2, 32]`.
    fn merge_position_horizontal(_n: u32, ) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `480 + n * (160 ±0)`
        //  Estimated: `87186`
        // Minimum execution time: 2_760_000 nanoseconds.
        Weight::from_parts(30_589_000_000, 87186)
            .saturating_add(T::DbWeight::get().reads(67))
            .saturating_add(T::DbWeight::get().writes(66))
    }
    /// Storage: `MarketCommons::Markets` (r:1 w:0)
    /// Proof: `MarketCommons::Markets` (`max_values`: None, `max_size`: Some(708), added: 3183, mode: `MaxEncodedLen`)
    /// Storage: `Tokens::Accounts` (r:1 w:1)
    /// Proof: `Tokens::Accounts` (`max_values`: None, `max_size`: Some(137), added: 2612, mode: `MaxEncodedLen`)
    /// Storage: `Tokens::TotalIssuance` (r:1 w:1)
    /// Proof: `Tokens::TotalIssuance` (`max_values`: None, `max_size`: Some(57), added: 2532, mode: `MaxEncodedLen`)
    /// Storage: `System::Account` (r:1 w:1)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(132), added: 2607, mode: `MaxEncodedLen`)
    /// The range of component `n` is `[2, 32]`.
    fn redeem_position_sans_parent(_n: u32, ) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `780`
        //  Estimated: `4173`
        // Minimum execution time: 979_000 nanoseconds.
        Weight::from_parts(986_000_000, 4173)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    /// Storage: `MarketCommons::Markets` (r:1 w:0)
    /// Proof: `MarketCommons::Markets` (`max_values`: None, `max_size`: Some(708), added: 3183, mode: `MaxEncodedLen`)
    /// Storage: `Tokens::Accounts` (r:2 w:2)
    /// Proof: `Tokens::Accounts` (`max_values`: None, `max_size`: Some(137), added: 2612, mode: `MaxEncodedLen`)
    /// Storage: `Tokens::TotalIssuance` (r:2 w:2)
    /// Proof: `Tokens::TotalIssuance` (`max_values`: None, `max_size`: Some(57), added: 2532, mode: `MaxEncodedLen`)
    /// The range of component `n` is `[2, 32]`.
    fn redeem_position_with_parent(_n: u32, ) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `674`
        //  Estimated: `6214`
        // Minimum execution time: 1_193_000 nanoseconds.
        Weight::from_parts(1_215_000_000, 6214)
            .saturating_add(T::DbWeight::get().reads(5))
            .saturating_add(T::DbWeight::get().writes(4))
    }
}
