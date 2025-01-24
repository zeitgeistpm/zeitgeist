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

//! Autogenerated weights for zrml_court
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: `2024-08-27`, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `zeitgeist-benchmark`, CPU: `AMD EPYC 7601 32-Core Processor`
//! EXECUTION: ``, WASM-EXECUTION: `Compiled`, CHAIN: `Some("dev")`, DB CACHE: `1024`

// Executed Command:
// ./target/production/zeitgeist
// benchmark
// pallet
// --chain=dev
// --steps=50
// --repeat=20
// --pallet=zrml_court
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --template=./misc/weight_template.hbs
// --header=./HEADER_GPL3
// --output=./zrml/court/src/weights.rs

#![allow(unused_parens)]
#![allow(unused_imports)]

use core::marker::PhantomData;
use frame_support::{traits::Get, weights::Weight};

///  Trait containing the required functions for weight retrival within
/// zrml_court (automatically generated)
pub trait WeightInfoZeitgeist {
    fn join_court(j: u32) -> Weight;
    fn delegate(j: u32, d: u32) -> Weight;
    fn prepare_exit_court(j: u32) -> Weight;
    fn exit_court_remove() -> Weight;
    fn exit_court_set() -> Weight;
    fn vote(d: u32) -> Weight;
    fn denounce_vote(d: u32) -> Weight;
    fn reveal_vote(d: u32) -> Weight;
    fn appeal(j: u32, a: u32, r: u32, f: u32) -> Weight;
    fn reassign_court_stakes(d: u32) -> Weight;
    fn set_inflation() -> Weight;
    fn handle_inflation(j: u32) -> Weight;
    fn select_participants(a: u32) -> Weight;
    fn on_dispute(j: u32, r: u32) -> Weight;
    fn on_resolution(d: u32) -> Weight;
    fn exchange(a: u32) -> Weight;
    fn get_auto_resolve() -> Weight;
    fn has_failed() -> Weight;
    fn on_global_dispute(a: u32, d: u32) -> Weight;
    fn clear(d: u32) -> Weight;
}

/// Weight functions for zrml_court (automatically generated)
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfoZeitgeist for WeightInfo<T> {
    /// Storage: `Court::CourtPool` (r:1 w:1)
    /// Proof: `Court::CourtPool` (`max_values`: Some(1), `max_size`: Some(96002), added: 96497, mode: `MaxEncodedLen`)
    /// Storage: `Court::Participants` (r:1 w:1)
    /// Proof: `Court::Participants` (`max_values`: None, `max_size`: Some(251), added: 2726, mode: `MaxEncodedLen`)
    /// Storage: `Balances::Locks` (r:1 w:1)
    /// Proof: `Balances::Locks` (`max_values`: None, `max_size`: Some(1299), added: 3774, mode: `MaxEncodedLen`)
    /// Storage: `Balances::Freezes` (r:1 w:0)
    /// Proof: `Balances::Freezes` (`max_values`: None, `max_size`: Some(65), added: 2540, mode: `MaxEncodedLen`)
    /// The range of component `j` is `[0, 999]`.
    fn join_court(j: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1082 + j * (96 ±0)`
        //  Estimated: `97487`
        // Minimum execution time: 45_531 nanoseconds.
        Weight::from_parts(55_818_024, 97487)
            // Standard Error: 429
            .saturating_add(Weight::from_parts(87_585, 0).saturating_mul(j.into()))
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    /// Storage: `Court::CourtPool` (r:1 w:1)
    /// Proof: `Court::CourtPool` (`max_values`: Some(1), `max_size`: Some(96002), added: 96497, mode: `MaxEncodedLen`)
    /// Storage: `Court::Participants` (r:6 w:1)
    /// Proof: `Court::Participants` (`max_values`: None, `max_size`: Some(251), added: 2726, mode: `MaxEncodedLen`)
    /// Storage: `Balances::Locks` (r:1 w:1)
    /// Proof: `Balances::Locks` (`max_values`: None, `max_size`: Some(1299), added: 3774, mode: `MaxEncodedLen`)
    /// Storage: `Balances::Freezes` (r:1 w:0)
    /// Proof: `Balances::Freezes` (`max_values`: None, `max_size`: Some(65), added: 2540, mode: `MaxEncodedLen`)
    /// The range of component `j` is `[5, 999]`.
    /// The range of component `d` is `[1, 5]`.
    fn delegate(j: u32, d: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0 + d * (651 ±0) + j * (98 ±0)`
        //  Estimated: `97487 + d * (2726 ±0)`
        // Minimum execution time: 70_523 nanoseconds.
        Weight::from_parts(51_742_170, 97487)
            // Standard Error: 637
            .saturating_add(Weight::from_parts(118_140, 0).saturating_mul(j.into()))
            // Standard Error: 138_375
            .saturating_add(Weight::from_parts(6_736_864, 0).saturating_mul(d.into()))
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(d.into())))
            .saturating_add(T::DbWeight::get().writes(3))
            .saturating_add(Weight::from_parts(0, 2726).saturating_mul(d.into()))
    }
    /// Storage: `Court::Participants` (r:1 w:1)
    /// Proof: `Court::Participants` (`max_values`: None, `max_size`: Some(251), added: 2726, mode: `MaxEncodedLen`)
    /// Storage: `Court::CourtPool` (r:1 w:1)
    /// Proof: `Court::CourtPool` (`max_values`: Some(1), `max_size`: Some(96002), added: 96497, mode: `MaxEncodedLen`)
    /// The range of component `j` is `[0, 999]`.
    fn prepare_exit_court(j: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1020 + j * (96 ±0)`
        //  Estimated: `97487`
        // Minimum execution time: 23_170 nanoseconds.
        Weight::from_parts(31_695_050, 97487)
            // Standard Error: 292
            .saturating_add(Weight::from_parts(73_562, 0).saturating_mul(j.into()))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    /// Storage: `Court::Participants` (r:1 w:1)
    /// Proof: `Court::Participants` (`max_values`: None, `max_size`: Some(251), added: 2726, mode: `MaxEncodedLen`)
    /// Storage: `Balances::Locks` (r:1 w:1)
    /// Proof: `Balances::Locks` (`max_values`: None, `max_size`: Some(1299), added: 3774, mode: `MaxEncodedLen`)
    /// Storage: `Balances::Freezes` (r:1 w:0)
    /// Proof: `Balances::Freezes` (`max_values`: None, `max_size`: Some(65), added: 2540, mode: `MaxEncodedLen`)
    fn exit_court_remove() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `276`
        //  Estimated: `4764`
        // Minimum execution time: 43_861 nanoseconds.
        Weight::from_parts(45_190_000, 4764)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    /// Storage: `Court::Participants` (r:1 w:1)
    /// Proof: `Court::Participants` (`max_values`: None, `max_size`: Some(251), added: 2726, mode: `MaxEncodedLen`)
    /// Storage: `Balances::Locks` (r:1 w:1)
    /// Proof: `Balances::Locks` (`max_values`: None, `max_size`: Some(1299), added: 3774, mode: `MaxEncodedLen`)
    /// Storage: `Balances::Freezes` (r:1 w:0)
    /// Proof: `Balances::Freezes` (`max_values`: None, `max_size`: Some(65), added: 2540, mode: `MaxEncodedLen`)
    fn exit_court_set() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `276`
        //  Estimated: `4764`
        // Minimum execution time: 35_870 nanoseconds.
        Weight::from_parts(36_931_000, 4764)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    /// Storage: `Court::Courts` (r:1 w:0)
    /// Proof: `Court::Courts` (`max_values`: None, `max_size`: Some(349), added: 2824, mode: `MaxEncodedLen`)
    /// Storage: `Court::SelectedDraws` (r:1 w:1)
    /// Proof: `Court::SelectedDraws` (`max_values`: None, `max_size`: Some(149974), added: 152449, mode: `MaxEncodedLen`)
    /// The range of component `d` is `[1, 510]`.
    fn vote(d: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `387 + d * (53 ±0)`
        //  Estimated: `153439`
        // Minimum execution time: 34_871 nanoseconds.
        Weight::from_parts(37_428_708, 153439)
            // Standard Error: 381
            .saturating_add(Weight::from_parts(103_937, 0).saturating_mul(d.into()))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    /// Storage: `Court::CourtIdToMarketId` (r:1 w:0)
    /// Proof: `Court::CourtIdToMarketId` (`max_values`: None, `max_size`: Some(40), added: 2515, mode: `MaxEncodedLen`)
    /// Storage: `MarketCommons::Markets` (r:1 w:0)
    /// Proof: `MarketCommons::Markets` (`max_values`: None, `max_size`: Some(694), added: 3169, mode: `MaxEncodedLen`)
    /// Storage: `Court::Participants` (r:1 w:0)
    /// Proof: `Court::Participants` (`max_values`: None, `max_size`: Some(251), added: 2726, mode: `MaxEncodedLen`)
    /// Storage: `Court::Courts` (r:1 w:0)
    /// Proof: `Court::Courts` (`max_values`: None, `max_size`: Some(349), added: 2824, mode: `MaxEncodedLen`)
    /// Storage: `Court::SelectedDraws` (r:1 w:1)
    /// Proof: `Court::SelectedDraws` (`max_values`: None, `max_size`: Some(149974), added: 152449, mode: `MaxEncodedLen`)
    /// The range of component `d` is `[1, 510]`.
    fn denounce_vote(d: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1496 + d * (53 ±0)`
        //  Estimated: `153439`
        // Minimum execution time: 47_831 nanoseconds.
        Weight::from_parts(54_598_122, 153439)
            // Standard Error: 621
            .saturating_add(Weight::from_parts(126_668, 0).saturating_mul(d.into()))
            .saturating_add(T::DbWeight::get().reads(5))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    /// Storage: `Court::CourtIdToMarketId` (r:1 w:0)
    /// Proof: `Court::CourtIdToMarketId` (`max_values`: None, `max_size`: Some(40), added: 2515, mode: `MaxEncodedLen`)
    /// Storage: `MarketCommons::Markets` (r:1 w:0)
    /// Proof: `MarketCommons::Markets` (`max_values`: None, `max_size`: Some(694), added: 3169, mode: `MaxEncodedLen`)
    /// Storage: `Court::Participants` (r:1 w:0)
    /// Proof: `Court::Participants` (`max_values`: None, `max_size`: Some(251), added: 2726, mode: `MaxEncodedLen`)
    /// Storage: `Court::Courts` (r:1 w:0)
    /// Proof: `Court::Courts` (`max_values`: None, `max_size`: Some(349), added: 2824, mode: `MaxEncodedLen`)
    /// Storage: `Court::SelectedDraws` (r:1 w:1)
    /// Proof: `Court::SelectedDraws` (`max_values`: None, `max_size`: Some(149974), added: 152449, mode: `MaxEncodedLen`)
    /// The range of component `d` is `[1, 510]`.
    fn reveal_vote(d: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `2034 + d * (53 ±0)`
        //  Estimated: `153439`
        // Minimum execution time: 61_301 nanoseconds.
        Weight::from_parts(63_924_015, 153439)
            // Standard Error: 321
            .saturating_add(Weight::from_parts(103_527, 0).saturating_mul(d.into()))
            .saturating_add(T::DbWeight::get().reads(5))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    /// Storage: `Court::Courts` (r:1 w:1)
    /// Proof: `Court::Courts` (`max_values`: None, `max_size`: Some(349), added: 2824, mode: `MaxEncodedLen`)
    /// Storage: `Court::CourtIdToMarketId` (r:1 w:0)
    /// Proof: `Court::CourtIdToMarketId` (`max_values`: None, `max_size`: Some(40), added: 2515, mode: `MaxEncodedLen`)
    /// Storage: `MarketCommons::Markets` (r:1 w:0)
    /// Proof: `MarketCommons::Markets` (`max_values`: None, `max_size`: Some(694), added: 3169, mode: `MaxEncodedLen`)
    /// Storage: `Court::SelectedDraws` (r:1 w:1)
    /// Proof: `Court::SelectedDraws` (`max_values`: None, `max_size`: Some(149974), added: 152449, mode: `MaxEncodedLen`)
    /// Storage: `Court::CourtPool` (r:1 w:1)
    /// Proof: `Court::CourtPool` (`max_values`: Some(1), `max_size`: Some(96002), added: 96497, mode: `MaxEncodedLen`)
    /// Storage: `Court::SelectionNonce` (r:1 w:1)
    /// Proof: `Court::SelectionNonce` (`max_values`: Some(1), `max_size`: Some(8), added: 503, mode: `MaxEncodedLen`)
    /// Storage: `RandomnessCollectiveFlip::RandomMaterial` (r:1 w:0)
    /// Proof: `RandomnessCollectiveFlip::RandomMaterial` (`max_values`: Some(1), `max_size`: Some(2594), added: 3089, mode: `MaxEncodedLen`)
    /// Storage: `Court::Participants` (r:347 w:343)
    /// Proof: `Court::Participants` (`max_values`: None, `max_size`: Some(251), added: 2726, mode: `MaxEncodedLen`)
    /// Storage: `Court::RequestBlock` (r:1 w:0)
    /// Proof: `Court::RequestBlock` (`max_values`: Some(1), `max_size`: Some(8), added: 503, mode: `MaxEncodedLen`)
    /// Storage: `PredictionMarkets::MarketIdsPerDisputeBlock` (r:2 w:2)
    /// Proof: `PredictionMarkets::MarketIdsPerDisputeBlock` (`max_values`: None, `max_size`: Some(1042), added: 3517, mode: `MaxEncodedLen`)
    /// Storage: `Balances::Reserves` (r:1 w:1)
    /// Proof: `Balances::Reserves` (`max_values`: None, `max_size`: Some(1249), added: 3724, mode: `MaxEncodedLen`)
    /// The range of component `j` is `[255, 1000]`.
    /// The range of component `a` is `[0, 2]`.
    /// The range of component `r` is `[0, 62]`.
    /// The range of component `f` is `[0, 62]`.
    fn appeal(j: u32, a: u32, _r: u32, _f: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0 + a * (24688 ±0) + f * (16 ±0) + j * (149 ±0) + r * (16 ±0)`
        //  Estimated: `268138 + a * (318078 ±1_049) + j * (194 ±3)`
        // Minimum execution time: 3_332_141 nanoseconds.
        Weight::from_parts(3_442_003_000, 268138)
            // Standard Error: 62_139
            .saturating_add(Weight::from_parts(5_788_506, 0).saturating_mul(j.into()))
            // Standard Error: 21_635_039
            .saturating_add(Weight::from_parts(3_898_041_505, 0).saturating_mul(a.into()))
            .saturating_add(T::DbWeight::get().reads(7))
            .saturating_add(T::DbWeight::get().reads((127_u64).saturating_mul(a.into())))
            .saturating_add(T::DbWeight::get().writes(100))
            .saturating_add(T::DbWeight::get().writes((117_u64).saturating_mul(a.into())))
            .saturating_add(Weight::from_parts(0, 318078).saturating_mul(a.into()))
            .saturating_add(Weight::from_parts(0, 194).saturating_mul(j.into()))
    }
    /// Storage: `Court::Courts` (r:1 w:1)
    /// Proof: `Court::Courts` (`max_values`: None, `max_size`: Some(349), added: 2824, mode: `MaxEncodedLen`)
    /// Storage: `Court::SelectedDraws` (r:1 w:1)
    /// Proof: `Court::SelectedDraws` (`max_values`: None, `max_size`: Some(149974), added: 152449, mode: `MaxEncodedLen`)
    /// Storage: `Court::Participants` (r:510 w:510)
    /// Proof: `Court::Participants` (`max_values`: None, `max_size`: Some(251), added: 2726, mode: `MaxEncodedLen`)
    /// Storage: `System::Account` (r:511 w:510)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(132), added: 2607, mode: `MaxEncodedLen`)
    /// The range of component `d` is `[5, 510]`.
    fn reassign_court_stakes(d: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `947 + d * (523 ±0)`
        //  Estimated: `153439 + d * (2726 ±0)`
        // Minimum execution time: 151_234 nanoseconds.
        Weight::from_parts(152_663_000, 153439)
            // Standard Error: 89_829
            .saturating_add(Weight::from_parts(79_360_224, 0).saturating_mul(d.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().reads((2_u64).saturating_mul(d.into())))
            .saturating_add(T::DbWeight::get().writes(2))
            .saturating_add(T::DbWeight::get().writes((2_u64).saturating_mul(d.into())))
            .saturating_add(Weight::from_parts(0, 2726).saturating_mul(d.into()))
    }
    /// Storage: `Court::YearlyInflation` (r:0 w:1)
    /// Proof: `Court::YearlyInflation` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
    fn set_inflation() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 10_490 nanoseconds.
        Weight::from_parts(10_980_000, 0).saturating_add(T::DbWeight::get().writes(1))
    }
    /// Storage: `Court::YearlyInflation` (r:1 w:0)
    /// Proof: `Court::YearlyInflation` (`max_values`: Some(1), `max_size`: Some(4), added: 499, mode: `MaxEncodedLen`)
    /// Storage: `Court::CourtPool` (r:1 w:0)
    /// Proof: `Court::CourtPool` (`max_values`: Some(1), `max_size`: Some(96002), added: 96497, mode: `MaxEncodedLen`)
    /// Storage: `System::Account` (r:999 w:999)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(132), added: 2607, mode: `MaxEncodedLen`)
    /// The range of component `j` is `[1, 1000]`.
    fn handle_inflation(j: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0 + j * (235 ±0)`
        //  Estimated: `97487 + j * (2607 ±0)`
        // Minimum execution time: 32_871 nanoseconds.
        Weight::from_parts(33_220_000, 97487)
            // Standard Error: 25_472
            .saturating_add(Weight::from_parts(21_011_552, 0).saturating_mul(j.into()))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(j.into())))
            .saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(j.into())))
            .saturating_add(Weight::from_parts(0, 2607).saturating_mul(j.into()))
    }
    /// Storage: `Court::CourtPool` (r:1 w:1)
    /// Proof: `Court::CourtPool` (`max_values`: Some(1), `max_size`: Some(96002), added: 96497, mode: `MaxEncodedLen`)
    /// Storage: `Court::SelectionNonce` (r:1 w:1)
    /// Proof: `Court::SelectionNonce` (`max_values`: Some(1), `max_size`: Some(8), added: 503, mode: `MaxEncodedLen`)
    /// Storage: `RandomnessCollectiveFlip::RandomMaterial` (r:1 w:0)
    /// Proof: `RandomnessCollectiveFlip::RandomMaterial` (`max_values`: Some(1), `max_size`: Some(2594), added: 3089, mode: `MaxEncodedLen`)
    /// Storage: `Court::Participants` (r:240 w:236)
    /// Proof: `Court::Participants` (`max_values`: None, `max_size`: Some(251), added: 2726, mode: `MaxEncodedLen`)
    /// The range of component `a` is `[0, 3]`.
    fn select_participants(a: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `113583 + a * (14765 ±0)`
        //  Estimated: `97487 + a * (136685 ±1_677)`
        // Minimum execution time: 1_493_831 nanoseconds.
        Weight::from_parts(1_180_557_253, 97487)
            // Standard Error: 33_970_293
            .saturating_add(Weight::from_parts(2_500_165_551, 0).saturating_mul(a.into()))
            .saturating_add(T::DbWeight::get().reads(31))
            .saturating_add(T::DbWeight::get().reads((50_u64).saturating_mul(a.into())))
            .saturating_add(T::DbWeight::get().writes(26))
            .saturating_add(T::DbWeight::get().writes((50_u64).saturating_mul(a.into())))
            .saturating_add(Weight::from_parts(0, 136685).saturating_mul(a.into()))
    }
    /// Storage: `Court::NextCourtId` (r:1 w:1)
    /// Proof: `Court::NextCourtId` (`max_values`: Some(1), `max_size`: Some(16), added: 511, mode: `MaxEncodedLen`)
    /// Storage: `Court::CourtPool` (r:1 w:1)
    /// Proof: `Court::CourtPool` (`max_values`: Some(1), `max_size`: Some(96002), added: 96497, mode: `MaxEncodedLen`)
    /// Storage: `Court::SelectionNonce` (r:1 w:1)
    /// Proof: `Court::SelectionNonce` (`max_values`: Some(1), `max_size`: Some(8), added: 503, mode: `MaxEncodedLen`)
    /// Storage: `RandomnessCollectiveFlip::RandomMaterial` (r:1 w:0)
    /// Proof: `RandomnessCollectiveFlip::RandomMaterial` (`max_values`: Some(1), `max_size`: Some(2594), added: 3089, mode: `MaxEncodedLen`)
    /// Storage: `Court::Participants` (r:31 w:31)
    /// Proof: `Court::Participants` (`max_values`: None, `max_size`: Some(251), added: 2726, mode: `MaxEncodedLen`)
    /// Storage: `Court::RequestBlock` (r:1 w:0)
    /// Proof: `Court::RequestBlock` (`max_values`: Some(1), `max_size`: Some(8), added: 503, mode: `MaxEncodedLen`)
    /// Storage: `PredictionMarkets::MarketIdsPerDisputeBlock` (r:1 w:1)
    /// Proof: `PredictionMarkets::MarketIdsPerDisputeBlock` (`max_values`: None, `max_size`: Some(1042), added: 3517, mode: `MaxEncodedLen`)
    /// Storage: `Court::SelectedDraws` (r:0 w:1)
    /// Proof: `Court::SelectedDraws` (`max_values`: None, `max_size`: Some(149974), added: 152449, mode: `MaxEncodedLen`)
    /// Storage: `Court::CourtIdToMarketId` (r:0 w:1)
    /// Proof: `Court::CourtIdToMarketId` (`max_values`: None, `max_size`: Some(40), added: 2515, mode: `MaxEncodedLen`)
    /// Storage: `Court::MarketIdToCourtId` (r:0 w:1)
    /// Proof: `Court::MarketIdToCourtId` (`max_values`: None, `max_size`: Some(40), added: 2515, mode: `MaxEncodedLen`)
    /// Storage: `Court::Courts` (r:0 w:1)
    /// Proof: `Court::Courts` (`max_values`: None, `max_size`: Some(349), added: 2824, mode: `MaxEncodedLen`)
    /// The range of component `j` is `[31, 1000]`.
    /// The range of component `r` is `[0, 62]`.
    fn on_dispute(j: u32, r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `5181 + j * (104 ±0) + r * (16 ±0)`
        //  Estimated: `97487 + j * (8 ±0) + r * (25 ±4)`
        // Minimum execution time: 285_517 nanoseconds.
        Weight::from_parts(323_671_356, 97487)
            // Standard Error: 2_211
            .saturating_add(Weight::from_parts(154_272, 0).saturating_mul(j.into()))
            // Standard Error: 34_279
            .saturating_add(Weight::from_parts(220_032, 0).saturating_mul(r.into()))
            .saturating_add(T::DbWeight::get().reads(34))
            .saturating_add(T::DbWeight::get().writes(36))
            .saturating_add(Weight::from_parts(0, 8).saturating_mul(j.into()))
            .saturating_add(Weight::from_parts(0, 25).saturating_mul(r.into()))
    }
    /// Storage: `Court::MarketIdToCourtId` (r:1 w:0)
    /// Proof: `Court::MarketIdToCourtId` (`max_values`: None, `max_size`: Some(40), added: 2515, mode: `MaxEncodedLen`)
    /// Storage: `Court::Courts` (r:1 w:1)
    /// Proof: `Court::Courts` (`max_values`: None, `max_size`: Some(349), added: 2824, mode: `MaxEncodedLen`)
    /// Storage: `Court::SelectedDraws` (r:1 w:0)
    /// Proof: `Court::SelectedDraws` (`max_values`: None, `max_size`: Some(149974), added: 152449, mode: `MaxEncodedLen`)
    /// Storage: `Court::CourtIdToMarketId` (r:1 w:0)
    /// Proof: `Court::CourtIdToMarketId` (`max_values`: None, `max_size`: Some(40), added: 2515, mode: `MaxEncodedLen`)
    /// Storage: `MarketCommons::Markets` (r:1 w:0)
    /// Proof: `MarketCommons::Markets` (`max_values`: None, `max_size`: Some(694), added: 3169, mode: `MaxEncodedLen`)
    /// Storage: `Court::Participants` (r:510 w:510)
    /// Proof: `Court::Participants` (`max_values`: None, `max_size`: Some(251), added: 2726, mode: `MaxEncodedLen`)
    /// The range of component `d` is `[1, 510]`.
    fn on_resolution(d: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `890 + d * (224 ±0)`
        //  Estimated: `153439 + d * (2726 ±0)`
        // Minimum execution time: 42_071 nanoseconds.
        Weight::from_parts(42_901_000, 153439)
            // Standard Error: 8_352
            .saturating_add(Weight::from_parts(6_168_005, 0).saturating_mul(d.into()))
            .saturating_add(T::DbWeight::get().reads(5))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(d.into())))
            .saturating_add(T::DbWeight::get().writes(1))
            .saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(d.into())))
            .saturating_add(Weight::from_parts(0, 2726).saturating_mul(d.into()))
    }
    /// Storage: `Court::MarketIdToCourtId` (r:1 w:0)
    /// Proof: `Court::MarketIdToCourtId` (`max_values`: None, `max_size`: Some(40), added: 2515, mode: `MaxEncodedLen`)
    /// Storage: `Court::Courts` (r:1 w:0)
    /// Proof: `Court::Courts` (`max_values`: None, `max_size`: Some(349), added: 2824, mode: `MaxEncodedLen`)
    /// Storage: `Balances::Reserves` (r:4 w:4)
    /// Proof: `Balances::Reserves` (`max_values`: None, `max_size`: Some(1249), added: 3724, mode: `MaxEncodedLen`)
    /// Storage: `System::Account` (r:4 w:4)
    /// Proof: `System::Account` (`max_values`: None, `max_size`: Some(132), added: 2607, mode: `MaxEncodedLen`)
    /// The range of component `a` is `[0, 4]`.
    fn exchange(a: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `388 + a * (320 ±0)`
        //  Estimated: `3814 + a * (3724 ±0)`
        // Minimum execution time: 13_140 nanoseconds.
        Weight::from_parts(15_216_631, 3814)
            // Standard Error: 53_048
            .saturating_add(Weight::from_parts(27_465_058, 0).saturating_mul(a.into()))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().reads((2_u64).saturating_mul(a.into())))
            .saturating_add(T::DbWeight::get().writes((2_u64).saturating_mul(a.into())))
            .saturating_add(Weight::from_parts(0, 3724).saturating_mul(a.into()))
    }
    /// Storage: `Court::MarketIdToCourtId` (r:1 w:0)
    /// Proof: `Court::MarketIdToCourtId` (`max_values`: None, `max_size`: Some(40), added: 2515, mode: `MaxEncodedLen`)
    /// Storage: `Court::Courts` (r:1 w:0)
    /// Proof: `Court::Courts` (`max_values`: None, `max_size`: Some(349), added: 2824, mode: `MaxEncodedLen`)
    fn get_auto_resolve() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `391`
        //  Estimated: `3814`
        // Minimum execution time: 11_440 nanoseconds.
        Weight::from_parts(11_800_000, 3814).saturating_add(T::DbWeight::get().reads(2))
    }
    /// Storage: `Court::MarketIdToCourtId` (r:1 w:0)
    /// Proof: `Court::MarketIdToCourtId` (`max_values`: None, `max_size`: Some(40), added: 2515, mode: `MaxEncodedLen`)
    /// Storage: `Court::CourtPool` (r:1 w:0)
    /// Proof: `Court::CourtPool` (`max_values`: Some(1), `max_size`: Some(96002), added: 96497, mode: `MaxEncodedLen`)
    /// Storage: `Court::Courts` (r:1 w:0)
    /// Proof: `Court::Courts` (`max_values`: None, `max_size`: Some(349), added: 2824, mode: `MaxEncodedLen`)
    /// Storage: `Court::CourtIdToMarketId` (r:1 w:0)
    /// Proof: `Court::CourtIdToMarketId` (`max_values`: None, `max_size`: Some(40), added: 2515, mode: `MaxEncodedLen`)
    /// Storage: `MarketCommons::Markets` (r:1 w:0)
    /// Proof: `MarketCommons::Markets` (`max_values`: None, `max_size`: Some(694), added: 3169, mode: `MaxEncodedLen`)
    fn has_failed() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `3854`
        //  Estimated: `97487`
        // Minimum execution time: 31_291 nanoseconds.
        Weight::from_parts(33_051_000, 97487).saturating_add(T::DbWeight::get().reads(5))
    }
    /// Storage: `Court::MarketIdToCourtId` (r:1 w:0)
    /// Proof: `Court::MarketIdToCourtId` (`max_values`: None, `max_size`: Some(40), added: 2515, mode: `MaxEncodedLen`)
    /// Storage: `Court::Courts` (r:1 w:1)
    /// Proof: `Court::Courts` (`max_values`: None, `max_size`: Some(349), added: 2824, mode: `MaxEncodedLen`)
    /// Storage: `Court::SelectedDraws` (r:1 w:1)
    /// Proof: `Court::SelectedDraws` (`max_values`: None, `max_size`: Some(149974), added: 152449, mode: `MaxEncodedLen`)
    /// Storage: `Court::Participants` (r:510 w:510)
    /// Proof: `Court::Participants` (`max_values`: None, `max_size`: Some(251), added: 2726, mode: `MaxEncodedLen`)
    /// The range of component `a` is `[0, 4]`.
    /// The range of component `d` is `[1, 510]`.
    fn on_global_dispute(_a: u32, d: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `422 + a * (66 ±0) + d * (224 ±0)`
        //  Estimated: `153439 + d * (2726 ±0)`
        // Minimum execution time: 28_151 nanoseconds.
        Weight::from_parts(28_631_000, 153439)
            // Standard Error: 5_176
            .saturating_add(Weight::from_parts(6_350_915, 0).saturating_mul(d.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(d.into())))
            .saturating_add(T::DbWeight::get().writes(2))
            .saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(d.into())))
            .saturating_add(Weight::from_parts(0, 2726).saturating_mul(d.into()))
    }
    /// Storage: `Court::MarketIdToCourtId` (r:1 w:0)
    /// Proof: `Court::MarketIdToCourtId` (`max_values`: None, `max_size`: Some(40), added: 2515, mode: `MaxEncodedLen`)
    /// Storage: `Court::SelectedDraws` (r:1 w:1)
    /// Proof: `Court::SelectedDraws` (`max_values`: None, `max_size`: Some(149974), added: 152449, mode: `MaxEncodedLen`)
    /// Storage: `Court::Participants` (r:510 w:510)
    /// Proof: `Court::Participants` (`max_values`: None, `max_size`: Some(251), added: 2726, mode: `MaxEncodedLen`)
    /// Storage: `Court::Courts` (r:0 w:1)
    /// Proof: `Court::Courts` (`max_values`: None, `max_size`: Some(349), added: 2824, mode: `MaxEncodedLen`)
    /// The range of component `d` is `[1, 510]`.
    fn clear(d: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `367 + d * (224 ±0)`
        //  Estimated: `153439 + d * (2726 ±0)`
        // Minimum execution time: 22_210 nanoseconds.
        Weight::from_parts(22_941_000, 153439)
            // Standard Error: 8_027
            .saturating_add(Weight::from_parts(6_074_804, 0).saturating_mul(d.into()))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(d.into())))
            .saturating_add(T::DbWeight::get().writes(2))
            .saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(d.into())))
            .saturating_add(Weight::from_parts(0, 2726).saturating_mul(d.into()))
    }
}
