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

//! Autogenerated weights for pallet_membership
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-02-08, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/production/zeitgeist
// benchmark
// pallet
// --chain=dev
// --steps=50
// --repeat=20
// --pallet=pallet_membership
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --template=./misc/frame_weight_template.hbs
// --output=./runtime/common/src/weights/

#![allow(unused_parens)]
#![allow(unused_imports)]

use core::marker::PhantomData;
use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, Weight},
};

/// Weight functions for pallet_membership (automatically generated)
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_membership::weights::WeightInfo for WeightInfo<T> {
    // Storage: AdvisoryCommitteeMembership Members (r:1 w:1)
    // Storage: AdvisoryCommittee Proposals (r:1 w:0)
    // Storage: AdvisoryCommittee Members (r:0 w:1)
    // Storage: AdvisoryCommittee Prime (r:0 w:1)
    fn add_member(m: u32) -> Weight {
        Weight::from_ref_time(15_790_016)
            // Standard Error: 647
            .saturating_add(Weight::from_ref_time(31_177).saturating_mul(m.into()))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    // Storage: AdvisoryCommitteeMembership Members (r:1 w:1)
    // Storage: AdvisoryCommittee Proposals (r:1 w:0)
    // Storage: AdvisoryCommitteeMembership Prime (r:1 w:0)
    // Storage: AdvisoryCommittee Members (r:0 w:1)
    // Storage: AdvisoryCommittee Prime (r:0 w:1)
    fn remove_member(m: u32) -> Weight {
        Weight::from_ref_time(17_898_825)
            // Standard Error: 616
            .saturating_add(Weight::from_ref_time(27_596).saturating_mul(m.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    // Storage: AdvisoryCommitteeMembership Members (r:1 w:1)
    // Storage: AdvisoryCommittee Proposals (r:1 w:0)
    // Storage: AdvisoryCommitteeMembership Prime (r:1 w:0)
    // Storage: AdvisoryCommittee Members (r:0 w:1)
    // Storage: AdvisoryCommittee Prime (r:0 w:1)
    fn swap_member(m: u32) -> Weight {
        Weight::from_ref_time(17_795_029)
            // Standard Error: 628
            .saturating_add(Weight::from_ref_time(36_370).saturating_mul(m.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    // Storage: AdvisoryCommitteeMembership Members (r:1 w:1)
    // Storage: AdvisoryCommittee Proposals (r:1 w:0)
    // Storage: AdvisoryCommitteeMembership Prime (r:1 w:0)
    // Storage: AdvisoryCommittee Members (r:0 w:1)
    // Storage: AdvisoryCommittee Prime (r:0 w:1)
    fn reset_member(m: u32) -> Weight {
        Weight::from_ref_time(17_561_643)
            // Standard Error: 691
            .saturating_add(Weight::from_ref_time(104_626).saturating_mul(m.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    // Storage: AdvisoryCommitteeMembership Members (r:1 w:1)
    // Storage: AdvisoryCommittee Proposals (r:1 w:0)
    // Storage: AdvisoryCommitteeMembership Prime (r:1 w:1)
    // Storage: AdvisoryCommittee Members (r:0 w:1)
    // Storage: AdvisoryCommittee Prime (r:0 w:1)
    fn change_key(m: u32) -> Weight {
        Weight::from_ref_time(18_510_706)
            // Standard Error: 686
            .saturating_add(Weight::from_ref_time(34_672).saturating_mul(m.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(4))
    }
    // Storage: AdvisoryCommitteeMembership Members (r:1 w:0)
    // Storage: AdvisoryCommitteeMembership Prime (r:0 w:1)
    // Storage: AdvisoryCommittee Prime (r:0 w:1)
    fn set_prime(m: u32) -> Weight {
        Weight::from_ref_time(6_527_451)
            // Standard Error: 269
            .saturating_add(Weight::from_ref_time(10_485).saturating_mul(m.into()))
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    // Storage: AdvisoryCommitteeMembership Prime (r:0 w:1)
    // Storage: AdvisoryCommittee Prime (r:0 w:1)
    fn clear_prime(m: u32) -> Weight {
        Weight::from_ref_time(3_713_794)
            // Standard Error: 136
            .saturating_add(Weight::from_ref_time(834).saturating_mul(m.into()))
            .saturating_add(T::DbWeight::get().writes(2))
    }
}
