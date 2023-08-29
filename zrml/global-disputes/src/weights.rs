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

//! Autogenerated weights for zrml_global_disputes
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-08-04, STEPS: `10`, REPEAT: 1000, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/zeitgeist
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
// --output=./zrml/global-disputes/src/weights.rs
// --template=./misc/weight_template.hbs

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
    fn reward_outcome_owner_shared_possession(o: u32) -> Weight;
    fn reward_outcome_owner_paid_possession() -> Weight;
    fn purge_outcomes(k: u32, o: u32) -> Weight;
    fn refund_vote_fees(k: u32, o: u32) -> Weight;
}

/// Weight functions for zrml_global_disputes (automatically generated)
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfoZeitgeist for WeightInfo<T> {
    // Storage: GlobalDisputes GlobalDisputesInfo (r:1 w:1)
    // Storage: GlobalDisputes Outcomes (r:1 w:1)
    // Storage: GlobalDisputes Locks (r:1 w:1)
    // Storage: Balances Locks (r:1 w:1)
    fn vote_on_outcome(_o: u32, v: u32) -> Weight {
        Weight::from_ref_time(89_222_217)
            // Standard Error: 4_589
            .saturating_add(Weight::from_ref_time(183_754).saturating_mul(v.into()))
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(4))
    }
    // Storage: GlobalDisputes Locks (r:1 w:1)
    // Storage: Balances Locks (r:1 w:1)
    // Storage: System Account (r:1 w:0)
    // Storage: GlobalDisputes GlobalDisputesInfo (r:5 w:0)
    fn unlock_vote_balance_set(l: u32, o: u32) -> Weight {
        Weight::from_ref_time(54_445_996)
            // Standard Error: 8_942
            .saturating_add(Weight::from_ref_time(4_333_876).saturating_mul(l.into()))
            // Standard Error: 50_061
            .saturating_add(Weight::from_ref_time(1_054_193).saturating_mul(o.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(l.into())))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    // Storage: GlobalDisputes Locks (r:1 w:1)
    // Storage: Balances Locks (r:1 w:1)
    // Storage: System Account (r:1 w:0)
    // Storage: GlobalDisputes GlobalDisputesInfo (r:5 w:0)
    fn unlock_vote_balance_remove(l: u32, o: u32) -> Weight {
        Weight::from_ref_time(61_165_913)
            // Standard Error: 9_374
            .saturating_add(Weight::from_ref_time(4_014_112).saturating_mul(l.into()))
            // Standard Error: 52_477
            .saturating_add(Weight::from_ref_time(199_810).saturating_mul(o.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(l.into())))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    // Storage: MarketCommons Markets (r:1 w:0)
    // Storage: GlobalDisputes GlobalDisputesInfo (r:1 w:1)
    // Storage: GlobalDisputes Outcomes (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    fn add_vote_outcome(_w: u32) -> Weight {
        Weight::from_ref_time(100_295_681)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    // Storage: GlobalDisputes Outcomes (r:1 w:0)
    // Storage: GlobalDisputes GlobalDisputesInfo (r:1 w:0)
    // Storage: System Account (r:2 w:2)
    fn reward_outcome_owner_shared_possession(o: u32) -> Weight {
        Weight::from_ref_time(36_741_000)
            // Standard Error: 20_000
            .saturating_add(Weight::from_ref_time(22_017_000).saturating_mul(o.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(o.into())))
            .saturating_add(T::DbWeight::get().writes(1))
            .saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(o.into())))
    }
    // Storage: GlobalDisputes Outcomes (r:1 w:0)
    // Storage: GlobalDisputes GlobalDisputesInfo (r:1 w:0)
    // Storage: System Account (r:2 w:2)
    fn reward_outcome_owner_paid_possession() -> Weight {
        Weight::from_ref_time(56_000_000)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    // Storage: GlobalDisputes GlobalDisputesInfo (r:1 w:1)
    // Storage: GlobalDisputes Outcomes (r:3 w:2)
    fn purge_outcomes(k: u32, _o: u32) -> Weight {
        Weight::from_ref_time(168_932_238)
            // Standard Error: 26_203
            .saturating_add(Weight::from_ref_time(19_445_586).saturating_mul(k.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(k.into())))
            .saturating_add(T::DbWeight::get().writes(2))
            .saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(k.into())))
    }
    // Storage: GlobalDisputes GlobalDisputesInfo (r:1 w:0)
    // Storage: GlobalDisputes Outcomes (r:3 w:2)
    fn refund_vote_fees(k: u32, _o: u32) -> Weight {
        Weight::from_ref_time(31_076_000)
            // Standard Error: 4_000
            .saturating_add(Weight::from_ref_time(13_543_000).saturating_mul(k.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(k.into())))
            .saturating_add(T::DbWeight::get().writes(1))
            .saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(k.into())))
    }
}
