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
//! DATE: 2023-04-24, STEPS: `10`, REPEAT: 1000, LOW RANGE: `[]`, HIGH RANGE: `[]`
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
    fn reassign_juror_stakes(d: u32) -> Weight;
    fn set_inflation() -> Weight;
    fn handle_inflation(j: u32) -> Weight;
    fn select_jurors(a: u32) -> Weight;
    fn on_dispute(j: u32, r: u32) -> Weight;
    fn on_resolution(j: u32, d: u32) -> Weight;
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
        Weight::from_ref_time(33_635_000)
            // Standard Error: 0
            .saturating_add(Weight::from_ref_time(98_000).saturating_mul(j.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    // Storage: Court JurorPool (r:1 w:1)
    // Storage: Court Jurors (r:6 w:1)
    // Storage: Balances Locks (r:1 w:1)
    fn delegate(j: u32, d: u32) -> Weight {
        Weight::from_ref_time(48_104_000)
            // Standard Error: 0
            .saturating_add(Weight::from_ref_time(122_000).saturating_mul(j.into()))
            // Standard Error: 45_000
            .saturating_add(Weight::from_ref_time(447_000).saturating_mul(d.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(d.into())))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    // Storage: Court Jurors (r:1 w:1)
    // Storage: Court JurorPool (r:1 w:1)
    fn prepare_exit_court(j: u32) -> Weight {
        Weight::from_ref_time(19_274_000)
            // Standard Error: 0
            .saturating_add(Weight::from_ref_time(84_000).saturating_mul(j.into()))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    // Storage: Court Jurors (r:1 w:1)
    // Storage: Balances Locks (r:1 w:1)
    fn exit_court_remove() -> Weight {
        Weight::from_ref_time(39_000_000)
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
        Weight::from_ref_time(49_081_000)
            // Standard Error: 0
            .saturating_add(Weight::from_ref_time(106_000).saturating_mul(d.into()))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    // Storage: MarketCommons Markets (r:1 w:0)
    // Storage: Court Jurors (r:1 w:0)
    // Storage: Court Courts (r:1 w:0)
    // Storage: Court SelectedDraws (r:1 w:1)
    fn denounce_vote(d: u32) -> Weight {
        Weight::from_ref_time(39_078_000)
            // Standard Error: 0
            .saturating_add(Weight::from_ref_time(124_000).saturating_mul(d.into()))
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    // Storage: MarketCommons Markets (r:1 w:0)
    // Storage: Court Jurors (r:1 w:0)
    // Storage: Court Courts (r:1 w:0)
    // Storage: Court SelectedDraws (r:1 w:1)
    fn reveal_vote(d: u32) -> Weight {
        Weight::from_ref_time(66_728_000)
            // Standard Error: 0
            .saturating_add(Weight::from_ref_time(91_000).saturating_mul(d.into()))
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    // Storage: Court Courts (r:1 w:1)
    // Storage: MarketCommons Markets (r:1 w:0)
    // Storage: Court SelectedDraws (r:1 w:1)
    // Storage: Court JurorPool (r:1 w:1)
    // Storage: Court JurorsSelectionNonce (r:1 w:1)
    // Storage: RandomnessCollectiveFlip RandomMaterial (r:1 w:0)
    // Storage: Court Jurors (r:42 w:41)
    // Storage: Court RequestBlock (r:1 w:0)
    // Storage: PredictionMarkets MarketIdsPerDisputeBlock (r:2 w:2)
    // Storage: Balances Reserves (r:1 w:1)
    fn appeal(j: u32, a: u32, _r: u32, _f: u32) -> Weight {
        Weight::from_ref_time(0)
            // Standard Error: 3_000
            .saturating_add(Weight::from_ref_time(1_390_000).saturating_mul(j.into()))
            // Standard Error: 1_348_000
            .saturating_add(Weight::from_ref_time(630_362_000).saturating_mul(a.into()))
            .saturating_add(T::DbWeight::get().reads(13))
            .saturating_add(T::DbWeight::get().reads((28_u64).saturating_mul(a.into())))
            .saturating_add(T::DbWeight::get().writes(8))
            .saturating_add(T::DbWeight::get().writes((28_u64).saturating_mul(a.into())))
    }
    // Storage: Court Courts (r:1 w:1)
    // Storage: Court SelectedDraws (r:1 w:1)
    // Storage: Court Jurors (r:5 w:5)
    // Storage: System Account (r:6 w:5)
    fn reassign_juror_stakes(d: u32) -> Weight {
        Weight::from_ref_time(0)
            // Standard Error: 8_000
            .saturating_add(Weight::from_ref_time(41_797_000).saturating_mul(d.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().reads((2_u64).saturating_mul(d.into())))
            .saturating_add(T::DbWeight::get().writes(2))
            .saturating_add(T::DbWeight::get().writes((2_u64).saturating_mul(d.into())))
    }
    // Storage: Court YearlyInflation (r:0 w:1)
    fn set_inflation() -> Weight {
        Weight::from_ref_time(6_000_000).saturating_add(T::DbWeight::get().writes(1))
    }
    // Storage: Court YearlyInflation (r:1 w:0)
    // Storage: Court JurorPool (r:1 w:0)
    // Storage: System Account (r:1 w:1)
    fn handle_inflation(j: u32) -> Weight {
        Weight::from_ref_time(0)
            // Standard Error: 25_000
            .saturating_add(Weight::from_ref_time(14_531_000).saturating_mul(j.into()))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(j.into())))
            .saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(j.into())))
    }
    // Storage: Court JurorPool (r:1 w:1)
    // Storage: Court JurorsSelectionNonce (r:1 w:1)
    // Storage: RandomnessCollectiveFlip RandomMaterial (r:1 w:0)
    // Storage: Court Jurors (r:9 w:5)
    fn select_jurors(a: u32) -> Weight {
        Weight::from_ref_time(226_140_000)
            // Standard Error: 2_247_000
            .saturating_add(Weight::from_ref_time(507_706_000).saturating_mul(a.into()))
            .saturating_add(T::DbWeight::get().reads(9))
            .saturating_add(T::DbWeight::get().reads((12_u64).saturating_mul(a.into())))
            .saturating_add(T::DbWeight::get().writes(4))
            .saturating_add(T::DbWeight::get().writes((12_u64).saturating_mul(a.into())))
    }
    // Storage: Court Courts (r:1 w:1)
    // Storage: Court JurorPool (r:1 w:1)
    // Storage: Court JurorsSelectionNonce (r:1 w:1)
    // Storage: RandomnessCollectiveFlip RandomMaterial (r:1 w:0)
    // Storage: Court Jurors (r:4 w:4)
    // Storage: Court RequestBlock (r:1 w:0)
    // Storage: PredictionMarkets MarketIdsPerDisputeBlock (r:1 w:1)
    // Storage: Court SelectedDraws (r:0 w:1)
    fn on_dispute(j: u32, r: u32) -> Weight {
        Weight::from_ref_time(65_336_000)
            // Standard Error: 0
            .saturating_add(Weight::from_ref_time(164_000).saturating_mul(j.into()))
            // Standard Error: 10_000
            .saturating_add(Weight::from_ref_time(185_000).saturating_mul(r.into()))
            .saturating_add(T::DbWeight::get().reads(11))
            .saturating_add(T::DbWeight::get().writes(10))
    }
    // Storage: Court Courts (r:1 w:1)
    // Storage: Court SelectedDraws (r:1 w:0)
    // Storage: MarketCommons Markets (r:1 w:0)
    // Storage: Court Jurors (r:94 w:94)
    fn on_resolution(j: u32, d: u32) -> Weight {
        Weight::from_ref_time(0)
            // Standard Error: 0
            .saturating_add(Weight::from_ref_time(99_000).saturating_mul(j.into()))
            // Standard Error: 3_000
            .saturating_add(Weight::from_ref_time(4_737_000).saturating_mul(d.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(d.into())))
            .saturating_add(T::DbWeight::get().writes(1))
            .saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(d.into())))
    }
    // Storage: Court Courts (r:1 w:0)
    // Storage: Balances Reserves (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    fn exchange(a: u32) -> Weight {
        Weight::from_ref_time(13_802_000)
            // Standard Error: 30_000
            .saturating_add(Weight::from_ref_time(21_667_000).saturating_mul(a.into()))
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().reads((2_u64).saturating_mul(a.into())))
            .saturating_add(T::DbWeight::get().writes((2_u64).saturating_mul(a.into())))
    }
    // Storage: Court Courts (r:1 w:0)
    fn get_auto_resolve() -> Weight {
        Weight::from_ref_time(7_000_000).saturating_add(T::DbWeight::get().reads(1))
    }
    // Storage: Court JurorPool (r:1 w:0)
    // Storage: Court Courts (r:1 w:0)
    // Storage: MarketCommons Markets (r:1 w:0)
    fn has_failed() -> Weight {
        Weight::from_ref_time(17_000_000).saturating_add(T::DbWeight::get().reads(3))
    }
    // Storage: Court Courts (r:1 w:1)
    // Storage: Court SelectedDraws (r:1 w:1)
    // Storage: Court Jurors (r:94 w:94)
    fn on_global_dispute(_a: u32, d: u32) -> Weight {
        Weight::from_ref_time(12_713_000)
            // Standard Error: 1_000
            .saturating_add(Weight::from_ref_time(4_069_000).saturating_mul(d.into()))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(d.into())))
            .saturating_add(T::DbWeight::get().writes(2))
            .saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(d.into())))
    }
    // Storage: Court SelectedDraws (r:1 w:1)
    // Storage: Court Jurors (r:1 w:1)
    // Storage: Court Courts (r:0 w:1)
    fn clear(d: u32) -> Weight {
        Weight::from_ref_time(9_761_000)
            // Standard Error: 1_000
            .saturating_add(Weight::from_ref_time(4_033_000).saturating_mul(d.into()))
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(d.into())))
            .saturating_add(T::DbWeight::get().writes(2))
            .saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(d.into())))
    }
}
