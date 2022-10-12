//! Autogenerated weights for pallet_bounties
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-08-31, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/zeitgeist
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
        (20_068_000 as Weight)
            // Standard Error: 0
            .saturating_add((1_000 as Weight).saturating_mul(d as Weight))
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    // Storage: Bounties Bounties (r:1 w:1)
    // Storage: Bounties BountyApprovals (r:1 w:1)
    fn approve_bounty() -> Weight {
        (5_712_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: Bounties Bounties (r:1 w:1)
    fn propose_curator() -> Weight {
        (4_147_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: Bounties Bounties (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    fn unassign_curator() -> Weight {
        (23_050_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: Bounties Bounties (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    fn accept_curator() -> Weight {
        (17_347_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: Bounties Bounties (r:1 w:1)
    fn award_bounty() -> Weight {
        (12_240_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: Bounties Bounties (r:1 w:1)
    // Storage: System Account (r:3 w:3)
    // Storage: Bounties BountyDescriptions (r:0 w:1)
    fn claim_bounty() -> Weight {
        (54_625_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(4 as Weight))
            .saturating_add(T::DbWeight::get().writes(5 as Weight))
    }
    // Storage: Bounties Bounties (r:1 w:1)
    // Storage: Bounties BountyDescriptions (r:0 w:1)
    fn close_bounty_proposed() -> Weight {
        (22_075_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: Bounties Bounties (r:1 w:1)
    // Storage: System Account (r:3 w:3)
    // Storage: Bounties BountyDescriptions (r:0 w:1)
    fn close_bounty_active() -> Weight {
        (39_006_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(4 as Weight))
            .saturating_add(T::DbWeight::get().writes(5 as Weight))
    }
    // Storage: Bounties Bounties (r:1 w:1)
    fn extend_bounty_expiry() -> Weight {
        (11_908_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: Bounties BountyApprovals (r:1 w:1)
    // Storage: Bounties Bounties (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    fn spend_funds(b: u32) -> Weight {
        (6_737_000 as Weight)
            // Standard Error: 12_000
            .saturating_add((26_347_000 as Weight).saturating_mul(b as Weight))
            .saturating_add(T::DbWeight::get().reads((3 as Weight).saturating_mul(b as Weight)))
            .saturating_add(T::DbWeight::get().writes((3 as Weight).saturating_mul(b as Weight)))
    }
}
