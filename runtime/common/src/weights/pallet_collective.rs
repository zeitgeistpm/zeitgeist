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

//! Autogenerated weights for pallet_collective
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: `2024-12-06`, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
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
    /// Storage: `AdvisoryCommittee::Members` (r:1 w:1)
    /// Proof: `AdvisoryCommittee::Members` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `AdvisoryCommittee::Proposals` (r:1 w:0)
    /// Proof: `AdvisoryCommittee::Proposals` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `AdvisoryCommittee::Voting` (r:255 w:255)
    /// Proof: `AdvisoryCommittee::Voting` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `AdvisoryCommittee::Prime` (r:0 w:1)
    /// Proof: `AdvisoryCommittee::Prime` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// The range of component `m` is `[0, 100]`.
    /// The range of component `n` is `[0, 100]`.
    /// The range of component `p` is `[0, 255]`.
    fn set_members(m: u32, _n: u32, p: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0 + m * (8192 ±0) + p * (3194 ±0)`
        //  Estimated: `32708 + m * (4970 ±60) + p * (4343 ±23)`
        // Minimum execution time: 20_230 nanoseconds.
        Weight::from_parts(20_660_000, 32708)
            // Standard Error: 175_177
            .saturating_add(Weight::from_parts(12_549_827, 0).saturating_mul(m.into()))
            // Standard Error: 68_778
            .saturating_add(Weight::from_parts(9_576_190, 0).saturating_mul(p.into()))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(p.into())))
            .saturating_add(T::DbWeight::get().writes(2))
            .saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(p.into())))
            .saturating_add(Weight::from_parts(0, 4970).saturating_mul(m.into()))
            .saturating_add(Weight::from_parts(0, 4343).saturating_mul(p.into()))
    }
    /// Storage: `AdvisoryCommittee::Members` (r:1 w:0)
    /// Proof: `AdvisoryCommittee::Members` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// The range of component `b` is `[2, 1024]`.
    /// The range of component `m` is `[1, 100]`.
    fn execute(b: u32, m: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `30 + m * (32 ±0)`
        //  Estimated: `1516 + m * (32 ±0)`
        // Minimum execution time: 19_090 nanoseconds.
        Weight::from_parts(19_282_201, 1516)
            // Standard Error: 85
            .saturating_add(Weight::from_parts(1_712, 0).saturating_mul(b.into()))
            // Standard Error: 876
            .saturating_add(Weight::from_parts(14_421, 0).saturating_mul(m.into()))
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(Weight::from_parts(0, 32).saturating_mul(m.into()))
    }
    /// Storage: `AdvisoryCommittee::Members` (r:1 w:0)
    /// Proof: `AdvisoryCommittee::Members` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `AdvisoryCommittee::ProposalOf` (r:1 w:0)
    /// Proof: `AdvisoryCommittee::ProposalOf` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// The range of component `b` is `[2, 1024]`.
    /// The range of component `m` is `[1, 100]`.
    fn propose_execute(b: u32, m: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `30 + m * (32 ±0)`
        //  Estimated: `3496 + m * (32 ±0)`
        // Minimum execution time: 22_160 nanoseconds.
        Weight::from_parts(21_652_824, 3496)
            // Standard Error: 388
            .saturating_add(Weight::from_parts(2_994, 0).saturating_mul(b.into()))
            // Standard Error: 4_003
            .saturating_add(Weight::from_parts(28_532, 0).saturating_mul(m.into()))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(Weight::from_parts(0, 32).saturating_mul(m.into()))
    }
    /// Storage: `AdvisoryCommittee::Members` (r:1 w:0)
    /// Proof: `AdvisoryCommittee::Members` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `AdvisoryCommittee::ProposalOf` (r:1 w:1)
    /// Proof: `AdvisoryCommittee::ProposalOf` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `AdvisoryCommittee::Proposals` (r:1 w:1)
    /// Proof: `AdvisoryCommittee::Proposals` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `AdvisoryCommittee::ProposalCount` (r:1 w:1)
    /// Proof: `AdvisoryCommittee::ProposalCount` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `AdvisoryCommittee::Voting` (r:0 w:1)
    /// Proof: `AdvisoryCommittee::Voting` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// The range of component `b` is `[2, 1024]`.
    /// The range of component `m` is `[2, 100]`.
    /// The range of component `p` is `[1, 255]`.
    fn propose_proposed(b: u32, m: u32, p: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `463 + m * (32 ±0) + p * (33 ±0)`
        //  Estimated: `3888 + m * (32 ±0) + p * (34 ±0)`
        // Minimum execution time: 30_561 nanoseconds.
        Weight::from_parts(37_247_607, 3888)
            // Standard Error: 300
            .saturating_add(Weight::from_parts(1_974, 0).saturating_mul(b.into()))
            // Standard Error: 3_132
            .saturating_add(Weight::from_parts(12_092, 0).saturating_mul(m.into()))
            // Standard Error: 1_205
            .saturating_add(Weight::from_parts(181_573, 0).saturating_mul(p.into()))
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(4))
            .saturating_add(Weight::from_parts(0, 32).saturating_mul(m.into()))
            .saturating_add(Weight::from_parts(0, 34).saturating_mul(p.into()))
    }
    /// Storage: `AdvisoryCommittee::Members` (r:1 w:0)
    /// Proof: `AdvisoryCommittee::Members` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `AdvisoryCommittee::Voting` (r:1 w:1)
    /// Proof: `AdvisoryCommittee::Voting` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// The range of component `m` is `[5, 100]`.
    fn vote(m: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1137 + m * (64 ±0)`
        //  Estimated: `4601 + m * (64 ±0)`
        // Minimum execution time: 30_150 nanoseconds.
        Weight::from_parts(34_509_322, 4601)
            // Standard Error: 1_030
            .saturating_add(Weight::from_parts(43_510, 0).saturating_mul(m.into()))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
            .saturating_add(Weight::from_parts(0, 64).saturating_mul(m.into()))
    }
    /// Storage: `AdvisoryCommittee::Voting` (r:1 w:1)
    /// Proof: `AdvisoryCommittee::Voting` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `AdvisoryCommittee::Members` (r:1 w:0)
    /// Proof: `AdvisoryCommittee::Members` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `AdvisoryCommittee::Proposals` (r:1 w:1)
    /// Proof: `AdvisoryCommittee::Proposals` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `AdvisoryCommittee::ProposalOf` (r:0 w:1)
    /// Proof: `AdvisoryCommittee::ProposalOf` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// The range of component `m` is `[4, 100]`.
    /// The range of component `p` is `[1, 255]`.
    fn close_early_disapproved(m: u32, p: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `548 + m * (64 ±0) + p * (33 ±0)`
        //  Estimated: `3968 + m * (65 ±0) + p * (34 ±0)`
        // Minimum execution time: 35_251 nanoseconds.
        Weight::from_parts(38_650_159, 3968)
            // Standard Error: 3_045
            .saturating_add(Weight::from_parts(17_702, 0).saturating_mul(m.into()))
            // Standard Error: 1_157
            .saturating_add(Weight::from_parts(177_006, 0).saturating_mul(p.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
            .saturating_add(Weight::from_parts(0, 65).saturating_mul(m.into()))
            .saturating_add(Weight::from_parts(0, 34).saturating_mul(p.into()))
    }
    /// Storage: `AdvisoryCommittee::Voting` (r:1 w:1)
    /// Proof: `AdvisoryCommittee::Voting` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `AdvisoryCommittee::Members` (r:1 w:0)
    /// Proof: `AdvisoryCommittee::Members` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `AdvisoryCommittee::ProposalOf` (r:1 w:1)
    /// Proof: `AdvisoryCommittee::ProposalOf` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `AdvisoryCommittee::Proposals` (r:1 w:1)
    /// Proof: `AdvisoryCommittee::Proposals` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// The range of component `b` is `[2, 1024]`.
    /// The range of component `m` is `[4, 100]`.
    /// The range of component `p` is `[1, 255]`.
    fn close_early_approved(b: u32, m: u32, p: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `760 + b * (1 ±0) + m * (64 ±0) + p * (36 ±0)`
        //  Estimated: `4299 + b * (1 ±0) + m * (62 ±0) + p * (36 ±0)`
        // Minimum execution time: 48_761 nanoseconds.
        Weight::from_parts(52_434_394, 4299)
            // Standard Error: 413
            .saturating_add(Weight::from_parts(2_904, 0).saturating_mul(b.into()))
            // Standard Error: 4_365
            .saturating_add(Weight::from_parts(25_525, 0).saturating_mul(m.into()))
            // Standard Error: 1_659
            .saturating_add(Weight::from_parts(190_105, 0).saturating_mul(p.into()))
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(3))
            .saturating_add(Weight::from_parts(0, 1).saturating_mul(b.into()))
            .saturating_add(Weight::from_parts(0, 62).saturating_mul(m.into()))
            .saturating_add(Weight::from_parts(0, 36).saturating_mul(p.into()))
    }
    /// Storage: `AdvisoryCommittee::Voting` (r:1 w:1)
    /// Proof: `AdvisoryCommittee::Voting` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `AdvisoryCommittee::Members` (r:1 w:0)
    /// Proof: `AdvisoryCommittee::Members` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `AdvisoryCommittee::Prime` (r:1 w:0)
    /// Proof: `AdvisoryCommittee::Prime` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `AdvisoryCommittee::Proposals` (r:1 w:1)
    /// Proof: `AdvisoryCommittee::Proposals` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `AdvisoryCommittee::ProposalOf` (r:0 w:1)
    /// Proof: `AdvisoryCommittee::ProposalOf` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// The range of component `m` is `[4, 100]`.
    /// The range of component `p` is `[1, 255]`.
    fn close_disapproved(m: u32, p: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `568 + m * (64 ±0) + p * (33 ±0)`
        //  Estimated: `3988 + m * (65 ±0) + p * (34 ±0)`
        // Minimum execution time: 38_400 nanoseconds.
        Weight::from_parts(39_328_977, 3988)
            // Standard Error: 3_370
            .saturating_add(Weight::from_parts(45_313, 0).saturating_mul(m.into()))
            // Standard Error: 1_280
            .saturating_add(Weight::from_parts(180_092, 0).saturating_mul(p.into()))
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(3))
            .saturating_add(Weight::from_parts(0, 65).saturating_mul(m.into()))
            .saturating_add(Weight::from_parts(0, 34).saturating_mul(p.into()))
    }
    /// Storage: `AdvisoryCommittee::Voting` (r:1 w:1)
    /// Proof: `AdvisoryCommittee::Voting` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `AdvisoryCommittee::Members` (r:1 w:0)
    /// Proof: `AdvisoryCommittee::Members` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `AdvisoryCommittee::Prime` (r:1 w:0)
    /// Proof: `AdvisoryCommittee::Prime` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `AdvisoryCommittee::ProposalOf` (r:1 w:1)
    /// Proof: `AdvisoryCommittee::ProposalOf` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `AdvisoryCommittee::Proposals` (r:1 w:1)
    /// Proof: `AdvisoryCommittee::Proposals` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// The range of component `b` is `[2, 1024]`.
    /// The range of component `m` is `[4, 100]`.
    /// The range of component `p` is `[1, 255]`.
    fn close_approved(b: u32, m: u32, p: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `780 + b * (1 ±0) + m * (64 ±0) + p * (36 ±0)`
        //  Estimated: `4319 + b * (1 ±0) + m * (62 ±0) + p * (36 ±0)`
        // Minimum execution time: 51_891 nanoseconds.
        Weight::from_parts(69_242_224, 4319)
            // Standard Error: 3_089
            .saturating_add(Weight::from_parts(195_731, 0).saturating_mul(p.into()))
            .saturating_add(T::DbWeight::get().reads(5))
            .saturating_add(T::DbWeight::get().writes(3))
            .saturating_add(Weight::from_parts(0, 1).saturating_mul(b.into()))
            .saturating_add(Weight::from_parts(0, 62).saturating_mul(m.into()))
            .saturating_add(Weight::from_parts(0, 36).saturating_mul(p.into()))
    }
    /// Storage: `AdvisoryCommittee::Proposals` (r:1 w:1)
    /// Proof: `AdvisoryCommittee::Proposals` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
    /// Storage: `AdvisoryCommittee::Voting` (r:0 w:1)
    /// Proof: `AdvisoryCommittee::Voting` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// Storage: `AdvisoryCommittee::ProposalOf` (r:0 w:1)
    /// Proof: `AdvisoryCommittee::ProposalOf` (`max_values`: None, `max_size`: None, mode: `Measured`)
    /// The range of component `p` is `[1, 255]`.
    fn disapprove_proposal(p: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `188 + p * (32 ±0)`
        //  Estimated: `1672 + p * (32 ±0)`
        // Minimum execution time: 19_031 nanoseconds.
        Weight::from_parts(22_701_163, 1672)
            // Standard Error: 1_123
            .saturating_add(Weight::from_parts(194_782, 0).saturating_mul(p.into()))
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(3))
            .saturating_add(Weight::from_parts(0, 32).saturating_mul(p.into()))
    }
}
