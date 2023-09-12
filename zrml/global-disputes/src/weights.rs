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
//! DATE: `2023-09-11`, STEPS: `10`, REPEAT: `1000`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `zeitgeist-benchmark`, CPU: `AMD EPYC 7601 32-Core Processor`
//! EXECUTION: `Some(Wasm)`, WASM-EXECUTION: `Compiled`, CHAIN: `Some("dev")`, DB CACHE: `1024`

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
// --header=./HEADER_GPL3
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
    fn reward_outcome_owner_shared_possession(o: u32) -> Weight;
    fn reward_outcome_owner_paid_possession() -> Weight;
    fn purge_outcomes(k: u32, o: u32) -> Weight;
    fn refund_vote_fees(k: u32, o: u32) -> Weight;
}

/// Weight functions for zrml_global_disputes (automatically generated)
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfoZeitgeist for WeightInfo<T> {
    /// Storage: GlobalDisputes GlobalDisputesInfo (r:1 w:1)
    /// Proof: GlobalDisputes GlobalDisputesInfo (max_values: None, max_size: Some(396), added: 2871, mode: MaxEncodedLen)
    /// Storage: GlobalDisputes Outcomes (r:1 w:1)
    /// Proof: GlobalDisputes Outcomes (max_values: None, max_size: Some(395), added: 2870, mode: MaxEncodedLen)
    /// Storage: GlobalDisputes Locks (r:1 w:1)
    /// Proof: GlobalDisputes Locks (max_values: None, max_size: Some(1641), added: 4116, mode: MaxEncodedLen)
    /// Storage: Balances Locks (r:1 w:1)
    /// Proof: Balances Locks (max_values: None, max_size: Some(1299), added: 3774, mode: MaxEncodedLen)
    fn vote_on_outcome(_o: u32, v: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `556 + o * (26 ±0) + v * (32 ±0)`
        //  Estimated: `13631`
        // Minimum execution time: 57_340 nanoseconds.
        Weight::from_parts(64_978_934, 13631)
            // Standard Error: 3_950
            .saturating_add(Weight::from_ref_time(121_314).saturating_mul(v.into()))
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().writes(4_u64))
    }
    /// Storage: GlobalDisputes Locks (r:1 w:1)
    /// Proof: GlobalDisputes Locks (max_values: None, max_size: Some(1641), added: 4116, mode: MaxEncodedLen)
    /// Storage: GlobalDisputes GlobalDisputesInfo (r:50 w:0)
    /// Proof: GlobalDisputes GlobalDisputesInfo (max_values: None, max_size: Some(396), added: 2871, mode: MaxEncodedLen)
    /// Storage: Balances Locks (r:1 w:1)
    /// Proof: Balances Locks (max_values: None, max_size: Some(1299), added: 3774, mode: MaxEncodedLen)
    /// Storage: System Account (r:1 w:0)
    /// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
    fn unlock_vote_balance_set(l: u32, o: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0 + l * (467 ±0) + o * (1600 ±0)`
        //  Estimated: `10497 + l * (2871 ±0)`
        // Minimum execution time: 31_920 nanoseconds.
        Weight::from_parts(37_861_017, 10497)
            // Standard Error: 10_688
            .saturating_add(Weight::from_ref_time(4_091_299).saturating_mul(l.into()))
            // Standard Error: 59_834
            .saturating_add(Weight::from_ref_time(813_711).saturating_mul(o.into()))
            .saturating_add(T::DbWeight::get().reads(3_u64))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(l.into())))
            .saturating_add(T::DbWeight::get().writes(2_u64))
            .saturating_add(Weight::from_proof_size(2871).saturating_mul(l.into()))
    }
    /// Storage: GlobalDisputes Locks (r:1 w:1)
    /// Proof: GlobalDisputes Locks (max_values: None, max_size: Some(1641), added: 4116, mode: MaxEncodedLen)
    /// Storage: GlobalDisputes GlobalDisputesInfo (r:50 w:0)
    /// Proof: GlobalDisputes GlobalDisputesInfo (max_values: None, max_size: Some(396), added: 2871, mode: MaxEncodedLen)
    /// Storage: Balances Locks (r:1 w:1)
    /// Proof: Balances Locks (max_values: None, max_size: Some(1299), added: 3774, mode: MaxEncodedLen)
    /// Storage: System Account (r:1 w:0)
    /// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
    fn unlock_vote_balance_remove(l: u32, o: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0 + l * (451 ±0) + o * (1600 ±0)`
        //  Estimated: `10497 + l * (2871 ±0)`
        // Minimum execution time: 32_400 nanoseconds.
        Weight::from_parts(20_243_436, 10497)
            // Standard Error: 10_469
            .saturating_add(Weight::from_ref_time(3_915_774).saturating_mul(l.into()))
            // Standard Error: 58_607
            .saturating_add(Weight::from_ref_time(2_533_975).saturating_mul(o.into()))
            .saturating_add(T::DbWeight::get().reads(3_u64))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(l.into())))
            .saturating_add(T::DbWeight::get().writes(2_u64))
            .saturating_add(Weight::from_proof_size(2871).saturating_mul(l.into()))
    }
    /// Storage: MarketCommons Markets (r:1 w:0)
    /// Proof: MarketCommons Markets (max_values: None, max_size: Some(541), added: 3016, mode: MaxEncodedLen)
    /// Storage: GlobalDisputes GlobalDisputesInfo (r:1 w:1)
    /// Proof: GlobalDisputes GlobalDisputesInfo (max_values: None, max_size: Some(396), added: 2871, mode: MaxEncodedLen)
    /// Storage: GlobalDisputes Outcomes (r:1 w:1)
    /// Proof: GlobalDisputes Outcomes (max_values: None, max_size: Some(395), added: 2870, mode: MaxEncodedLen)
    /// Storage: System Account (r:1 w:1)
    /// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
    fn add_vote_outcome(w: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `686 + w * (32 ±0)`
        //  Estimated: `11364`
        // Minimum execution time: 67_610 nanoseconds.
        Weight::from_parts(77_082_434, 11364)
            // Standard Error: 27_021
            .saturating_add(Weight::from_ref_time(566_268).saturating_mul(w.into()))
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().writes(3_u64))
    }
    /// Storage: GlobalDisputes Outcomes (r:1 w:0)
    /// Proof: GlobalDisputes Outcomes (max_values: None, max_size: Some(395), added: 2870, mode: MaxEncodedLen)
    /// Storage: GlobalDisputes GlobalDisputesInfo (r:1 w:0)
    /// Proof: GlobalDisputes GlobalDisputesInfo (max_values: None, max_size: Some(396), added: 2871, mode: MaxEncodedLen)
    /// Storage: System Account (r:11 w:11)
    /// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
    fn reward_outcome_owner_shared_possession(o: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `490 + o * (41 ±0)`
        //  Estimated: `8869 + o * (2702 ±6)`
        // Minimum execution time: 66_040 nanoseconds.
        Weight::from_parts(59_723_714, 8869)
            // Standard Error: 98_936
            .saturating_add(Weight::from_ref_time(30_343_508).saturating_mul(o.into()))
            .saturating_add(T::DbWeight::get().reads(3_u64))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(o.into())))
            .saturating_add(T::DbWeight::get().writes(1_u64))
            .saturating_add(Weight::from_proof_size(2702).saturating_mul(o.into()))
    }
    /// Storage: GlobalDisputes Outcomes (r:1 w:0)
    /// Proof: GlobalDisputes Outcomes (max_values: None, max_size: Some(395), added: 2870, mode: MaxEncodedLen)
    /// Storage: GlobalDisputes GlobalDisputesInfo (r:1 w:0)
    /// Proof: GlobalDisputes GlobalDisputesInfo (max_values: None, max_size: Some(396), added: 2871, mode: MaxEncodedLen)
    /// Storage: System Account (r:2 w:2)
    /// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
    fn reward_outcome_owner_paid_possession() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `537`
        //  Estimated: `10955`
        // Minimum execution time: 65_341 nanoseconds.
        Weight::from_parts(74_930_000, 10955)
            .saturating_add(T::DbWeight::get().reads(4_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    /// Storage: GlobalDisputes GlobalDisputesInfo (r:1 w:1)
    /// Proof: GlobalDisputes GlobalDisputesInfo (max_values: None, max_size: Some(396), added: 2871, mode: MaxEncodedLen)
    /// Storage: GlobalDisputes Outcomes (r:250 w:249)
    /// Proof: GlobalDisputes Outcomes (max_values: None, max_size: Some(395), added: 2870, mode: MaxEncodedLen)
    fn purge_outcomes(k: u32, _o: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `407 + k * (122 ±0) + o * (32 ±0)`
        //  Estimated: `8611 + k * (2870 ±0)`
        // Minimum execution time: 75_541 nanoseconds.
        Weight::from_parts(49_011_584, 8611)
            // Standard Error: 16_439
            .saturating_add(Weight::from_ref_time(18_148_873).saturating_mul(k.into()))
            .saturating_add(T::DbWeight::get().reads(3_u64))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(k.into())))
            .saturating_add(T::DbWeight::get().writes(2_u64))
            .saturating_add(Weight::from_proof_size(2870).saturating_mul(k.into()))
    }
    /// Storage: GlobalDisputes GlobalDisputesInfo (r:1 w:0)
    /// Proof: GlobalDisputes GlobalDisputesInfo (max_values: None, max_size: Some(396), added: 2871, mode: MaxEncodedLen)
    /// Storage: GlobalDisputes Outcomes (r:250 w:249)
    /// Proof: GlobalDisputes Outcomes (max_values: None, max_size: Some(395), added: 2870, mode: MaxEncodedLen)
    fn refund_vote_fees(k: u32, _o: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `407 + k * (122 ±0) + o * (32 ±0)`
        //  Estimated: `8611 + k * (2870 ±0)`
        // Minimum execution time: 70_531 nanoseconds.
        Weight::from_parts(66_128_182, 8611)
            // Standard Error: 19_994
            .saturating_add(Weight::from_ref_time(18_221_796).saturating_mul(k.into()))
            .saturating_add(T::DbWeight::get().reads(3_u64))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(k.into())))
            .saturating_add(T::DbWeight::get().writes(1_u64))
            .saturating_add(Weight::from_proof_size(2870).saturating_mul(k.into()))
    }
}
