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
//! DATE: `2024-01-15`, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `zafoi`, CPU: `AMD Ryzen 9 5900X 12-Core Processor`
//! EXECUTION: `Some(Native)`, WASM-EXECUTION: `Compiled`, CHAIN: `Some("dev")`, DB CACHE: `1024`

// Executed Command:
// ./target/release/zeitgeist
// benchmark
// pallet
// --chain=dev
// --steps=50
// --repeat=20
// --pallet=zrml_authorized
// --extrinsic=*
// --execution=native
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
    /// Storage: MarketCommons Markets (r:1 w:0)
    /// Proof: MarketCommons Markets (max_values: None, max_size: Some(678), added: 3153, mode: MaxEncodedLen)
    /// Storage: Authorized AuthorizedOutcomeReports (r:1 w:1)
    /// Proof: Authorized AuthorizedOutcomeReports (max_values: None, max_size: Some(49), added: 2524, mode: MaxEncodedLen)
    /// Storage: PredictionMarkets MarketIdsPerDisputeBlock (r:1 w:1)
    /// Proof: PredictionMarkets MarketIdsPerDisputeBlock (max_values: None, max_size: Some(1042), added: 3517, mode: MaxEncodedLen)
    /// The range of component `m` is `[1, 63]`.
    fn authorize_market_outcome_first_report(m: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `805 + m * (22 ±0)`
        //  Estimated: `9194`
        // Minimum execution time: 40_600 nanoseconds.
        Weight::from_parts(46_642_457, 9194)
            // Standard Error: 26_413
            .saturating_add(Weight::from_parts(190_563, 0).saturating_mul(m.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    /// Storage: MarketCommons Markets (r:1 w:0)
    /// Proof: MarketCommons Markets (max_values: None, max_size: Some(678), added: 3153, mode: MaxEncodedLen)
    /// Storage: Authorized AuthorizedOutcomeReports (r:1 w:1)
    /// Proof: Authorized AuthorizedOutcomeReports (max_values: None, max_size: Some(49), added: 2524, mode: MaxEncodedLen)
    fn authorize_market_outcome_existing_report() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `610`
        //  Estimated: `5677`
        // Minimum execution time: 35_410 nanoseconds.
        Weight::from_parts(36_251_000, 5677)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    fn on_dispute_weight() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 290 nanoseconds.
        Weight::from_parts(320_000, 0)
    }
    /// Storage: Authorized AuthorizedOutcomeReports (r:1 w:1)
    /// Proof: Authorized AuthorizedOutcomeReports (max_values: None, max_size: Some(49), added: 2524, mode: MaxEncodedLen)
    fn on_resolution_weight() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `250`
        //  Estimated: `2524`
        // Minimum execution time: 10_110 nanoseconds.
        Weight::from_parts(12_690_000, 2524)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    fn exchange_weight() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 2_680 nanoseconds.
        Weight::from_parts(2_980_000, 0)
    }
    /// Storage: Authorized AuthorizedOutcomeReports (r:1 w:0)
    /// Proof: Authorized AuthorizedOutcomeReports (max_values: None, max_size: Some(49), added: 2524, mode: MaxEncodedLen)
    fn get_auto_resolve_weight() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `250`
        //  Estimated: `2524`
        // Minimum execution time: 9_200 nanoseconds.
        Weight::from_parts(9_520_000, 2524).saturating_add(T::DbWeight::get().reads(1))
    }
    fn has_failed_weight() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 250 nanoseconds.
        Weight::from_parts(280_000, 0)
    }
    fn on_global_dispute_weight() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 270 nanoseconds.
        Weight::from_parts(310_000, 0)
    }
    /// Storage: Authorized AuthorizedOutcomeReports (r:0 w:1)
    /// Proof: Authorized AuthorizedOutcomeReports (max_values: None, max_size: Some(49), added: 2524, mode: MaxEncodedLen)
    fn clear_weight() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 2_390 nanoseconds.
        Weight::from_parts(2_970_000, 0).saturating_add(T::DbWeight::get().writes(1))
    }
}
