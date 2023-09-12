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

//! Autogenerated weights for zrml_court
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
    /// Storage: Court CourtPool (r:1 w:1)
    /// Proof: Court CourtPool (max_values: Some(1), max_size: Some(72002), added: 72497, mode: MaxEncodedLen)
    /// Storage: Court Participants (r:1 w:1)
    /// Proof: Court Participants (max_values: None, max_size: Some(251), added: 2726, mode: MaxEncodedLen)
    /// Storage: Balances Locks (r:1 w:1)
    /// Proof: Balances Locks (max_values: None, max_size: Some(1299), added: 3774, mode: MaxEncodedLen)
    fn join_court(j: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1096 + j * (72 ±0)`
        //  Estimated: `78997`
        // Minimum execution time: 44_140 nanoseconds.
        Weight::from_parts(65_518_720, 78997)
            // Standard Error: 264
            .saturating_add(Weight::from_ref_time(150_281).saturating_mul(j.into()))
            .saturating_add(T::DbWeight::get().reads(3_u64))
            .saturating_add(T::DbWeight::get().writes(3_u64))
    }
    /// Storage: Court CourtPool (r:1 w:1)
    /// Proof: Court CourtPool (max_values: Some(1), max_size: Some(72002), added: 72497, mode: MaxEncodedLen)
    /// Storage: Court Participants (r:6 w:1)
    /// Proof: Court Participants (max_values: None, max_size: Some(251), added: 2726, mode: MaxEncodedLen)
    /// Storage: Balances Locks (r:1 w:1)
    /// Proof: Balances Locks (max_values: None, max_size: Some(1299), added: 3774, mode: MaxEncodedLen)
    fn delegate(j: u32, d: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0 + j * (74 ±0) + d * (685 ±0)`
        //  Estimated: `78997 + d * (2726 ±0)`
        // Minimum execution time: 73_850 nanoseconds.
        Weight::from_parts(50_792_351, 78997)
            // Standard Error: 251
            .saturating_add(Weight::from_ref_time(190_377).saturating_mul(j.into()))
            // Standard Error: 54_900
            .saturating_add(Weight::from_ref_time(11_551_233).saturating_mul(d.into()))
            .saturating_add(T::DbWeight::get().reads(3_u64))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(d.into())))
            .saturating_add(T::DbWeight::get().writes(3_u64))
            .saturating_add(Weight::from_proof_size(2726).saturating_mul(d.into()))
    }
    /// Storage: Court Participants (r:1 w:1)
    /// Proof: Court Participants (max_values: None, max_size: Some(251), added: 2726, mode: MaxEncodedLen)
    /// Storage: Court CourtPool (r:1 w:1)
    /// Proof: Court CourtPool (max_values: Some(1), max_size: Some(72002), added: 72497, mode: MaxEncodedLen)
    fn prepare_exit_court(j: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1034 + j * (72 ±0)`
        //  Estimated: `75223`
        // Minimum execution time: 28_760 nanoseconds.
        Weight::from_parts(46_309_265, 75223)
            // Standard Error: 257
            .saturating_add(Weight::from_ref_time(117_570).saturating_mul(j.into()))
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    /// Storage: Court Participants (r:1 w:1)
    /// Proof: Court Participants (max_values: None, max_size: Some(251), added: 2726, mode: MaxEncodedLen)
    /// Storage: Balances Locks (r:1 w:1)
    /// Proof: Balances Locks (max_values: None, max_size: Some(1299), added: 3774, mode: MaxEncodedLen)
    fn exit_court_remove() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `273`
        //  Estimated: `6500`
        // Minimum execution time: 38_990 nanoseconds.
        Weight::from_parts(48_090_000, 6500)
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    /// Storage: Court Participants (r:1 w:1)
    /// Proof: Court Participants (max_values: None, max_size: Some(251), added: 2726, mode: MaxEncodedLen)
    /// Storage: Balances Locks (r:1 w:1)
    /// Proof: Balances Locks (max_values: None, max_size: Some(1299), added: 3774, mode: MaxEncodedLen)
    fn exit_court_set() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `273`
        //  Estimated: `6500`
        // Minimum execution time: 37_120 nanoseconds.
        Weight::from_parts(45_810_000, 6500)
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(2_u64))
    }
    /// Storage: Court Courts (r:1 w:0)
    /// Proof: Court Courts (max_values: None, max_size: Some(349), added: 2824, mode: MaxEncodedLen)
    /// Storage: Court SelectedDraws (r:1 w:1)
    /// Proof: Court SelectedDraws (max_values: None, max_size: Some(149974), added: 152449, mode: MaxEncodedLen)
    fn vote(d: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `416 + d * (53 ±0)`
        //  Estimated: `155273`
        // Minimum execution time: 57_331 nanoseconds.
        Weight::from_parts(70_805_856, 155273)
            // Standard Error: 135
            .saturating_add(Weight::from_ref_time(130_051).saturating_mul(d.into()))
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
    /// Storage: Court CourtIdToMarketId (r:1 w:0)
    /// Proof: Court CourtIdToMarketId (max_values: None, max_size: Some(40), added: 2515, mode: MaxEncodedLen)
    /// Storage: MarketCommons Markets (r:1 w:0)
    /// Proof: MarketCommons Markets (max_values: None, max_size: Some(541), added: 3016, mode: MaxEncodedLen)
    /// Storage: Court Participants (r:1 w:0)
    /// Proof: Court Participants (max_values: None, max_size: Some(251), added: 2726, mode: MaxEncodedLen)
    /// Storage: Court Courts (r:1 w:0)
    /// Proof: Court Courts (max_values: None, max_size: Some(349), added: 2824, mode: MaxEncodedLen)
    /// Storage: Court SelectedDraws (r:1 w:1)
    /// Proof: Court SelectedDraws (max_values: None, max_size: Some(149974), added: 152449, mode: MaxEncodedLen)
    fn denounce_vote(d: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `1525 + d * (53 ±0)`
        //  Estimated: `163530`
        // Minimum execution time: 58_061 nanoseconds.
        Weight::from_parts(80_304_402, 163530)
            // Standard Error: 400
            .saturating_add(Weight::from_ref_time(195_807).saturating_mul(d.into()))
            .saturating_add(T::DbWeight::get().reads(5_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
    /// Storage: Court CourtIdToMarketId (r:1 w:0)
    /// Proof: Court CourtIdToMarketId (max_values: None, max_size: Some(40), added: 2515, mode: MaxEncodedLen)
    /// Storage: MarketCommons Markets (r:1 w:0)
    /// Proof: MarketCommons Markets (max_values: None, max_size: Some(541), added: 3016, mode: MaxEncodedLen)
    /// Storage: Court Participants (r:1 w:0)
    /// Proof: Court Participants (max_values: None, max_size: Some(251), added: 2726, mode: MaxEncodedLen)
    /// Storage: Court Courts (r:1 w:0)
    /// Proof: Court Courts (max_values: None, max_size: Some(349), added: 2824, mode: MaxEncodedLen)
    /// Storage: Court SelectedDraws (r:1 w:1)
    /// Proof: Court SelectedDraws (max_values: None, max_size: Some(149974), added: 152449, mode: MaxEncodedLen)
    fn reveal_vote(d: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `2107 + d * (53 ±0)`
        //  Estimated: `163530`
        // Minimum execution time: 95_970 nanoseconds.
        Weight::from_parts(118_381_168, 163530)
            // Standard Error: 188
            .saturating_add(Weight::from_ref_time(127_581).saturating_mul(d.into()))
            .saturating_add(T::DbWeight::get().reads(5_u64))
            .saturating_add(T::DbWeight::get().writes(1_u64))
    }
    /// Storage: Court Courts (r:1 w:1)
    /// Proof: Court Courts (max_values: None, max_size: Some(349), added: 2824, mode: MaxEncodedLen)
    /// Storage: Court CourtIdToMarketId (r:1 w:0)
    /// Proof: Court CourtIdToMarketId (max_values: None, max_size: Some(40), added: 2515, mode: MaxEncodedLen)
    /// Storage: MarketCommons Markets (r:1 w:0)
    /// Proof: MarketCommons Markets (max_values: None, max_size: Some(541), added: 3016, mode: MaxEncodedLen)
    /// Storage: Court SelectedDraws (r:1 w:1)
    /// Proof: Court SelectedDraws (max_values: None, max_size: Some(149974), added: 152449, mode: MaxEncodedLen)
    /// Storage: Court CourtPool (r:1 w:1)
    /// Proof: Court CourtPool (max_values: Some(1), max_size: Some(72002), added: 72497, mode: MaxEncodedLen)
    /// Storage: Court SelectionNonce (r:1 w:1)
    /// Proof: Court SelectionNonce (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
    /// Storage: RandomnessCollectiveFlip RandomMaterial (r:1 w:0)
    /// Proof: RandomnessCollectiveFlip RandomMaterial (max_values: Some(1), max_size: Some(2594), added: 3089, mode: MaxEncodedLen)
    /// Storage: Court Participants (r:347 w:343)
    /// Proof: Court Participants (max_values: None, max_size: Some(251), added: 2726, mode: MaxEncodedLen)
    /// Storage: Court RequestBlock (r:1 w:0)
    /// Proof: Court RequestBlock (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
    /// Storage: PredictionMarkets MarketIdsPerDisputeBlock (r:2 w:2)
    /// Proof: PredictionMarkets MarketIdsPerDisputeBlock (max_values: None, max_size: Some(1042), added: 3517, mode: MaxEncodedLen)
    /// Storage: Balances Reserves (r:1 w:1)
    /// Proof: Balances Reserves (max_values: None, max_size: Some(1249), added: 3724, mode: MaxEncodedLen)
    fn appeal(j: u32, a: u32, _r: u32, _f: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0 + j * (132 ±0) + a * (35486 ±0) + r * (16 ±0) + f * (16 ±0)`
        //  Estimated: `515302 + j * (203 ±1) + a * (314898 ±368)`
        // Minimum execution time: 3_652_190 nanoseconds.
        Weight::from_parts(4_409_134_000, 515302)
            // Standard Error: 26_318
            .saturating_add(Weight::from_ref_time(6_785_123).saturating_mul(j.into()))
            // Standard Error: 9_392_573
            .saturating_add(Weight::from_ref_time(4_808_492_598).saturating_mul(a.into()))
            .saturating_add(T::DbWeight::get().reads(109_u64))
            .saturating_add(T::DbWeight::get().reads((116_u64).saturating_mul(a.into())))
            .saturating_add(T::DbWeight::get().writes(100_u64))
            .saturating_add(Weight::from_proof_size(203).saturating_mul(j.into()))
            .saturating_add(Weight::from_proof_size(314898).saturating_mul(a.into()))
    }
    /// Storage: Court Courts (r:1 w:1)
    /// Proof: Court Courts (max_values: None, max_size: Some(349), added: 2824, mode: MaxEncodedLen)
    /// Storage: Court SelectedDraws (r:1 w:1)
    /// Proof: Court SelectedDraws (max_values: None, max_size: Some(149974), added: 152449, mode: MaxEncodedLen)
    /// Storage: Court Participants (r:510 w:510)
    /// Proof: Court Participants (max_values: None, max_size: Some(251), added: 2726, mode: MaxEncodedLen)
    /// Storage: System Account (r:511 w:510)
    /// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
    fn reassign_court_stakes(d: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `911 + d * (587 ±0)`
        //  Estimated: `157880 + d * (5333 ±0)`
        // Minimum execution time: 153_811 nanoseconds.
        Weight::from_parts(188_241_000, 157880)
            // Standard Error: 40_407
            .saturating_add(Weight::from_ref_time(76_164_607).saturating_mul(d.into()))
            .saturating_add(T::DbWeight::get().reads(3_u64))
            .saturating_add(T::DbWeight::get().reads((2_u64).saturating_mul(d.into())))
            .saturating_add(T::DbWeight::get().writes(2_u64))
            .saturating_add(Weight::from_proof_size(5333).saturating_mul(d.into()))
    }
    /// Storage: Court YearlyInflation (r:0 w:1)
    /// Proof: Court YearlyInflation (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
    fn set_inflation() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0`
        //  Estimated: `0`
        // Minimum execution time: 13_390 nanoseconds.
        Weight::from_ref_time(16_751_000).saturating_add(T::DbWeight::get().writes(1_u64))
    }
    /// Storage: Court YearlyInflation (r:1 w:0)
    /// Proof: Court YearlyInflation (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
    /// Storage: Court CourtPool (r:1 w:0)
    /// Proof: Court CourtPool (max_values: Some(1), max_size: Some(72002), added: 72497, mode: MaxEncodedLen)
    /// Storage: System Account (r:1000 w:1000)
    /// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
    fn handle_inflation(j: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `0 + j * (243 ±0)`
        //  Estimated: `72996 + j * (2607 ±0)`
        // Minimum execution time: 34_750 nanoseconds.
        Weight::from_parts(42_340_000, 72996)
            // Standard Error: 7_714
            .saturating_add(Weight::from_ref_time(22_591_960).saturating_mul(j.into()))
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(j.into())))
            .saturating_add(Weight::from_proof_size(2607).saturating_mul(j.into()))
    }
    /// Storage: Court CourtPool (r:1 w:1)
    /// Proof: Court CourtPool (max_values: Some(1), max_size: Some(72002), added: 72497, mode: MaxEncodedLen)
    /// Storage: Court SelectionNonce (r:1 w:1)
    /// Proof: Court SelectionNonce (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
    /// Storage: RandomnessCollectiveFlip RandomMaterial (r:1 w:0)
    /// Proof: RandomnessCollectiveFlip RandomMaterial (max_values: Some(1), max_size: Some(2594), added: 3089, mode: MaxEncodedLen)
    /// Storage: Court Participants (r:240 w:236)
    /// Proof: Court Participants (max_values: None, max_size: Some(251), added: 2726, mode: MaxEncodedLen)
    fn select_participants(a: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `84018 + a * (19595 ±0)`
        //  Estimated: `133335 + a * (162878 ±713)`
        // Minimum execution time: 1_583_905 nanoseconds.
        Weight::from_parts(979_468_625, 133335)
            // Standard Error: 19_324_539
            .saturating_add(Weight::from_ref_time(3_719_940_913).saturating_mul(a.into()))
            .saturating_add(T::DbWeight::get().reads(24_u64))
            .saturating_add(T::DbWeight::get().reads((60_u64).saturating_mul(a.into())))
            .saturating_add(T::DbWeight::get().writes(19_u64))
            .saturating_add(Weight::from_proof_size(162878).saturating_mul(a.into()))
    }
    /// Storage: Court NextCourtId (r:1 w:1)
    /// Proof: Court NextCourtId (max_values: Some(1), max_size: Some(16), added: 511, mode: MaxEncodedLen)
    /// Storage: Court CourtPool (r:1 w:1)
    /// Proof: Court CourtPool (max_values: Some(1), max_size: Some(72002), added: 72497, mode: MaxEncodedLen)
    /// Storage: Court SelectionNonce (r:1 w:1)
    /// Proof: Court SelectionNonce (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
    /// Storage: RandomnessCollectiveFlip RandomMaterial (r:1 w:0)
    /// Proof: RandomnessCollectiveFlip RandomMaterial (max_values: Some(1), max_size: Some(2594), added: 3089, mode: MaxEncodedLen)
    /// Storage: Court Participants (r:31 w:31)
    /// Proof: Court Participants (max_values: None, max_size: Some(251), added: 2726, mode: MaxEncodedLen)
    /// Storage: Court RequestBlock (r:1 w:0)
    /// Proof: Court RequestBlock (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
    /// Storage: PredictionMarkets MarketIdsPerDisputeBlock (r:1 w:1)
    /// Proof: PredictionMarkets MarketIdsPerDisputeBlock (max_values: None, max_size: Some(1042), added: 3517, mode: MaxEncodedLen)
    /// Storage: Court SelectedDraws (r:0 w:1)
    /// Proof: Court SelectedDraws (max_values: None, max_size: Some(149974), added: 152449, mode: MaxEncodedLen)
    /// Storage: Court CourtIdToMarketId (r:0 w:1)
    /// Proof: Court CourtIdToMarketId (max_values: None, max_size: Some(40), added: 2515, mode: MaxEncodedLen)
    /// Storage: Court MarketIdToCourtId (r:0 w:1)
    /// Proof: Court MarketIdToCourtId (max_values: None, max_size: Some(40), added: 2515, mode: MaxEncodedLen)
    /// Storage: Court Courts (r:0 w:1)
    /// Proof: Court Courts (max_values: None, max_size: Some(349), added: 2824, mode: MaxEncodedLen)
    fn on_dispute(j: u32, r: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `6039 + j * (80 ±0) + r * (16 ±0)`
        //  Estimated: `153295 + j * (11 ±0) + r * (29 ±1)`
        // Minimum execution time: 299_840 nanoseconds.
        Weight::from_parts(376_923_517, 153295)
            // Standard Error: 867
            .saturating_add(Weight::from_ref_time(261_807).saturating_mul(j.into()))
            // Standard Error: 13_422
            .saturating_add(Weight::from_ref_time(289_721).saturating_mul(r.into()))
            .saturating_add(T::DbWeight::get().reads(33_u64))
            .saturating_add(T::DbWeight::get().writes(35_u64))
            .saturating_add(Weight::from_proof_size(11).saturating_mul(j.into()))
            .saturating_add(Weight::from_proof_size(29).saturating_mul(r.into()))
    }
    /// Storage: Court MarketIdToCourtId (r:1 w:0)
    /// Proof: Court MarketIdToCourtId (max_values: None, max_size: Some(40), added: 2515, mode: MaxEncodedLen)
    /// Storage: Court Courts (r:1 w:1)
    /// Proof: Court Courts (max_values: None, max_size: Some(349), added: 2824, mode: MaxEncodedLen)
    /// Storage: Court SelectedDraws (r:1 w:0)
    /// Proof: Court SelectedDraws (max_values: None, max_size: Some(149974), added: 152449, mode: MaxEncodedLen)
    /// Storage: Court CourtIdToMarketId (r:1 w:0)
    /// Proof: Court CourtIdToMarketId (max_values: None, max_size: Some(40), added: 2515, mode: MaxEncodedLen)
    /// Storage: MarketCommons Markets (r:1 w:0)
    /// Proof: MarketCommons Markets (max_values: None, max_size: Some(541), added: 3016, mode: MaxEncodedLen)
    /// Storage: Court Participants (r:510 w:510)
    /// Proof: Court Participants (max_values: None, max_size: Some(251), added: 2726, mode: MaxEncodedLen)
    fn on_resolution(d: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `930 + d * (256 ±0)`
        //  Estimated: `163319 + d * (2726 ±0)`
        // Minimum execution time: 45_441 nanoseconds.
        Weight::from_parts(47_380_000, 163319)
            // Standard Error: 5_948
            .saturating_add(Weight::from_ref_time(7_493_292).saturating_mul(d.into()))
            .saturating_add(T::DbWeight::get().reads(5_u64))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(d.into())))
            .saturating_add(T::DbWeight::get().writes(1_u64))
            .saturating_add(Weight::from_proof_size(2726).saturating_mul(d.into()))
    }
    /// Storage: Court MarketIdToCourtId (r:1 w:0)
    /// Proof: Court MarketIdToCourtId (max_values: None, max_size: Some(40), added: 2515, mode: MaxEncodedLen)
    /// Storage: Court Courts (r:1 w:0)
    /// Proof: Court Courts (max_values: None, max_size: Some(349), added: 2824, mode: MaxEncodedLen)
    /// Storage: Balances Reserves (r:4 w:4)
    /// Proof: Balances Reserves (max_values: None, max_size: Some(1249), added: 3724, mode: MaxEncodedLen)
    /// Storage: System Account (r:4 w:4)
    /// Proof: System Account (max_values: None, max_size: Some(132), added: 2607, mode: MaxEncodedLen)
    fn exchange(a: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `386 + a * (352 ±0)`
        //  Estimated: `5339 + a * (6331 ±0)`
        // Minimum execution time: 16_360 nanoseconds.
        Weight::from_parts(21_054_877, 5339)
            // Standard Error: 27_645
            .saturating_add(Weight::from_ref_time(34_674_206).saturating_mul(a.into()))
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().reads((2_u64).saturating_mul(a.into())))
            .saturating_add(Weight::from_proof_size(6331).saturating_mul(a.into()))
    }
    /// Storage: Court MarketIdToCourtId (r:1 w:0)
    /// Proof: Court MarketIdToCourtId (max_values: None, max_size: Some(40), added: 2515, mode: MaxEncodedLen)
    /// Storage: Court Courts (r:1 w:0)
    /// Proof: Court Courts (max_values: None, max_size: Some(349), added: 2824, mode: MaxEncodedLen)
    fn get_auto_resolve() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `389`
        //  Estimated: `5339`
        // Minimum execution time: 13_860 nanoseconds.
        Weight::from_parts(17_320_000, 5339).saturating_add(T::DbWeight::get().reads(2_u64))
    }
    /// Storage: Court MarketIdToCourtId (r:1 w:0)
    /// Proof: Court MarketIdToCourtId (max_values: None, max_size: Some(40), added: 2515, mode: MaxEncodedLen)
    /// Storage: Court CourtPool (r:1 w:0)
    /// Proof: Court CourtPool (max_values: Some(1), max_size: Some(72002), added: 72497, mode: MaxEncodedLen)
    /// Storage: Court Courts (r:1 w:0)
    /// Proof: Court Courts (max_values: None, max_size: Some(349), added: 2824, mode: MaxEncodedLen)
    /// Storage: Court CourtIdToMarketId (r:1 w:0)
    /// Proof: Court CourtIdToMarketId (max_values: None, max_size: Some(40), added: 2515, mode: MaxEncodedLen)
    /// Storage: MarketCommons Markets (r:1 w:0)
    /// Proof: MarketCommons Markets (max_values: None, max_size: Some(541), added: 3016, mode: MaxEncodedLen)
    fn has_failed() -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `3151`
        //  Estimated: `83367`
        // Minimum execution time: 34_920 nanoseconds.
        Weight::from_parts(43_370_000, 83367).saturating_add(T::DbWeight::get().reads(5_u64))
    }
    /// Storage: Court MarketIdToCourtId (r:1 w:0)
    /// Proof: Court MarketIdToCourtId (max_values: None, max_size: Some(40), added: 2515, mode: MaxEncodedLen)
    /// Storage: Court Courts (r:1 w:1)
    /// Proof: Court Courts (max_values: None, max_size: Some(349), added: 2824, mode: MaxEncodedLen)
    /// Storage: Court SelectedDraws (r:1 w:1)
    /// Proof: Court SelectedDraws (max_values: None, max_size: Some(149974), added: 152449, mode: MaxEncodedLen)
    /// Storage: Court Participants (r:510 w:510)
    /// Proof: Court Participants (max_values: None, max_size: Some(251), added: 2726, mode: MaxEncodedLen)
    fn on_global_dispute(_a: u32, d: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `448 + a * (66 ±0) + d * (256 ±0)`
        //  Estimated: `157788 + d * (2726 ±0)`
        // Minimum execution time: 31_350 nanoseconds.
        Weight::from_parts(34_136_188, 157788)
            // Standard Error: 7_860
            .saturating_add(Weight::from_ref_time(7_658_064).saturating_mul(d.into()))
            .saturating_add(T::DbWeight::get().reads(3_u64))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(d.into())))
            .saturating_add(T::DbWeight::get().writes(2_u64))
            .saturating_add(Weight::from_proof_size(2726).saturating_mul(d.into()))
    }
    /// Storage: Court MarketIdToCourtId (r:1 w:0)
    /// Proof: Court MarketIdToCourtId (max_values: None, max_size: Some(40), added: 2515, mode: MaxEncodedLen)
    /// Storage: Court SelectedDraws (r:1 w:1)
    /// Proof: Court SelectedDraws (max_values: None, max_size: Some(149974), added: 152449, mode: MaxEncodedLen)
    /// Storage: Court Participants (r:510 w:510)
    /// Proof: Court Participants (max_values: None, max_size: Some(251), added: 2726, mode: MaxEncodedLen)
    /// Storage: Court Courts (r:0 w:1)
    /// Proof: Court Courts (max_values: None, max_size: Some(349), added: 2824, mode: MaxEncodedLen)
    fn clear(d: u32) -> Weight {
        // Proof Size summary in bytes:
        //  Measured:  `363 + d * (256 ±0)`
        //  Estimated: `154964 + d * (2726 ±0)`
        // Minimum execution time: 25_481 nanoseconds.
        Weight::from_parts(29_170_000, 154964)
            // Standard Error: 5_395
            .saturating_add(Weight::from_ref_time(7_136_451).saturating_mul(d.into()))
            .saturating_add(T::DbWeight::get().reads(2_u64))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(d.into())))
            .saturating_add(T::DbWeight::get().writes(2_u64))
            .saturating_add(Weight::from_proof_size(2726).saturating_mul(d.into()))
    }
}
