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

//! Autogenerated weights for zrml_global_disputes
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-06-16, STEPS: `10`, REPEAT: 1000, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/production/zeitgeist
// benchmark
// pallet
// --chain=dev
// --steps=10
// --repeat=1000
// --pallet=zrml_global_disputes
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --template=./misc/weight_template.hbs
// --output=./zrml/global-disputes/src/weights.rs

#![allow(unused_parens)]
#![allow(unused_imports)]

use core::marker::PhantomData;
use frame_support::{traits::Get, weights::Weight};

///  Trait containing the required functions for weight retrival within
/// zrml_global_disputes (automatically generated)
pub trait WeightInfoZeitgeist {
    fn vote_on_outcome(o: u32, v: u32) -> Weight;
    fn unlock_vote_balance_set(l: u32, o: u32) -> Weight;
    fn unlock_vote_balance_remove(l: u32, o: u32) -> Weight;
    fn add_vote_outcome(w: u32) -> Weight;
    fn reward_outcome_owner_with_funds(o: u32) -> Weight;
    fn reward_outcome_owner_no_funds(o: u32) -> Weight;
    fn purge_outcomes(k: u32, o: u32) -> Weight;
}

/// Weight functions for zrml_global_disputes (automatically generated)
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfoZeitgeist for WeightInfo<T> {
    // Storage: GlobalDisputes Winners (r:1 w:1)
    // Storage: GlobalDisputes Outcomes (r:1 w:1)
    // Storage: GlobalDisputes Locks (r:1 w:1)
    // Storage: Balances Locks (r:1 w:1)
    fn vote_on_outcome(o: u32, v: u32) -> Weight {
        Weight::from_ref_time(78_692_161)
            // Standard Error: 15_119
            .saturating_add(Weight::from_ref_time(59_007).saturating_mul(o.into()))
            // Standard Error: 2_748
            .saturating_add(Weight::from_ref_time(87_375).saturating_mul(v.into()))
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(4))
    }
    // Storage: GlobalDisputes Locks (r:1 w:1)
    // Storage: Balances Locks (r:1 w:1)
    // Storage: System Account (r:1 w:0)
    // Storage: GlobalDisputes Winners (r:5 w:0)
    fn unlock_vote_balance_set(l: u32, o: u32) -> Weight {
        Weight::from_ref_time(47_121_043)
            // Standard Error: 10_506
            .saturating_add(Weight::from_ref_time(3_575_177).saturating_mul(l.into()))
            // Standard Error: 58_814
            .saturating_add(Weight::from_ref_time(1_025_080).saturating_mul(o.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(l.into())))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    // Storage: GlobalDisputes Locks (r:1 w:1)
    // Storage: Balances Locks (r:1 w:1)
    // Storage: System Account (r:1 w:0)
    // Storage: GlobalDisputes Winners (r:5 w:0)
    fn unlock_vote_balance_remove(l: u32, o: u32) -> Weight {
        Weight::from_ref_time(45_059_975)
            // Standard Error: 9_769
            .saturating_add(Weight::from_ref_time(3_392_794).saturating_mul(l.into()))
            // Standard Error: 54_688
            .saturating_add(Weight::from_ref_time(856_867).saturating_mul(o.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(l.into())))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    // Storage: GlobalDisputes Winners (r:1 w:1)
    // Storage: GlobalDisputes Outcomes (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    fn add_vote_outcome(w: u32) -> Weight {
        Weight::from_ref_time(85_081_228)
            // Standard Error: 23_929
            .saturating_add(Weight::from_ref_time(247_830).saturating_mul(w.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    // Storage: GlobalDisputes Outcomes (r:1 w:0)
    // Storage: GlobalDisputes Winners (r:1 w:0)
    // Storage: System Account (r:2 w:2)
    fn reward_outcome_owner_with_funds(o: u32) -> Weight {
        Weight::from_ref_time(69_114_969)
            // Standard Error: 102_841
            .saturating_add(Weight::from_ref_time(27_858_004).saturating_mul(o.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(o.into())))
            .saturating_add(T::DbWeight::get().writes(1))
            .saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(o.into())))
    }
    // Storage: GlobalDisputes Outcomes (r:1 w:0)
    // Storage: GlobalDisputes Winners (r:1 w:0)
    // Storage: System Account (r:1 w:0)
    fn reward_outcome_owner_no_funds(o: u32) -> Weight {
        Weight::from_ref_time(43_940_716)
            // Standard Error: 11_242
            .saturating_add(Weight::from_ref_time(322_069).saturating_mul(o.into()))
            .saturating_add(T::DbWeight::get().reads(3))
    }
    // Storage: GlobalDisputes Winners (r:1 w:1)
    // Storage: GlobalDisputes Outcomes (r:3 w:2)
    fn purge_outcomes(k: u32, _o: u32) -> Weight {
        Weight::from_ref_time(134_980_974)
            // Standard Error: 24_117
            .saturating_add(Weight::from_ref_time(17_002_999).saturating_mul(k.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(k.into())))
            .saturating_add(T::DbWeight::get().writes(2))
            .saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(k.into())))
    }
}
