// Copyright 2022-2025 Forecasting Technologies LTD.
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

//! Autogenerated weights for pallet_treasury
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: `2025-02-26`, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `ztg-benchmark`, CPU: `AMD EPYC 7601 32-Core Processor`
//! EXECUTION: ``, WASM-EXECUTION: `Compiled`, CHAIN: `Some("dev")`, DB CACHE: `1024`

// Executed Command:
// ./target/production/zeitgeist
// benchmark
// pallet
// --chain=dev
// --steps=50
// --repeat=20
// --pallet=pallet_treasury
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

/// Weight functions for pallet_treasury (automatically generated)
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_treasury::weights::WeightInfo for WeightInfo<T> {
    /// Storage: `Treasury::ProposalCount` (r:1 w:1)
    /// Proof: `Treasury::ProposalCount` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
    /// Storage: `Treasury::Approvals` (r:1 w:1)
    /// Proof: `Treasury::Approvals` (`max_values`: Some(1), `max_size`: Some(402), added: 897, mode: `MaxEncodedLen`)
    /// Storage: `Treasury::Proposals` (r:0 w:1)
    /// Proof: `Treasury::Proposals` (`max_values`: None, `max_size`: Some(108), added: 2583, mode: `MaxEncodedLen`)
    fn spend() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `42`
        //  Estimated: `1887`
        // Minimum execution time: 16_790 nanoseconds.
        Weight::from_parts(17_430_000, 1887)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    /// Storage: `Treasury::ProposalCount` (r:1 w:1)
    /// Proof: `Treasury::ProposalCount` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
    /// Storage: `Treasury::Proposals` (r:0 w:1)
    /// Proof: `Treasury::Proposals` (`max_values`: None, `max_size`: Some(108), added: 2583, mode: `MaxEncodedLen`)
    fn propose_spend() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `147`
        //  Estimated: `1489`
        // Minimum execution time: 31_711 nanoseconds.
        Weight::from_parts(33_221_000, 1489)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    /// Storage: `Treasury::Proposals` (r:1 w:1)
    /// Proof: `Treasury::Proposals` (`max_values`: None, `max_size`: Some(108), added: 2583, mode: `MaxEncodedLen`)
    /// Storage: `System::Account` (r:2 w:2)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(132), added: 2607, mode: `MaxEncodedLen`)
    fn reject_proposal() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `449`
        //  Estimated: `6204`
        // Minimum execution time: 50_921 nanoseconds.
        Weight::from_parts(54_261_000, 6204)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    /// Storage: `Treasury::Proposals` (r:1 w:0)
    /// Proof: `Treasury::Proposals` (`max_values`: None, `max_size`: Some(108), added: 2583, mode: `MaxEncodedLen`)
    /// Storage: `Treasury::Approvals` (r:1 w:1)
    /// Proof: `Treasury::Approvals` (`max_values`: Some(1), `max_size`: Some(402), added: 897, mode: `MaxEncodedLen`)
    /// The range of component `p` is `[0, 99]`.
    fn approve_proposal(p: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `469 + p * (8 ±0)`
        //  Estimated: `3573`
        // Minimum execution time: 12_590 nanoseconds.
        Weight::from_parts(16_828_041, 3573)
            // Standard Error: 2_894
            .saturating_add(Weight::from_parts(63_291, 0).saturating_mul(p.into()))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    /// Storage: `Treasury::Approvals` (r:1 w:1)
    /// Proof: `Treasury::Approvals` (`max_values`: Some(1), `max_size`: Some(402), added: 897, mode: `MaxEncodedLen`)
    fn remove_approval() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `127`
        //  Estimated: `1887`
        // Minimum execution time: 9_670 nanoseconds.
        Weight::from_parts(10_330_000, 1887)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    /// Storage: `System::Account` (r:201 w:201)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(132), added: 2607, mode: `MaxEncodedLen`)
    /// Storage: `Treasury::Deactivated` (r:1 w:1)
    /// Proof: `Treasury::Deactivated` (`max_values`: Some(1), `max_size`: Some(16), added: 511, mode: `MaxEncodedLen`)
    /// Storage: `Treasury::Approvals` (r:1 w:1)
    /// Proof: `Treasury::Approvals` (`max_values`: Some(1), `max_size`: Some(402), added: 897, mode: `MaxEncodedLen`)
    /// Storage: `Treasury::Proposals` (r:100 w:100)
    /// Proof: `Treasury::Proposals` (`max_values`: None, `max_size`: Some(108), added: 2583, mode: `MaxEncodedLen`)
    /// Storage: `Bounties::BountyApprovals` (r:1 w:1)
    /// Proof: `Bounties::BountyApprovals` (`max_values`: Some(1), `max_size`: Some(402), added: 897, mode: `MaxEncodedLen`)
    /// The range of component `p` is `[0, 100]`.
    fn on_initialize_proposals(p: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `320 + p * (255 ±0)`
        //  Estimated: `3597 + p * (5214 ±0)`
        // Minimum execution time: 51_051 nanoseconds.
        Weight::from_parts(47_735_754, 3597)
            // Standard Error: 72_264
            .saturating_add(Weight::from_parts(47_484_503, 0).saturating_mul(p.into()))
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().reads((3_u64).saturating_mul(p.into())))
            .saturating_add(T::DbWeight::get().writes(4))
            .saturating_add(T::DbWeight::get().writes((3_u64).saturating_mul(p.into())))
            .saturating_add(Weight::from_parts(0, 5214).saturating_mul(p.into()))
    }
}
