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

//! Autogenerated weights for pallet_collective
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: `2023-09-19`, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
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
// --pallet=pallet_collective
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

/// Weight functions for pallet_collective (automatically generated)
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_collective::weights::WeightInfo for WeightInfo<T> {
    /// Storage: AdvisoryCommittee Members (r:1 w:1)
    /// Proof Skipped: AdvisoryCommittee Members (max_values: Some(1), max_size: None, mode: Measured)
    /// Storage: AdvisoryCommittee Proposals (r:1 w:0)
    /// Proof Skipped: AdvisoryCommittee Proposals (max_values: Some(1), max_size: None, mode: Measured)
    /// Storage: AdvisoryCommittee Voting (r:255 w:255)
    /// Proof Skipped: AdvisoryCommittee Voting (max_values: None, max_size: None, mode: Measured)
    /// Storage: AdvisoryCommittee Prime (r:0 w:1)
    /// Proof Skipped: AdvisoryCommittee Prime (max_values: Some(1), max_size: None, mode: Measured)
    fn set_members(m: u32, _n: u32, p: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0 + m * (8195 ±0) + p * (3227 ±0)`
        //  Estimated: `33167 + m * (19751 ±60) + p * (10255 ±23)`
        // Minimum execution time: 30_430 nanoseconds.
        Weight::from_parts(30_980_000, 33167)
            // Standard Error: 332_697
            .saturating_add(Weight::from_ref_time(25_449_256).saturating_mul(m.into()))
            // Standard Error: 130_624
            .saturating_add(Weight::from_ref_time(15_889_730).saturating_mul(p.into()))
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(p.into())))
            .saturating_add(T::DbWeight::get().writes(2_u64))
            .saturating_add(Weight::from_proof_size(19751).saturating_mul(m.into()))
            .saturating_add(Weight::from_proof_size(10255).saturating_mul(p.into()))
    }
    /// Storage: AdvisoryCommittee Members (r:1 w:0)
    /// Proof Skipped: AdvisoryCommittee Members (max_values: Some(1), max_size: None, mode: Measured)
    fn execute(b: u32, m: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `100 + m * (32 ±0)`
        //  Estimated: `596 + m * (32 ±0)`
        // Minimum execution time: 29_380 nanoseconds.
        Weight::from_parts(32_168_334, 596)
            // Standard Error: 176
            .saturating_add(Weight::from_ref_time(4_002).saturating_mul(b.into()))
            // Standard Error: 1_816
            .saturating_add(Weight::from_ref_time(36_737).saturating_mul(m.into()))
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(Weight::from_proof_size(32).saturating_mul(m.into()))
    }
    /// Storage: AdvisoryCommittee Members (r:1 w:0)
    /// Proof Skipped: AdvisoryCommittee Members (max_values: Some(1), max_size: None, mode: Measured)
    /// Storage: AdvisoryCommittee ProposalOf (r:1 w:0)
    /// Proof Skipped: AdvisoryCommittee ProposalOf (max_values: None, max_size: None, mode: Measured)
    fn propose_execute(b: u32, m: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `100 + m * (32 ±0)`
        //  Estimated: `3172 + m * (64 ±0)`
        // Minimum execution time: 32_490 nanoseconds.
        Weight::from_parts(32_891_978, 3172)
            // Standard Error: 606
            .saturating_add(Weight::from_ref_time(5_974).saturating_mul(b.into()))
            // Standard Error: 6_249
            .saturating_add(Weight::from_ref_time(100_564).saturating_mul(m.into()))
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(Weight::from_proof_size(64).saturating_mul(m.into()))
    }
    /// Storage: AdvisoryCommittee Members (r:1 w:0)
    /// Proof Skipped: AdvisoryCommittee Members (max_values: Some(1), max_size: None, mode: Measured)
    /// Storage: AdvisoryCommittee ProposalOf (r:1 w:1)
    /// Proof Skipped: AdvisoryCommittee ProposalOf (max_values: None, max_size: None, mode: Measured)
    /// Storage: AdvisoryCommittee Proposals (r:1 w:1)
    /// Proof Skipped: AdvisoryCommittee Proposals (max_values: Some(1), max_size: None, mode: Measured)
    /// Storage: AdvisoryCommittee ProposalCount (r:1 w:1)
    /// Proof Skipped: AdvisoryCommittee ProposalCount (max_values: Some(1), max_size: None, mode: Measured)
    /// Storage: AdvisoryCommittee Voting (r:0 w:1)
    /// Proof Skipped: AdvisoryCommittee Voting (max_values: None, max_size: None, mode: Measured)
    fn propose_proposed(b: u32, m: u32, p: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `565 + m * (32 ±0) + p * (33 ±0)`
        //  Estimated: `6570 + m * (160 ±0) + p * (170 ±0)`
        // Minimum execution time: 46_210 nanoseconds.
        Weight::from_parts(43_886_366, 6570)
            // Standard Error: 751
            .saturating_add(Weight::from_ref_time(11_946).saturating_mul(b.into()))
            // Standard Error: 7_842
            .saturating_add(Weight::from_ref_time(27_670).saturating_mul(m.into()))
            // Standard Error: 3_019
            .saturating_add(Weight::from_ref_time(220_177).saturating_mul(p.into()))
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().writes(4_u64))
            .saturating_add(Weight::from_proof_size(160).saturating_mul(m.into()))
            .saturating_add(Weight::from_proof_size(170).saturating_mul(p.into()))
    }
    /// Storage: AdvisoryCommittee Members (r:1 w:0)
    /// Proof Skipped: AdvisoryCommittee Members (max_values: Some(1), max_size: None, mode: Measured)
    /// Storage: AdvisoryCommittee Voting (r:1 w:1)
    /// Proof Skipped: AdvisoryCommittee Voting (max_values: None, max_size: None, mode: Measured)
    fn vote(m: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1240 + m * (64 ±0)`
        //  Estimated: `5448 + m * (128 ±0)`
        // Minimum execution time: 42_820 nanoseconds.
        Weight::from_parts(57_247_570, 5448)
            // Standard Error: 5_586
            .saturating_add(Weight::from_ref_time(90_933).saturating_mul(m.into()))
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
            .saturating_add(Weight::from_proof_size(128).saturating_mul(m.into()))
    }
    /// Storage: AdvisoryCommittee Voting (r:1 w:1)
    /// Proof Skipped: AdvisoryCommittee Voting (max_values: None, max_size: None, mode: Measured)
    /// Storage: AdvisoryCommittee Members (r:1 w:0)
    /// Proof Skipped: AdvisoryCommittee Members (max_values: Some(1), max_size: None, mode: Measured)
    /// Storage: AdvisoryCommittee Proposals (r:1 w:1)
    /// Proof Skipped: AdvisoryCommittee Proposals (max_values: Some(1), max_size: None, mode: Measured)
    /// Storage: AdvisoryCommittee ProposalOf (r:0 w:1)
    /// Proof Skipped: AdvisoryCommittee ProposalOf (max_values: None, max_size: None, mode: Measured)
    fn close_early_disapproved(m: u32, p: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `683 + m * (64 ±0) + p * (33 ±0)`
        //  Estimated: `6017 + m * (260 ±0) + p * (136 ±0)`
        // Minimum execution time: 42_010 nanoseconds.
        Weight::from_parts(41_739_149, 6017)
            // Standard Error: 8_825
            .saturating_add(Weight::from_ref_time(133_832).saturating_mul(m.into()))
            // Standard Error: 3_354
            .saturating_add(Weight::from_ref_time(213_771).saturating_mul(p.into()))
            .saturating_add(T::DbWeight::get().reads(3_u64))
            .saturating_add(T::DbWeight::get().writes(3_u64))
            .saturating_add(Weight::from_proof_size(260).saturating_mul(m.into()))
            .saturating_add(Weight::from_proof_size(136).saturating_mul(p.into()))
    }
    /// Storage: AdvisoryCommittee Voting (r:1 w:1)
    /// Proof Skipped: AdvisoryCommittee Voting (max_values: None, max_size: None, mode: Measured)
    /// Storage: AdvisoryCommittee Members (r:1 w:0)
    /// Proof Skipped: AdvisoryCommittee Members (max_values: Some(1), max_size: None, mode: Measured)
    /// Storage: AdvisoryCommittee ProposalOf (r:1 w:1)
    /// Proof Skipped: AdvisoryCommittee ProposalOf (max_values: None, max_size: None, mode: Measured)
    /// Storage: AdvisoryCommittee Proposals (r:1 w:1)
    /// Proof Skipped: AdvisoryCommittee Proposals (max_values: Some(1), max_size: None, mode: Measured)
    fn close_early_approved(b: u32, m: u32, p: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `926 + b * (1 ±0) + m * (64 ±0) + p * (36 ±0)`
        //  Estimated: `9916 + b * (4 ±0) + m * (248 ±0) + p * (144 ±0)`
        // Minimum execution time: 57_630 nanoseconds.
        Weight::from_parts(71_246_911, 9916)
            // Standard Error: 914
            .saturating_add(Weight::from_ref_time(10_364).saturating_mul(b.into()))
            // Standard Error: 9_666
            .saturating_add(Weight::from_ref_time(28_754).saturating_mul(m.into()))
            // Standard Error: 3_674
            .saturating_add(Weight::from_ref_time(240_907).saturating_mul(p.into()))
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().writes(3_u64))
            .saturating_add(Weight::from_proof_size(4).saturating_mul(b.into()))
            .saturating_add(Weight::from_proof_size(248).saturating_mul(m.into()))
            .saturating_add(Weight::from_proof_size(144).saturating_mul(p.into()))
    }
    /// Storage: AdvisoryCommittee Voting (r:1 w:1)
    /// Proof Skipped: AdvisoryCommittee Voting (max_values: None, max_size: None, mode: Measured)
    /// Storage: AdvisoryCommittee Members (r:1 w:0)
    /// Proof Skipped: AdvisoryCommittee Members (max_values: Some(1), max_size: None, mode: Measured)
    /// Storage: AdvisoryCommittee Prime (r:1 w:0)
    /// Proof Skipped: AdvisoryCommittee Prime (max_values: Some(1), max_size: None, mode: Measured)
    /// Storage: AdvisoryCommittee Proposals (r:1 w:1)
    /// Proof Skipped: AdvisoryCommittee Proposals (max_values: Some(1), max_size: None, mode: Measured)
    /// Storage: AdvisoryCommittee ProposalOf (r:0 w:1)
    /// Proof Skipped: AdvisoryCommittee ProposalOf (max_values: None, max_size: None, mode: Measured)
    fn close_disapproved(m: u32, p: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `703 + m * (64 ±0) + p * (33 ±0)`
        //  Estimated: `7250 + m * (325 ±0) + p * (170 ±0)`
        // Minimum execution time: 47_540 nanoseconds.
        Weight::from_parts(58_061_738, 7250)
            // Standard Error: 9_360
            .saturating_add(Weight::from_ref_time(30_766).saturating_mul(m.into()))
            // Standard Error: 3_557
            .saturating_add(Weight::from_ref_time(204_081).saturating_mul(p.into()))
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().writes(3_u64))
            .saturating_add(Weight::from_proof_size(325).saturating_mul(m.into()))
            .saturating_add(Weight::from_proof_size(170).saturating_mul(p.into()))
    }
    /// Storage: AdvisoryCommittee Voting (r:1 w:1)
    /// Proof Skipped: AdvisoryCommittee Voting (max_values: None, max_size: None, mode: Measured)
    /// Storage: AdvisoryCommittee Members (r:1 w:0)
    /// Proof Skipped: AdvisoryCommittee Members (max_values: Some(1), max_size: None, mode: Measured)
    /// Storage: AdvisoryCommittee Prime (r:1 w:0)
    /// Proof Skipped: AdvisoryCommittee Prime (max_values: Some(1), max_size: None, mode: Measured)
    /// Storage: AdvisoryCommittee ProposalOf (r:1 w:1)
    /// Proof Skipped: AdvisoryCommittee ProposalOf (max_values: None, max_size: None, mode: Measured)
    /// Storage: AdvisoryCommittee Proposals (r:1 w:1)
    /// Proof Skipped: AdvisoryCommittee Proposals (max_values: Some(1), max_size: None, mode: Measured)
    fn close_approved(b: u32, m: u32, p: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `946 + b * (1 ±0) + m * (64 ±0) + p * (36 ±0)`
        //  Estimated: `11505 + b * (5 ±0) + m * (310 ±0) + p * (180 ±0)`
        // Minimum execution time: 69_630 nanoseconds.
        Weight::from_parts(75_042_860, 11505)
            // Standard Error: 818
            .saturating_add(Weight::from_ref_time(9_063).saturating_mul(b.into()))
            // Standard Error: 8_654
            .saturating_add(Weight::from_ref_time(37_972).saturating_mul(m.into()))
            // Standard Error: 3_289
            .saturating_add(Weight::from_ref_time(253_280).saturating_mul(p.into()))
            .saturating_add(T::DbWeight::get().reads(5_u64))
            .saturating_add(T::DbWeight::get().writes(3_u64))
            .saturating_add(Weight::from_proof_size(5).saturating_mul(b.into()))
            .saturating_add(Weight::from_proof_size(310).saturating_mul(m.into()))
            .saturating_add(Weight::from_proof_size(180).saturating_mul(p.into()))
    }
    /// Storage: AdvisoryCommittee Proposals (r:1 w:1)
    /// Proof Skipped: AdvisoryCommittee Proposals (max_values: Some(1), max_size: None, mode: Measured)
    /// Storage: AdvisoryCommittee Voting (r:0 w:1)
    /// Proof Skipped: AdvisoryCommittee Voting (max_values: None, max_size: None, mode: Measured)
    /// Storage: AdvisoryCommittee ProposalOf (r:0 w:1)
    /// Proof Skipped: AdvisoryCommittee ProposalOf (max_values: None, max_size: None, mode: Measured)
    fn disapprove_proposal(p: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `258 + p * (32 ±0)`
        //  Estimated: `1266 + p * (96 ±0)`
        // Minimum execution time: 27_850 nanoseconds.
        Weight::from_parts(34_667_615, 1266)
            // Standard Error: 3_964
            .saturating_add(Weight::from_ref_time(173_407).saturating_mul(p.into()))
            .saturating_add(T::DbWeight::get().reads(1_u64))
            .saturating_add(T::DbWeight::get().writes(3_u64))
            .saturating_add(Weight::from_proof_size(96).saturating_mul(p.into()))
    }
}
