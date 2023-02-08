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

//! Autogenerated weights for pallet_bounties
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
// --pallet=pallet_bounties
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

/// Weight functions for pallet_bounties (automatically generated)
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_bounties::weights::WeightInfo for WeightInfo<T> {
    // Storage: Bounties BountyCount (r:1 w:1)
    // Storage: Bounties BountyDescriptions (r:0 w:1)
    // Storage: Bounties Bounties (r:0 w:1)
    fn propose_bounty(d: u32) -> Weight {
        Weight::from_ref_time(22_070_891)
            // Standard Error: 7
            .saturating_add(Weight::from_ref_time(513).saturating_mul(d.into()))
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    // Storage: Bounties Bounties (r:1 w:1)
    // Storage: Bounties BountyApprovals (r:1 w:1)
    fn approve_bounty() -> Weight {
        Weight::from_ref_time(9_036_000)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    // Storage: Bounties Bounties (r:1 w:1)
    fn propose_curator() -> Weight {
        Weight::from_ref_time(8_789_000)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    // Storage: Bounties Bounties (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    fn unassign_curator() -> Weight {
        Weight::from_ref_time(25_810_000)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    // Storage: Bounties Bounties (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    fn accept_curator() -> Weight {
        Weight::from_ref_time(21_446_000)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    // Storage: Bounties Bounties (r:1 w:1)
    fn award_bounty() -> Weight {
        Weight::from_ref_time(16_247_000)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    // Storage: Bounties Bounties (r:1 w:1)
    // Storage: System Account (r:3 w:3)
    // Storage: Bounties BountyDescriptions (r:0 w:1)
    fn claim_bounty() -> Weight {
        Weight::from_ref_time(50_791_000)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(5))
    }
    // Storage: Bounties Bounties (r:1 w:1)
    // Storage: Bounties BountyDescriptions (r:0 w:1)
    fn close_bounty_proposed() -> Weight {
        Weight::from_ref_time(26_410_000)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    // Storage: Bounties Bounties (r:1 w:1)
    // Storage: System Account (r:3 w:3)
    // Storage: Bounties BountyDescriptions (r:0 w:1)
    fn close_bounty_active() -> Weight {
        Weight::from_ref_time(39_490_000)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(5))
    }
    // Storage: Bounties Bounties (r:1 w:1)
    fn extend_bounty_expiry() -> Weight {
        Weight::from_ref_time(16_379_000)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    // Storage: Bounties BountyApprovals (r:1 w:1)
    // Storage: Bounties Bounties (r:2 w:2)
    // Storage: System Account (r:3 w:3)
    fn spend_funds(b: u32) -> Weight {
        Weight::from_ref_time(19_140_664)
            // Standard Error: 26_913
            .saturating_add(Weight::from_ref_time(18_448_072).saturating_mul(b.into()))
            .saturating_add(T::DbWeight::get().reads((3_u64).saturating_mul(b.into())))
            .saturating_add(T::DbWeight::get().writes((3_u64).saturating_mul(b.into())))
    }
}
