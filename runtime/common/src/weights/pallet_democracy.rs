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

//! Autogenerated weights for pallet_democracy
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: `2024-04-15`, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
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
// --pallet=pallet_democracy
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

/// Weight functions for pallet_democracy (automatically generated)
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_democracy::weights::WeightInfo for WeightInfo<T> {
    /// Storage: Democracy PublicPropCount (r:1 w:1)
    /// Proof: Democracy PublicPropCount (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
    /// Storage: Democracy PublicProps (r:1 w:1)
    /// Proof: Democracy PublicProps (max_values: Some(1), max_size: Some(16702), added: 17197, mode: MaxEncodedLen)
    /// Storage: Democracy Blacklist (r:1 w:0)
    /// Proof: Democracy Blacklist (max_values: None, max_size: Some(3242), added: 5717, mode: MaxEncodedLen)
    /// Storage: Democracy DepositOf (r:0 w:1)
    /// Proof: Democracy DepositOf (max_values: None, max_size: Some(3230), added: 5705, mode: MaxEncodedLen)
    fn propose() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `4835`
        //  Estimated: `23413`
        // Minimum execution time: 50_690 nanoseconds.
        Weight::from_parts(63_110_000, 23413)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    /// Storage: Democracy DepositOf (r:1 w:1)
    /// Proof: Democracy DepositOf (max_values: None, max_size: Some(3230), added: 5705, mode: MaxEncodedLen)
    fn second() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `3591`
        //  Estimated: `5705`
        // Minimum execution time: 43_520 nanoseconds.
        Weight::from_parts(45_040_000, 5705)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    /// Storage: Democracy ReferendumInfoOf (r:1 w:1)
    /// Proof: Democracy ReferendumInfoOf (max_values: None, max_size: Some(209), added: 2684, mode: MaxEncodedLen)
    /// Storage: Democracy VotingOf (r:1 w:1)
    /// Proof: Democracy VotingOf (max_values: None, max_size: Some(3799), added: 6274, mode: MaxEncodedLen)
    /// Storage: Balances Locks (r:1 w:1)
    /// Proof: Balances Locks (max_values: None, max_size: Some(1299), added: 3774, mode: MaxEncodedLen)
    fn vote_new() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `3500`
        //  Estimated: `12732`
        // Minimum execution time: 59_010 nanoseconds.
        Weight::from_parts(72_290_000, 12732)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    /// Storage: Democracy ReferendumInfoOf (r:1 w:1)
    /// Proof: Democracy ReferendumInfoOf (max_values: None, max_size: Some(209), added: 2684, mode: MaxEncodedLen)
    /// Storage: Democracy VotingOf (r:1 w:1)
    /// Proof: Democracy VotingOf (max_values: None, max_size: Some(3799), added: 6274, mode: MaxEncodedLen)
    /// Storage: Balances Locks (r:1 w:1)
    /// Proof: Balances Locks (max_values: None, max_size: Some(1299), added: 3774, mode: MaxEncodedLen)
    fn vote_existing() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `3522`
        //  Estimated: `12732`
        // Minimum execution time: 59_810 nanoseconds.
        Weight::from_parts(72_890_000, 12732)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    /// Storage: Democracy ReferendumInfoOf (r:1 w:1)
    /// Proof: Democracy ReferendumInfoOf (max_values: None, max_size: Some(209), added: 2684, mode: MaxEncodedLen)
    /// Storage: Democracy Cancellations (r:1 w:1)
    /// Proof: Democracy Cancellations (max_values: None, max_size: Some(33), added: 2508, mode: MaxEncodedLen)
    fn emergency_cancel() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `295`
        //  Estimated: `5192`
        // Minimum execution time: 26_390 nanoseconds.
        Weight::from_parts(32_000_000, 5192)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    /// Storage: Democracy PublicProps (r:1 w:1)
    /// Proof: Democracy PublicProps (max_values: Some(1), max_size: Some(16702), added: 17197, mode: MaxEncodedLen)
    /// Storage: Democracy DepositOf (r:1 w:1)
    /// Proof: Democracy DepositOf (max_values: None, max_size: Some(3230), added: 5705, mode: MaxEncodedLen)
    /// Storage: System Account (r:2 w:2)
    /// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
    /// Storage: Democracy NextExternal (r:1 w:1)
    /// Proof: Democracy NextExternal (max_values: Some(1), max_size: Some(132), added: 627, mode: MaxEncodedLen)
    /// Storage: Democracy ReferendumInfoOf (r:1 w:1)
    /// Proof: Democracy ReferendumInfoOf (max_values: None, max_size: Some(209), added: 2684, mode: MaxEncodedLen)
    /// Storage: Democracy Blacklist (r:0 w:1)
    /// Proof: Democracy Blacklist (max_values: None, max_size: Some(3242), added: 5717, mode: MaxEncodedLen)
    fn blacklist() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `6251`
        //  Estimated: `31427`
        // Minimum execution time: 112_670 nanoseconds.
        Weight::from_parts(139_060_000, 31427)
            .saturating_add(T::DbWeight::get().reads(6))
            .saturating_add(T::DbWeight::get().writes(7))
    }
    /// Storage: Democracy NextExternal (r:1 w:1)
    /// Proof: Democracy NextExternal (max_values: Some(1), max_size: Some(132), added: 627, mode: MaxEncodedLen)
    /// Storage: Democracy Blacklist (r:1 w:0)
    /// Proof: Democracy Blacklist (max_values: None, max_size: Some(3242), added: 5717, mode: MaxEncodedLen)
    fn external_propose() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `3419`
        //  Estimated: `6344`
        // Minimum execution time: 28_210 nanoseconds.
        Weight::from_parts(28_970_000, 6344)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    /// Storage: Democracy NextExternal (r:0 w:1)
    /// Proof: Democracy NextExternal (max_values: Some(1), max_size: Some(132), added: 627, mode: MaxEncodedLen)
    fn external_propose_majority() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 6_060 nanoseconds.
        Weight::from_parts(6_540_000, 0).saturating_add(T::DbWeight::get().writes(1))
    }
    /// Storage: Democracy NextExternal (r:0 w:1)
    /// Proof: Democracy NextExternal (max_values: Some(1), max_size: Some(132), added: 627, mode: MaxEncodedLen)
    fn external_propose_default() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 6_150 nanoseconds.
        Weight::from_parts(6_650_000, 0).saturating_add(T::DbWeight::get().writes(1))
    }
    /// Storage: Democracy NextExternal (r:1 w:1)
    /// Proof: Democracy NextExternal (max_values: Some(1), max_size: Some(132), added: 627, mode: MaxEncodedLen)
    /// Storage: Democracy ReferendumCount (r:1 w:1)
    /// Proof: Democracy ReferendumCount (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
    /// Storage: Democracy ReferendumInfoOf (r:0 w:1)
    /// Proof: Democracy ReferendumInfoOf (max_values: None, max_size: Some(209), added: 2684, mode: MaxEncodedLen)
    fn fast_track() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `179`
        //  Estimated: `1126`
        // Minimum execution time: 28_070 nanoseconds.
        Weight::from_parts(34_820_000, 1126)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    /// Storage: Democracy NextExternal (r:1 w:1)
    /// Proof: Democracy NextExternal (max_values: Some(1), max_size: Some(132), added: 627, mode: MaxEncodedLen)
    /// Storage: Democracy Blacklist (r:1 w:1)
    /// Proof: Democracy Blacklist (max_values: None, max_size: Some(3242), added: 5717, mode: MaxEncodedLen)
    fn veto_external() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `3448`
        //  Estimated: `6344`
        // Minimum execution time: 36_350 nanoseconds.
        Weight::from_parts(44_300_000, 6344)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    /// Storage: Democracy PublicProps (r:1 w:1)
    /// Proof: Democracy PublicProps (max_values: Some(1), max_size: Some(16702), added: 17197, mode: MaxEncodedLen)
    /// Storage: Democracy DepositOf (r:1 w:1)
    /// Proof: Democracy DepositOf (max_values: None, max_size: Some(3230), added: 5705, mode: MaxEncodedLen)
    /// Storage: System Account (r:2 w:2)
    /// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
    fn cancel_proposal() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `6122`
        //  Estimated: `28116`
        // Minimum execution time: 94_210 nanoseconds.
        Weight::from_parts(115_161_000, 28116)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(4))
    }
    /// Storage: Democracy ReferendumInfoOf (r:0 w:1)
    /// Proof: Democracy ReferendumInfoOf (max_values: None, max_size: Some(209), added: 2684, mode: MaxEncodedLen)
    fn cancel_referendum() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 11_520 nanoseconds.
        Weight::from_parts(13_900_000, 0).saturating_add(T::DbWeight::get().writes(1))
    }
    /// Storage: Democracy LowestUnbaked (r:1 w:1)
    /// Proof: Democracy LowestUnbaked (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
    /// Storage: Democracy ReferendumCount (r:1 w:0)
    /// Proof: Democracy ReferendumCount (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
    /// Storage: Democracy ReferendumInfoOf (r:99 w:0)
    /// Proof: Democracy ReferendumInfoOf (max_values: None, max_size: Some(209), added: 2684, mode: MaxEncodedLen)
    /// The range of component `r` is `[0, 99]`.
    fn on_initialize_base(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `174 + r * (125 ±0)`
        //  Estimated: `998 + r * (2684 ±0)`
        // Minimum execution time: 14_730 nanoseconds.
        Weight::from_parts(28_833_321, 998)
            // Standard Error: 37_839
            .saturating_add(Weight::from_parts(4_701_815, 0).saturating_mul(r.into()))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(r.into())))
            .saturating_add(T::DbWeight::get().writes(1))
            .saturating_add(Weight::from_parts(0, 2684).saturating_mul(r.into()))
    }
    /// Storage: Democracy LowestUnbaked (r:1 w:1)
    /// Proof: Democracy LowestUnbaked (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
    /// Storage: Democracy ReferendumCount (r:1 w:0)
    /// Proof: Democracy ReferendumCount (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
    /// Storage: Democracy LastTabledWasExternal (r:1 w:0)
    /// Proof: Democracy LastTabledWasExternal (max_values: Some(1), max_size: Some(1), added: 496, mode: MaxEncodedLen)
    /// Storage: Democracy NextExternal (r:1 w:0)
    /// Proof: Democracy NextExternal (max_values: Some(1), max_size: Some(132), added: 627, mode: MaxEncodedLen)
    /// Storage: Democracy PublicProps (r:1 w:0)
    /// Proof: Democracy PublicProps (max_values: Some(1), max_size: Some(16702), added: 17197, mode: MaxEncodedLen)
    /// Storage: Democracy ReferendumInfoOf (r:99 w:0)
    /// Proof: Democracy ReferendumInfoOf (max_values: None, max_size: Some(209), added: 2684, mode: MaxEncodedLen)
    /// The range of component `r` is `[0, 99]`.
    fn on_initialize_base_with_launch_period(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `174 + r * (125 ±0)`
        //  Estimated: `19318 + r * (2684 ±0)`
        // Minimum execution time: 18_190 nanoseconds.
        Weight::from_parts(29_838_845, 19318)
            // Standard Error: 39_893
            .saturating_add(Weight::from_parts(4_571_724, 0).saturating_mul(r.into()))
            .saturating_add(T::DbWeight::get().reads(5))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(r.into())))
            .saturating_add(T::DbWeight::get().writes(1))
            .saturating_add(Weight::from_parts(0, 2684).saturating_mul(r.into()))
    }
    /// Storage: Democracy VotingOf (r:3 w:3)
    /// Proof: Democracy VotingOf (max_values: None, max_size: Some(3799), added: 6274, mode: MaxEncodedLen)
    /// Storage: Democracy ReferendumInfoOf (r:99 w:99)
    /// Proof: Democracy ReferendumInfoOf (max_values: None, max_size: Some(209), added: 2684, mode: MaxEncodedLen)
    /// Storage: Balances Locks (r:1 w:1)
    /// Proof: Balances Locks (max_values: None, max_size: Some(1299), added: 3774, mode: MaxEncodedLen)
    /// The range of component `r` is `[0, 99]`.
    fn delegate(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `897 + r * (147 ±0)`
        //  Estimated: `22596 + r * (2684 ±0)`
        // Minimum execution time: 55_960 nanoseconds.
        Weight::from_parts(72_937_258, 22596)
            // Standard Error: 49_578
            .saturating_add(Weight::from_parts(6_045_111, 0).saturating_mul(r.into()))
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(r.into())))
            .saturating_add(T::DbWeight::get().writes(4))
            .saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(r.into())))
            .saturating_add(Weight::from_parts(0, 2684).saturating_mul(r.into()))
    }
    /// Storage: Democracy VotingOf (r:2 w:2)
    /// Proof: Democracy VotingOf (max_values: None, max_size: Some(3799), added: 6274, mode: MaxEncodedLen)
    /// Storage: Democracy ReferendumInfoOf (r:99 w:99)
    /// Proof: Democracy ReferendumInfoOf (max_values: None, max_size: Some(209), added: 2684, mode: MaxEncodedLen)
    /// The range of component `r` is `[0, 99]`.
    fn undelegate(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `522 + r * (147 ±0)`
        //  Estimated: `12548 + r * (2684 ±0)`
        // Minimum execution time: 33_530 nanoseconds.
        Weight::from_parts(45_434_419, 12548)
            // Standard Error: 48_431
            .saturating_add(Weight::from_parts(6_200_443, 0).saturating_mul(r.into()))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(r.into())))
            .saturating_add(T::DbWeight::get().writes(2))
            .saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(r.into())))
            .saturating_add(Weight::from_parts(0, 2684).saturating_mul(r.into()))
    }
    /// Storage: Democracy PublicProps (r:0 w:1)
    /// Proof: Democracy PublicProps (max_values: Some(1), max_size: Some(16702), added: 17197, mode: MaxEncodedLen)
    fn clear_public_proposals() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 4_870 nanoseconds.
        Weight::from_parts(7_050_000, 0).saturating_add(T::DbWeight::get().writes(1))
    }
    /// Storage: Democracy VotingOf (r:1 w:1)
    /// Proof: Democracy VotingOf (max_values: None, max_size: Some(3799), added: 6274, mode: MaxEncodedLen)
    /// Storage: Balances Locks (r:1 w:1)
    /// Proof: Balances Locks (max_values: None, max_size: Some(1299), added: 3774, mode: MaxEncodedLen)
    /// Storage: System Account (r:1 w:1)
    /// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
    /// The range of component `r` is `[0, 99]`.
    fn unlock_remove(_r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `554`
        //  Estimated: `12655`
        // Minimum execution time: 31_950 nanoseconds.
        Weight::from_parts(46_451_047, 12655)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    /// Storage: Democracy VotingOf (r:1 w:1)
    /// Proof: Democracy VotingOf (max_values: None, max_size: Some(3799), added: 6274, mode: MaxEncodedLen)
    /// Storage: Balances Locks (r:1 w:1)
    /// Proof: Balances Locks (max_values: None, max_size: Some(1299), added: 3774, mode: MaxEncodedLen)
    /// Storage: System Account (r:1 w:1)
    /// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
    /// The range of component `r` is `[0, 99]`.
    fn unlock_set(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `555 + r * (22 ±0)`
        //  Estimated: `12655`
        // Minimum execution time: 34_100 nanoseconds.
        Weight::from_parts(44_669_466, 12655)
            // Standard Error: 5_077
            .saturating_add(Weight::from_parts(42_607, 0).saturating_mul(r.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    /// Storage: Democracy ReferendumInfoOf (r:1 w:1)
    /// Proof: Democracy ReferendumInfoOf (max_values: None, max_size: Some(209), added: 2684, mode: MaxEncodedLen)
    /// Storage: Democracy VotingOf (r:1 w:1)
    /// Proof: Democracy VotingOf (max_values: None, max_size: Some(3799), added: 6274, mode: MaxEncodedLen)
    /// The range of component `r` is `[1, 100]`.
    fn remove_vote(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `760 + r * (26 ±0)`
        //  Estimated: `8958`
        // Minimum execution time: 26_160 nanoseconds.
        Weight::from_parts(34_737_151, 8958)
            // Standard Error: 4_560
            .saturating_add(Weight::from_parts(64_111, 0).saturating_mul(r.into()))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    /// Storage: Democracy ReferendumInfoOf (r:1 w:1)
    /// Proof: Democracy ReferendumInfoOf (max_values: None, max_size: Some(209), added: 2684, mode: MaxEncodedLen)
    /// Storage: Democracy VotingOf (r:1 w:1)
    /// Proof: Democracy VotingOf (max_values: None, max_size: Some(3799), added: 6274, mode: MaxEncodedLen)
    /// The range of component `r` is `[1, 100]`.
    fn remove_other_vote(r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `760 + r * (26 ±0)`
        //  Estimated: `8958`
        // Minimum execution time: 25_821 nanoseconds.
        Weight::from_parts(32_571_293, 8958)
            // Standard Error: 3_984
            .saturating_add(Weight::from_parts(94_520, 0).saturating_mul(r.into()))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
}
