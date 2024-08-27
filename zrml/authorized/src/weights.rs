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

//! Autogenerated weights for zrml_authorized
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: `2024-08-12`, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `zeitgeist-benchmark`, CPU: `AMD EPYC 7601 32-Core Processor`
//! EXECUTION: ``, WASM-EXECUTION: `Compiled`, CHAIN: `Some("dev")`, DB CACHE: `1024`

// Executed Command:
// ./target/production/zeitgeist
// benchmark
// pallet
// --chain=dev
// --steps=50
// --repeat=20
// --pallet=zrml_authorized
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --template=./misc/weight_template.hbs
// --header=./HEADER_GPL3
// --output=./zrml/authorized/src/weights.rs

#![allow(unused_parens)]
#![allow(unused_imports)]

use core::marker::PhantomData;
use frame_support::{traits::Get, weights::Weight};

///  Trait containing the required functions for weight retrival within
/// zrml_authorized (automatically generated)
pub trait WeightInfoZeitgeist {
    fn authorize_market_outcome_first_report(m: u32) -> Weight;
    fn authorize_market_outcome_existing_report() -> Weight;
    fn on_dispute_weight() -> Weight;
    fn on_resolution_weight() -> Weight;
    fn exchange_weight() -> Weight;
    fn get_auto_resolve_weight() -> Weight;
    fn has_failed_weight() -> Weight;
    fn on_global_dispute_weight() -> Weight;
    fn clear_weight() -> Weight;
}

/// Weight functions for zrml_authorized (automatically generated)
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfoZeitgeist for WeightInfo<T> {
    /// Storage: `MarketCommons::Markets` (r:1 w:0)
    /// Proof: `MarketCommons::Markets` (`max_values`: None, `max_size`: Some(694), added: 3169, mode: `MaxEncodedLen`)
    /// Storage: `Authorized::AuthorizedOutcomeReports` (r:1 w:1)
    /// Proof: `Authorized::AuthorizedOutcomeReports` (`max_values`: None, `max_size`: Some(49), added: 2524, mode: `MaxEncodedLen`)
    /// Storage: `PredictionMarkets::MarketIdsPerDisputeBlock` (r:1 w:1)
    /// Proof: `PredictionMarkets::MarketIdsPerDisputeBlock` (`max_values`: None, `max_size`: Some(1042), added: 3517, mode: `MaxEncodedLen`)
    /// The range of component `m` is `[1, 63]`.
    fn authorize_market_outcome_first_report(m: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `757 + m * (22 ±0)`
        //  Estimated: `4507`
        // Minimum execution time: 29_411 nanoseconds.
        Weight::from_parts(31_577_727, 4507)
            // Standard Error: 2_714
            .saturating_add(Weight::from_parts(89_742, 0).saturating_mul(m.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    /// Storage: `MarketCommons::Markets` (r:1 w:0)
    /// Proof: `MarketCommons::Markets` (`max_values`: None, `max_size`: Some(694), added: 3169, mode: `MaxEncodedLen`)
    /// Storage: `Authorized::AuthorizedOutcomeReports` (r:1 w:1)
    /// Proof: `Authorized::AuthorizedOutcomeReports` (`max_values`: None, `max_size`: Some(49), added: 2524, mode: `MaxEncodedLen`)
    fn authorize_market_outcome_existing_report() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `561`
        //  Estimated: `4159`
        // Minimum execution time: 24_361 nanoseconds.
        Weight::from_parts(26_040_000, 4159)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    fn on_dispute_weight() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 250 nanoseconds.
        Weight::from_parts(310_000, 0)
    }
    /// Storage: `Authorized::AuthorizedOutcomeReports` (r:1 w:1)
    /// Proof: `Authorized::AuthorizedOutcomeReports` (`max_values`: None, `max_size`: Some(49), added: 2524, mode: `MaxEncodedLen`)
    fn on_resolution_weight() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `217`
        //  Estimated: `3514`
        // Minimum execution time: 7_130 nanoseconds.
        Weight::from_parts(7_620_000, 3514)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    fn exchange_weight() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 2_110 nanoseconds.
        Weight::from_parts(2_370_000, 0)
    }
    /// Storage: `Authorized::AuthorizedOutcomeReports` (r:1 w:0)
    /// Proof: `Authorized::AuthorizedOutcomeReports` (`max_values`: None, `max_size`: Some(49), added: 2524, mode: `MaxEncodedLen`)
    fn get_auto_resolve_weight() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `217`
        //  Estimated: `3514`
        // Minimum execution time: 6_541 nanoseconds.
        Weight::from_parts(6_830_000, 3514).saturating_add(T::DbWeight::get().reads(1))
    }
    fn has_failed_weight() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 260 nanoseconds.
        Weight::from_parts(310_000, 0)
    }
    fn on_global_dispute_weight() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 270 nanoseconds.
        Weight::from_parts(340_000, 0)
    }
    /// Storage: `Authorized::AuthorizedOutcomeReports` (r:0 w:1)
    /// Proof: `Authorized::AuthorizedOutcomeReports` (`max_values`: None, `max_size`: Some(49), added: 2524, mode: `MaxEncodedLen`)
    fn clear_weight() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 1_800 nanoseconds.
        Weight::from_parts(1_930_000, 0).saturating_add(T::DbWeight::get().writes(1))
    }
}
