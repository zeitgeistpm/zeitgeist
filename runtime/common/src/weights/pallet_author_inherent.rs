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

//! Autogenerated weights for pallet_author_inherent
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: `2023-09-12`, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
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
// --pallet=pallet_author_inherent
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

/// Weight functions for pallet_author_inherent (automatically generated)
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_author_inherent::weights::WeightInfo for WeightInfo<T> {
    /// Storage: ParachainSystem ValidationData (r:1 w:0)
    /// Proof Skipped: ParachainSystem ValidationData (max_values: Some(1), max_size: None, mode: Measured)
    /// Storage: AuthorInherent HighestSlotSeen (r:1 w:1)
    /// Proof: AuthorInherent HighestSlotSeen (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
    /// Storage: AuthorInherent Author (r:1 w:0)
    /// Proof: AuthorInherent Author (max_values: Some(1), max_size: Some(32), added: 527, mode: MaxEncodedLen)
    /// Storage: ParachainStaking SelectedCandidates (r:1 w:0)
    /// Proof Skipped: ParachainStaking SelectedCandidates (max_values: Some(1), max_size: None, mode: Measured)
    /// Storage: AuthorFilter EligibleCount (r:1 w:0)
    /// Proof Skipped: AuthorFilter EligibleCount (max_values: Some(1), max_size: None, mode: Measured)
    /// Storage: RandomnessCollectiveFlip RandomMaterial (r:1 w:0)
    /// Proof: RandomnessCollectiveFlip RandomMaterial (max_values: Some(1), max_size: Some(2594), added: 3089, mode: MaxEncodedLen)
    fn kick_off_authorship_validation() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `501`
        //  Estimated: `7103`
        // Minimum execution time: 37_010 nanoseconds.
        Weight::from_parts(38_090_000, 7103)
            .saturating_add(T::DbWeight::get().reads(6_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
}
