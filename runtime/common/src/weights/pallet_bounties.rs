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

//! Autogenerated weights for pallet_bounties
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: `2024-04-02`, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `zeitgeist-benchmark`, CPU: `AMD EPYC 7601 32-Core Processor`
//! EXECUTION: `Some(Wasm)`, WASM-EXECUTION: `Compiled`, CHAIN: `Some("dev")`, DB CACHE: `1024`

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
// --header=./HEADER_GPL3
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
    /// Storage: Bounties BountyCount (r:1 w:1)
    /// Proof: Bounties BountyCount (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
    /// Storage: System Account (r:1 w:1)
    /// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
    /// Storage: Bounties BountyDescriptions (r:0 w:1)
    /// Proof: Bounties BountyDescriptions (max_values: None, max_size: Some(8206), added: 10681, mode: MaxEncodedLen)
    /// Storage: Bounties Bounties (r:0 w:1)
    /// Proof: Bounties Bounties (max_values: None, max_size: Some(181), added: 2656, mode: MaxEncodedLen)
    /// The range of component `d` is `[0, 8192]`.
    fn propose_bounty(d: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `141`
        //  Estimated: `3106`
        // Minimum execution time: 37_041 nanoseconds.
        Weight::from_parts(41_907_852, 3106)
            // Standard Error: 77
            .saturating_add(Weight::from_parts(1_483, 0).saturating_mul(d.into()))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(4))
    }
    /// Storage: Bounties Bounties (r:1 w:1)
    /// Proof: Bounties Bounties (max_values: None, max_size: Some(181), added: 2656, mode: MaxEncodedLen)
    /// Storage: Bounties BountyApprovals (r:1 w:1)
    /// Proof: Bounties BountyApprovals (max_values: Some(1), max_size: Some(402), added: 897, mode: MaxEncodedLen)
    fn approve_bounty() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `229`
        //  Estimated: `3553`
        // Minimum execution time: 17_760 nanoseconds.
        Weight::from_parts(22_020_000, 3553)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    /// Storage: Bounties Bounties (r:1 w:1)
    /// Proof: Bounties Bounties (max_values: None, max_size: Some(181), added: 2656, mode: MaxEncodedLen)
    fn propose_curator() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `249`
        //  Estimated: `2656`
        // Minimum execution time: 15_430 nanoseconds.
        Weight::from_parts(16_320_000, 2656)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    /// Storage: Bounties Bounties (r:1 w:1)
    /// Proof: Bounties Bounties (max_values: None, max_size: Some(181), added: 2656, mode: MaxEncodedLen)
    /// Storage: System Account (r:2 w:2)
    /// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
    fn unassign_curator() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `641`
        //  Estimated: `7870`
        // Minimum execution time: 55_790 nanoseconds.
        Weight::from_parts(68_600_000, 7870)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    /// Storage: Bounties Bounties (r:1 w:1)
    /// Proof: Bounties Bounties (max_values: None, max_size: Some(181), added: 2656, mode: MaxEncodedLen)
    /// Storage: System Account (r:1 w:1)
    /// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
    fn accept_curator() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `457`
        //  Estimated: `5263`
        // Minimum execution time: 35_110 nanoseconds.
        Weight::from_parts(41_720_000, 5263)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    /// Storage: Bounties Bounties (r:1 w:1)
    /// Proof: Bounties Bounties (max_values: None, max_size: Some(181), added: 2656, mode: MaxEncodedLen)
    fn award_bounty() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `289`
        //  Estimated: `2656`
        // Minimum execution time: 25_530 nanoseconds.
        Weight::from_parts(30_380_000, 2656)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    /// Storage: Bounties Bounties (r:1 w:1)
    /// Proof: Bounties Bounties (max_values: None, max_size: Some(181), added: 2656, mode: MaxEncodedLen)
    /// Storage: System Account (r:3 w:3)
    /// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
    /// Storage: Bounties BountyDescriptions (r:0 w:1)
    /// Proof: Bounties BountyDescriptions (max_values: None, max_size: Some(8206), added: 10681, mode: MaxEncodedLen)
    fn claim_bounty() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `674`
        //  Estimated: `10477`
        // Minimum execution time: 99_900 nanoseconds.
        Weight::from_parts(110_310_000, 10477)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(5))
    }
    /// Storage: Bounties Bounties (r:1 w:1)
    /// Proof: Bounties Bounties (max_values: None, max_size: Some(181), added: 2656, mode: MaxEncodedLen)
    /// Storage: System Account (r:2 w:2)
    /// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
    /// Storage: Bounties BountyDescriptions (r:0 w:1)
    /// Proof: Bounties BountyDescriptions (max_values: None, max_size: Some(8206), added: 10681, mode: MaxEncodedLen)
    fn close_bounty_proposed() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `541`
        //  Estimated: `7870`
        // Minimum execution time: 70_890 nanoseconds.
        Weight::from_parts(72_370_000, 7870)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(4))
    }
    /// Storage: Bounties Bounties (r:1 w:1)
    /// Proof: Bounties Bounties (max_values: None, max_size: Some(181), added: 2656, mode: MaxEncodedLen)
    /// Storage: System Account (r:3 w:3)
    /// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
    /// Storage: Bounties BountyDescriptions (r:0 w:1)
    /// Proof: Bounties BountyDescriptions (max_values: None, max_size: Some(8206), added: 10681, mode: MaxEncodedLen)
    fn close_bounty_active() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `818`
        //  Estimated: `10477`
        // Minimum execution time: 75_191 nanoseconds.
        Weight::from_parts(92_190_000, 10477)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(5))
    }
    /// Storage: Bounties Bounties (r:1 w:1)
    /// Proof: Bounties Bounties (max_values: None, max_size: Some(181), added: 2656, mode: MaxEncodedLen)
    fn extend_bounty_expiry() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `289`
        //  Estimated: `2656`
        // Minimum execution time: 23_930 nanoseconds.
        Weight::from_parts(24_920_000, 2656)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    /// Storage: Bounties BountyApprovals (r:1 w:1)
    /// Proof: Bounties BountyApprovals (max_values: Some(1), max_size: Some(402), added: 897, mode: MaxEncodedLen)
    /// Storage: Bounties Bounties (r:100 w:100)
    /// Proof: Bounties Bounties (max_values: None, max_size: Some(181), added: 2656, mode: MaxEncodedLen)
    /// Storage: System Account (r:200 w:200)
    /// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
    /// The range of component `b` is `[0, 100]`.
    fn spend_funds(b: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `98 + b * (357 ±0)`
        //  Estimated: `897 + b * (7870 ±0)`
        // Minimum execution time: 6_910 nanoseconds.
        Weight::from_parts(18_947_391, 897)
            // Standard Error: 190_619
            .saturating_add(Weight::from_parts(46_824_225, 0).saturating_mul(b.into()))
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().reads((3_u64).saturating_mul(b.into())))
            .saturating_add(T::DbWeight::get().writes(1))
            .saturating_add(T::DbWeight::get().writes((3_u64).saturating_mul(b.into())))
            .saturating_add(Weight::from_parts(0, 7870).saturating_mul(b.into()))
    }
}
