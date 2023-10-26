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
//! DATE: `2023-10-26`, STEPS: `10`, REPEAT: `1000`, LOW RANGE: `[]`, HIGH RANGE: `[]`
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
    /// The range of component `o` is `[2, 10]`.
    /// The range of component `v` is `[0, 49]`.
    fn vote_on_outcome(o: u32, v: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `556 + o * (26 ±0) + v * (32 ±0)`
        //  Estimated: `13631`
        // Minimum execution time: 56_570 nanoseconds.
        Weight::from_parts(62_319_654, 13631)
            // Standard Error: 20_343
            .saturating_add(Weight::from_parts(133_505, 0).saturating_mul(o.into()))
            // Standard Error: 3_569
            .saturating_add(Weight::from_parts(91_527, 0).saturating_mul(v.into()))
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(4))
    }
    /// Storage: GlobalDisputes Locks (r:1 w:1)
    /// Proof: GlobalDisputes Locks (max_values: None, max_size: Some(1641), added: 4116, mode: MaxEncodedLen)
    /// Storage: GlobalDisputes GlobalDisputesInfo (r:50 w:0)
    /// Proof: GlobalDisputes GlobalDisputesInfo (max_values: None, max_size: Some(396), added: 2871, mode: MaxEncodedLen)
    /// Storage: Balances Locks (r:1 w:1)
    /// Proof: Balances Locks (max_values: None, max_size: Some(1299), added: 3774, mode: MaxEncodedLen)
    /// Storage: System Account (r:1 w:0)
    /// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
    /// The range of component `l` is `[0, 50]`.
    /// The range of component `o` is `[1, 10]`.
    fn unlock_vote_balance_set(l: u32, o: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0 + l * (467 ±0) + o * (1600 ±0)`
        //  Estimated: `10497 + l * (2871 ±0)`
        // Minimum execution time: 31_550 nanoseconds.
        Weight::from_parts(35_536_478, 10497)
            // Standard Error: 11_671
            .saturating_add(Weight::from_parts(3_910_915, 0).saturating_mul(l.into()))
            // Standard Error: 65_337
            .saturating_add(Weight::from_parts(1_067_736, 0).saturating_mul(o.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(l.into())))
            .saturating_add(T::DbWeight::get().writes(2))
            .saturating_add(Weight::from_parts(0, 2871).saturating_mul(l.into()))
    }
    /// Storage: GlobalDisputes Locks (r:1 w:1)
    /// Proof: GlobalDisputes Locks (max_values: None, max_size: Some(1641), added: 4116, mode: MaxEncodedLen)
    /// Storage: GlobalDisputes GlobalDisputesInfo (r:50 w:0)
    /// Proof: GlobalDisputes GlobalDisputesInfo (max_values: None, max_size: Some(396), added: 2871, mode: MaxEncodedLen)
    /// Storage: Balances Locks (r:1 w:1)
    /// Proof: Balances Locks (max_values: None, max_size: Some(1299), added: 3774, mode: MaxEncodedLen)
    /// Storage: System Account (r:1 w:0)
    /// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
    /// The range of component `l` is `[0, 50]`.
    /// The range of component `o` is `[1, 10]`.
    fn unlock_vote_balance_remove(l: u32, _o: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0 + l * (451 ±0) + o * (1600 ±0)`
        //  Estimated: `10497 + l * (2871 ±0)`
        // Minimum execution time: 31_640 nanoseconds.
        Weight::from_parts(38_456_566, 10497)
            // Standard Error: 10_816
            .saturating_add(Weight::from_parts(3_883_534, 0).saturating_mul(l.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(l.into())))
            .saturating_add(T::DbWeight::get().writes(2))
            .saturating_add(Weight::from_parts(0, 2871).saturating_mul(l.into()))
    }
    /// Storage: MarketCommons Markets (r:1 w:0)
    /// Proof: MarketCommons Markets (max_values: None, max_size: Some(678), added: 3153, mode: MaxEncodedLen)
    /// Storage: GlobalDisputes GlobalDisputesInfo (r:1 w:1)
    /// Proof: GlobalDisputes GlobalDisputesInfo (max_values: None, max_size: Some(396), added: 2871, mode: MaxEncodedLen)
    /// Storage: GlobalDisputes Outcomes (r:1 w:1)
    /// Proof: GlobalDisputes Outcomes (max_values: None, max_size: Some(395), added: 2870, mode: MaxEncodedLen)
    /// Storage: System Account (r:1 w:1)
    /// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
    /// The range of component `w` is `[1, 10]`.
    fn add_vote_outcome(w: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `690 + w * (32 ±0)`
        //  Estimated: `11501`
        // Minimum execution time: 67_621 nanoseconds.
        Weight::from_parts(77_496_340, 11501)
            // Standard Error: 25_163
            .saturating_add(Weight::from_parts(48_228, 0).saturating_mul(w.into()))
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    /// Storage: GlobalDisputes Outcomes (r:1 w:0)
    /// Proof: GlobalDisputes Outcomes (max_values: None, max_size: Some(395), added: 2870, mode: MaxEncodedLen)
    /// Storage: GlobalDisputes GlobalDisputesInfo (r:1 w:0)
    /// Proof: GlobalDisputes GlobalDisputesInfo (max_values: None, max_size: Some(396), added: 2871, mode: MaxEncodedLen)
    /// Storage: System Account (r:11 w:11)
    /// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
    /// The range of component `o` is `[1, 10]`.
    fn reward_outcome_owner_shared_possession(o: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `490 + o * (41 ±0)`
        //  Estimated: `8869 + o * (2702 ±6)`
        // Minimum execution time: 66_650 nanoseconds.
        Weight::from_parts(53_409_131, 8869)
            // Standard Error: 97_889
            .saturating_add(Weight::from_parts(30_309_941, 0).saturating_mul(o.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(o.into())))
            .saturating_add(T::DbWeight::get().writes(1))
            .saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(o.into())))
            .saturating_add(Weight::from_parts(0, 2702).saturating_mul(o.into()))
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
        // Minimum execution time: 66_220 nanoseconds.
        Weight::from_parts(75_191_000, 10955)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    /// Storage: GlobalDisputes GlobalDisputesInfo (r:1 w:1)
    /// Proof: GlobalDisputes GlobalDisputesInfo (max_values: None, max_size: Some(396), added: 2871, mode: MaxEncodedLen)
    /// Storage: GlobalDisputes Outcomes (r:250 w:249)
    /// Proof: GlobalDisputes Outcomes (max_values: None, max_size: Some(395), added: 2870, mode: MaxEncodedLen)
    /// The range of component `k` is `[2, 248]`.
    /// The range of component `o` is `[1, 10]`.
    fn purge_outcomes(k: u32, _o: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `407 + k * (122 ±0) + o * (32 ±0)`
        //  Estimated: `8611 + k * (2870 ±0)`
        // Minimum execution time: 76_340 nanoseconds.
        Weight::from_parts(8_305_894, 8611)
            // Standard Error: 21_012
            .saturating_add(Weight::from_parts(18_168_686, 0).saturating_mul(k.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(k.into())))
            .saturating_add(T::DbWeight::get().writes(2))
            .saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(k.into())))
            .saturating_add(Weight::from_parts(0, 2870).saturating_mul(k.into()))
    }
    /// Storage: GlobalDisputes GlobalDisputesInfo (r:1 w:0)
    /// Proof: GlobalDisputes GlobalDisputesInfo (max_values: None, max_size: Some(396), added: 2871, mode: MaxEncodedLen)
    /// Storage: GlobalDisputes Outcomes (r:250 w:249)
    /// Proof: GlobalDisputes Outcomes (max_values: None, max_size: Some(395), added: 2870, mode: MaxEncodedLen)
    /// The range of component `k` is `[2, 248]`.
    /// The range of component `o` is `[1, 10]`.
    fn refund_vote_fees(k: u32, _o: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `407 + k * (122 ±0) + o * (32 ±0)`
        //  Estimated: `8611 + k * (2870 ±0)`
        // Minimum execution time: 73_140 nanoseconds.
        Weight::from_parts(232_428_016, 8611)
            // Standard Error: 18_813
            .saturating_add(Weight::from_parts(17_740_443, 0).saturating_mul(k.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(k.into())))
            .saturating_add(T::DbWeight::get().writes(1))
            .saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(k.into())))
            .saturating_add(Weight::from_parts(0, 2870).saturating_mul(k.into()))
    }
}
