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

//! Autogenerated weights for zrml_parimutuel
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
// --pallet=zrml_parimutuel
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --template=./misc/weight_template.hbs
// --header=./HEADER_GPL3
// --output=./zrml/parimutuel/src/weights.rs

#![allow(unused_parens)]
#![allow(unused_imports)]

use core::marker::PhantomData;
use frame_support::{traits::Get, weights::Weight};

///  Trait containing the required functions for weight retrival within
/// zrml_parimutuel (automatically generated)
pub trait WeightInfoZeitgeist {
    fn buy() -> Weight;
    fn claim_rewards() -> Weight;
    fn claim_refunds() -> Weight;
}

/// Weight functions for zrml_parimutuel (automatically generated)
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfoZeitgeist for WeightInfo<T> {
    /// Storage: `MarketCommons::Markets` (r:1 w:0)
    /// Proof: `MarketCommons::Markets` (`max_values`: None, `max_size`: Some(708), added: 3183, mode: `MaxEncodedLen`)
    /// Storage: `System::Account` (r:1 w:1)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(132), added: 2607, mode: `MaxEncodedLen`)
    /// Storage: `Tokens::Accounts` (r:1 w:1)
    /// Proof: `Tokens::Accounts` (`max_values`: None, `max_size`: Some(137), added: 2612, mode: `MaxEncodedLen`)
    /// Storage: `Tokens::TotalIssuance` (r:1 w:1)
    /// Proof: `Tokens::TotalIssuance` (`max_values`: None, `max_size`: Some(57), added: 2532, mode: `MaxEncodedLen`)
    fn buy() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `390`
        //  Estimated: `4173`
        // Minimum execution time: 101_473 nanoseconds.
        Weight::from_parts(104_233_000, 4173)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    /// Storage: `MarketCommons::Markets` (r:1 w:0)
    /// Proof: `MarketCommons::Markets` (`max_values`: None, `max_size`: Some(708), added: 3183, mode: `MaxEncodedLen`)
    /// Storage: `Tokens::TotalIssuance` (r:1 w:1)
    /// Proof: `Tokens::TotalIssuance` (`max_values`: None, `max_size`: Some(57), added: 2532, mode: `MaxEncodedLen`)
    /// Storage: `Tokens::Accounts` (r:1 w:1)
    /// Proof: `Tokens::Accounts` (`max_values`: None, `max_size`: Some(137), added: 2612, mode: `MaxEncodedLen`)
    /// Storage: `System::Account` (r:1 w:1)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(132), added: 2607, mode: `MaxEncodedLen`)
    fn claim_rewards() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `790`
        //  Estimated: `4173`
        // Minimum execution time: 100_162 nanoseconds.
        Weight::from_parts(103_682_000, 4173)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    /// Storage: `MarketCommons::Markets` (r:1 w:0)
    /// Proof: `MarketCommons::Markets` (`max_values`: None, `max_size`: Some(708), added: 3183, mode: `MaxEncodedLen`)
    /// Storage: `Tokens::TotalIssuance` (r:2 w:1)
    /// Proof: `Tokens::TotalIssuance` (`max_values`: None, `max_size`: Some(57), added: 2532, mode: `MaxEncodedLen`)
    /// Storage: `Tokens::Accounts` (r:1 w:1)
    /// Proof: `Tokens::Accounts` (`max_values`: None, `max_size`: Some(137), added: 2612, mode: `MaxEncodedLen`)
    /// Storage: `System::Account` (r:1 w:1)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(132), added: 2607, mode: `MaxEncodedLen`)
    fn claim_refunds() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `790`
        //  Estimated: `6054`
        // Minimum execution time: 90_502 nanoseconds.
        Weight::from_parts(97_023_000, 6054)
            .saturating_add(T::DbWeight::get().reads(5))
            .saturating_add(T::DbWeight::get().writes(3))
    }
}
