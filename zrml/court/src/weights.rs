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

//! Autogenerated weights for zrml_court
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-05-11, STEPS: `10`, REPEAT: 1000, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/zeitgeist
// benchmark
// pallet
// --chain=dev
// --steps=10
// --repeat=1000
// --pallet=zrml_court
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./zrml/court/src/weights.rs
// --template=./misc/weight_template.hbs

#![allow(unused_parens)]
#![allow(unused_imports)]

use core::marker::PhantomData;
use frame_support::{traits::Get, weights::Weight};

///  Trait containing the required functions for weight retrival within
/// zrml_court (automatically generated)
pub trait WeightInfoZeitgeist {
    fn join_court(j: u32) -> Weight;
    fn delegate(j: u32, d: u32) -> Weight;
    fn prepare_exit_court(j: u32) -> Weight;
    fn exit_court_remove() -> Weight;
    fn exit_court_set() -> Weight;
    fn vote(d: u32) -> Weight;
    fn denounce_vote(d: u32) -> Weight;
    fn reveal_vote(d: u32) -> Weight;
    fn appeal(j: u32, a: u32, r: u32, f: u32) -> Weight;
    fn reassign_court_stakes(d: u32) -> Weight;
    fn set_inflation() -> Weight;
    fn handle_inflation(j: u32) -> Weight;
    fn select_jurors(a: u32) -> Weight;
    fn on_dispute(j: u32, r: u32) -> Weight;
    fn on_resolution(d: u32) -> Weight;
    fn exchange(a: u32) -> Weight;
    fn get_auto_resolve() -> Weight;
    fn has_failed() -> Weight;
    fn on_global_dispute(a: u32, d: u32) -> Weight;
    fn clear(d: u32) -> Weight;
}

/// Weight functions for zrml_court (automatically generated)
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfoZeitgeist for WeightInfo<T> {
    // Storage: Court JurorPool (r:1 w:1)
    // Storage: Court Jurors (r:1 w:1)
    // Storage: Balances Locks (r:1 w:1)
    fn join_court(j: u32) -> Weight {
        Weight::from_ref_time(33_951_000)
            // Standard Error: 0
            .saturating_add(Weight::from_ref_time(94_000).saturating_mul(j.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    // Storage: Court JurorPool (r:1 w:1)
    // Storage: Court Jurors (r:6 w:1)
    // Storage: Balances Locks (r:1 w:1)
    fn delegate(j: u32, d: u32) -> Weight {
        Weight::from_ref_time(46_155_000)
            // Standard Error: 0
            .saturating_add(Weight::from_ref_time(122_000).saturating_mul(j.into()))
            // Standard Error: 51_000
            .saturating_add(Weight::from_ref_time(863_000).saturating_mul(d.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(d.into())))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    // Storage: Court Jurors (r:1 w:1)
    // Storage: Court JurorPool (r:1 w:1)
    fn prepare_exit_court(j: u32) -> Weight {
        Weight::from_ref_time(19_325_000)
            // Standard Error: 0
            .saturating_add(Weight::from_ref_time(84_000).saturating_mul(j.into()))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    // Storage: Court Jurors (r:1 w:1)
    // Storage: Balances Locks (r:1 w:1)
    fn exit_court_remove() -> Weight {
        Weight::from_ref_time(38_000_000)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    // Storage: Court Jurors (r:1 w:1)
    // Storage: Balances Locks (r:1 w:1)
    fn exit_court_set() -> Weight {
        Weight::from_ref_time(37_000_000)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    // Storage: Court Courts (r:1 w:0)
    // Storage: Court SelectedDraws (r:1 w:1)
    fn vote(d: u32) -> Weight {
        Weight::from_ref_time(48_629_000)
            // Standard Error: 0
            .saturating_add(Weight::from_ref_time(90_000).saturating_mul(d.into()))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    // Storage: Court CourtIdToMarketId (r:1 w:0)
    // Storage: MarketCommons Markets (r:1 w:0)
    // Storage: Court Jurors (r:1 w:0)
    // Storage: Court Courts (r:1 w:0)
    // Storage: Court SelectedDraws (r:1 w:1)
    fn denounce_vote(d: u32) -> Weight {
        Weight::from_ref_time(41_779_000)
            // Standard Error: 0
            .saturating_add(Weight::from_ref_time(126_000).saturating_mul(d.into()))
            .saturating_add(T::DbWeight::get().reads(5))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    // Storage: Court CourtIdToMarketId (r:1 w:0)
    // Storage: MarketCommons Markets (r:1 w:0)
    // Storage: Court Jurors (r:1 w:0)
    // Storage: Court Courts (r:1 w:0)
    // Storage: Court SelectedDraws (r:1 w:1)
    fn reveal_vote(d: u32) -> Weight {
        Weight::from_ref_time(69_471_000)
            // Standard Error: 0
            .saturating_add(Weight::from_ref_time(92_000).saturating_mul(d.into()))
            .saturating_add(T::DbWeight::get().reads(5))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    // Storage: Court Courts (r:1 w:1)
    // Storage: Court CourtIdToMarketId (r:1 w:0)
    // Storage: MarketCommons Markets (r:1 w:0)
    // Storage: Court SelectedDraws (r:1 w:1)
    // Storage: Court JurorPool (r:1 w:1)
    // Storage: Court JurorsSelectionNonce (r:1 w:1)
    // Storage: RandomnessCollectiveFlip RandomMaterial (r:1 w:0)
    // Storage: Court Jurors (r:223 w:222)
    // Storage: Court RequestBlock (r:1 w:0)
    // Storage: PredictionMarkets MarketIdsPerDisputeBlock (r:2 w:2)
    // Storage: Balances Reserves (r:1 w:1)
    fn appeal(j: u32, a: u32, r: u32, _f: u32) -> Weight {
        Weight::from_ref_time(0)
            // Standard Error: 26_000
            .saturating_add(Weight::from_ref_time(5_584_000).saturating_mul(j.into()))
            // Standard Error: 7_923_000
            .saturating_add(Weight::from_ref_time(2_539_125_000).saturating_mul(a.into()))
            // Standard Error: 320_000
            .saturating_add(Weight::from_ref_time(1_503_000).saturating_mul(r.into()))
            .saturating_add(T::DbWeight::get().reads((128_u64).saturating_mul(a.into())))
            .saturating_add(T::DbWeight::get().writes((128_u64).saturating_mul(a.into())))
    }
    // Storage: Court Courts (r:1 w:1)
    // Storage: Court SelectedDraws (r:1 w:1)
    // Storage: Court Jurors (r:5 w:5)
    // Storage: System Account (r:6 w:5)
    fn reassign_court_stakes(d: u32) -> Weight {
        Weight::from_ref_time(0)
            // Standard Error: 19_000
            .saturating_add(Weight::from_ref_time(44_416_000).saturating_mul(d.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().reads((2_u64).saturating_mul(d.into())))
            .saturating_add(T::DbWeight::get().writes(2))
            .saturating_add(T::DbWeight::get().writes((2_u64).saturating_mul(d.into())))
    }
    // Storage: Court YearlyInflation (r:0 w:1)
    fn set_inflation() -> Weight {
        Weight::from_ref_time(16_000_000).saturating_add(T::DbWeight::get().writes(1))
    }
    // Storage: Court YearlyInflation (r:1 w:0)
    // Storage: Court JurorPool (r:1 w:0)
    // Storage: System Account (r:1 w:1)
    fn handle_inflation(j: u32) -> Weight {
        Weight::from_ref_time(0)
            // Standard Error: 4_000
            .saturating_add(Weight::from_ref_time(12_853_000).saturating_mul(j.into()))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(j.into())))
            .saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(j.into())))
    }
    // Storage: Court CourtPool (r:1 w:1)
    // Storage: Court SelectionNonce (r:1 w:1)
    // Storage: RandomnessCollectiveFlip RandomMaterial (r:1 w:0)
    // Storage: Court Participants (r:35 w:31)
    fn select_jurors(a: u32, ) -> Weight {
        Weight::from_ref_time(639_560_000)
            // Standard Error: 11_776_000
            .saturating_add(Weight::from_ref_time(2_310_239_000).saturating_mul(a.into()))
            .saturating_add(T::DbWeight::get().reads(24))
            .saturating_add(T::DbWeight::get().reads((60_u64).saturating_mul(a.into())))
            .saturating_add(T::DbWeight::get().writes(19))
            .saturating_add(T::DbWeight::get().writes((60_u64).saturating_mul(a.into())))
    }
    // Storage: Court NextCourtId (r:1 w:1)
    // Storage: Court JurorPool (r:1 w:1)
    // Storage: Court JurorsSelectionNonce (r:1 w:1)
    // Storage: RandomnessCollectiveFlip RandomMaterial (r:1 w:0)
    // Storage: Court Jurors (r:23 w:23)
    // Storage: Court RequestBlock (r:1 w:0)
    // Storage: PredictionMarkets MarketIdsPerDisputeBlock (r:1 w:1)
    // Storage: Court SelectedDraws (r:0 w:1)
    // Storage: Court CourtIdToMarketId (r:0 w:1)
    // Storage: Court MarketIdToCourtId (r:0 w:1)
    // Storage: Court Courts (r:0 w:1)
    fn on_dispute(j: u32, r: u32) -> Weight {
        Weight::from_ref_time(196_514_000)
            // Standard Error: 0
            .saturating_add(Weight::from_ref_time(144_000).saturating_mul(j.into()))
            // Standard Error: 3_000
            .saturating_add(Weight::from_ref_time(157_000).saturating_mul(r.into()))
            .saturating_add(T::DbWeight::get().reads(33))
            .saturating_add(T::DbWeight::get().writes(35))
    }
    // Storage: Court MarketIdToCourtId (r:1 w:0)
    // Storage: Court Courts (r:1 w:1)
    // Storage: Court SelectedDraws (r:1 w:0)
    // Storage: Court CourtIdToMarketId (r:1 w:0)
    // Storage: MarketCommons Markets (r:1 w:0)
    // Storage: Court Jurors (r:1 w:1)
    fn on_resolution(d: u32) -> Weight {
        Weight::from_ref_time(17_329_000)
            // Standard Error: 1_000
            .saturating_add(Weight::from_ref_time(4_102_000).saturating_mul(d.into()))
            .saturating_add(T::DbWeight::get().reads(5))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(d.into())))
            .saturating_add(T::DbWeight::get().writes(1))
            .saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(d.into())))
    }
    // Storage: Court MarketIdToCourtId (r:1 w:0)
    // Storage: Court Courts (r:1 w:0)
    // Storage: Balances Reserves (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    fn exchange(a: u32) -> Weight {
        Weight::from_ref_time(17_021_000)
            // Standard Error: 29_000
            .saturating_add(Weight::from_ref_time(21_348_000).saturating_mul(a.into()))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().reads((2_u64).saturating_mul(a.into())))
            .saturating_add(T::DbWeight::get().writes((2_u64).saturating_mul(a.into())))
    }
    // Storage: Court MarketIdToCourtId (r:1 w:0)
    // Storage: Court Courts (r:1 w:0)
    fn get_auto_resolve() -> Weight {
        Weight::from_ref_time(9_000_000).saturating_add(T::DbWeight::get().reads(2))
    }
    // Storage: Court MarketIdToCourtId (r:1 w:0)
    // Storage: Court JurorPool (r:1 w:0)
    // Storage: Court Courts (r:1 w:0)
    // Storage: Court CourtIdToMarketId (r:1 w:0)
    // Storage: MarketCommons Markets (r:1 w:0)
    fn has_failed() -> Weight {
        Weight::from_ref_time(24_000_000).saturating_add(T::DbWeight::get().reads(5))
    }
    // Storage: Court MarketIdToCourtId (r:1 w:0)
    // Storage: Court Courts (r:1 w:1)
    // Storage: Court SelectedDraws (r:1 w:1)
    // Storage: Court Jurors (r:510 w:510)
    fn on_global_dispute(a: u32, d: u32) -> Weight {
        Weight::from_ref_time(11_646_000)
            // Standard Error: 588_000
            .saturating_add(Weight::from_ref_time(20_187_000).saturating_mul(a.into()))
            // Standard Error: 5_000
            .saturating_add(Weight::from_ref_time(4_083_000).saturating_mul(d.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(d.into())))
            .saturating_add(T::DbWeight::get().writes(2))
            .saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(d.into())))
    }
    // Storage: Court MarketIdToCourtId (r:1 w:0)
    // Storage: Court SelectedDraws (r:1 w:1)
    // Storage: Court Jurors (r:1 w:1)
    // Storage: Court Courts (r:0 w:1)
    fn clear(d: u32) -> Weight {
        Weight::from_ref_time(4_229_000)
            // Standard Error: 0
            .saturating_add(Weight::from_ref_time(4_115_000).saturating_mul(d.into()))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(d.into())))
            .saturating_add(T::DbWeight::get().writes(2))
            .saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(d.into())))
    }
}
