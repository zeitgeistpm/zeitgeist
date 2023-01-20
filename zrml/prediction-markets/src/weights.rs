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

//! Autogenerated weights for zrml_prediction_markets
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-01-09, STEPS: `10`, REPEAT: 1000, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/zeitgeist
// benchmark
// pallet
// --chain=dev
// --steps=10
// --repeat=1000
// --pallet=zrml_prediction_markets
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --output=./zrml/prediction-markets/src/weights.rs
// --template=./misc/weight_template.hbs

#![allow(unused_parens)]
#![allow(unused_imports)]

use core::marker::PhantomData;
use frame_support::{traits::Get, weights::Weight};

///  Trait containing the required functions for weight retrival within
/// zrml_prediction_markets (automatically generated)
pub trait WeightInfoZeitgeist {
    fn admin_destroy_disputed_market(a: u32, o: u32, c: u32, r: u32) -> Weight;
    fn admin_destroy_reported_market(a: u32, o: u32, c: u32, r: u32) -> Weight;
    fn admin_move_market_to_closed(o: u32, c: u32) -> Weight;
    fn admin_move_market_to_resolved_scalar_reported(r: u32) -> Weight;
    fn admin_move_market_to_resolved_categorical_reported(r: u32) -> Weight;
    fn admin_move_market_to_resolved_scalar_disputed(r: u32) -> Weight;
    fn admin_move_market_to_resolved_categorical_disputed(r: u32) -> Weight;
    fn approve_market() -> Weight;
    fn request_edit(r: u32) -> Weight;
    fn buy_complete_set(a: u32) -> Weight;
    fn create_market(m: u32) -> Weight;
    fn edit_market(m: u32) -> Weight;
    fn deploy_swap_pool_for_market_future_pool(a: u32, o: u32) -> Weight;
    fn deploy_swap_pool_for_market_open_pool(a: u32) -> Weight;
    fn start_global_dispute(m: u32, n: u32) -> Weight;
    fn dispute_authorized() -> Weight;
    fn handle_expired_advised_market() -> Weight;
    fn internal_resolve_categorical_reported() -> Weight;
    fn internal_resolve_categorical_disputed() -> Weight;
    fn internal_resolve_scalar_reported() -> Weight;
    fn internal_resolve_scalar_disputed() -> Weight;
    fn on_initialize_resolve_overhead() -> Weight;
    fn process_subsidy_collecting_markets_raw(a: u32) -> Weight;
    fn redeem_shares_categorical() -> Weight;
    fn redeem_shares_scalar() -> Weight;
    fn reject_market(c: u32, o: u32, r: u32) -> Weight;
    fn report(m: u32) -> Weight;
    fn sell_complete_set(a: u32) -> Weight;
    fn start_subsidy(a: u32) -> Weight;
    fn market_status_manager(b: u32, f: u32) -> Weight;
    fn market_resolution_manager(r: u32, d: u32) -> Weight;
    fn process_subsidy_collecting_markets_dummy() -> Weight;
}

/// Weight functions for zrml_prediction_markets (automatically generated)
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfoZeitgeist for WeightInfo<T> {
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: Balances Reserves (r:2 w:2)
    // Storage: System Account (r:3 w:3)
    // Storage: MarketCommons MarketPool (r:1 w:1)
    // Storage: Swaps Pools (r:1 w:1)
    // Storage: Tokens Accounts (r:2 w:2)
    // Storage: Tokens TotalIssuance (r:2 w:2)
    // Storage: Authorized AuthorizedOutcomeReports (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerDisputeBlock (r:1 w:1)
    fn admin_destroy_disputed_market(a: u32, o: u32, c: u32, r: u32) -> Weight {
        Weight::from_ref_time(138_848_000)
            // Standard Error: 35_000
            .saturating_add(Weight::from_ref_time(20_922_000))
            .saturating_mul(a.into())
            // Standard Error: 34_000
            .saturating_add(Weight::from_ref_time(1_091_000).saturating_mul(o.into()))
            // Standard Error: 34_000
            .saturating_add(Weight::from_ref_time(984_000).saturating_mul(c.into()))
            // Standard Error: 34_000
            .saturating_add(Weight::from_ref_time(1_026_000).saturating_mul(r.into()))
            .saturating_add(T::DbWeight::get().reads(10))
            .saturating_add(T::DbWeight::get().reads((2_u64).saturating_mul(a.into())))
            .saturating_add(T::DbWeight::get().writes(10))
            .saturating_add(T::DbWeight::get().writes((2_u64).saturating_mul(a.into())))
    }
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: Balances Reserves (r:1 w:1)
    // Storage: System Account (r:2 w:2)
    // Storage: MarketCommons MarketPool (r:1 w:1)
    // Storage: Swaps Pools (r:1 w:1)
    // Storage: Tokens Accounts (r:2 w:2)
    // Storage: Tokens TotalIssuance (r:2 w:2)
    // Storage: PredictionMarkets MarketIdsPerReportBlock (r:1 w:1)
    // Storage: PredictionMarkets Disputes (r:0 w:1)
    fn admin_destroy_reported_market(a: u32, o: u32, c: u32, r: u32) -> Weight {
        Weight::from_ref_time(87_265_000)
            // Standard Error: 1_000
            .saturating_add(Weight::from_ref_time(10_869_000).saturating_mul(a.into()))
            // Standard Error: 1_000
            .saturating_add(Weight::from_ref_time(12_000).saturating_mul(o.into()))
            // Standard Error: 1_000
            .saturating_add(Weight::from_ref_time(21_000).saturating_mul(c.into()))
            // Standard Error: 1_000
            .saturating_add(Weight::from_ref_time(17_000).saturating_mul(r.into()))
            .saturating_add(T::DbWeight::get().reads(7))
            .saturating_add(T::DbWeight::get().reads((2_u64).saturating_mul(a.into())))
            .saturating_add(T::DbWeight::get().writes(8))
            .saturating_add(T::DbWeight::get().writes((2_u64).saturating_mul(a.into())))
    }
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerOpenTimeFrame (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerCloseTimeFrame (r:1 w:1)
    // Storage: MarketCommons MarketPool (r:1 w:0)
    fn admin_move_market_to_closed(o: u32, c: u32) -> Weight {
        Weight::from_ref_time(23_683_000)
            // Standard Error: 0
            .saturating_add(Weight::from_ref_time(14_000).saturating_mul(o.into()))
            // Standard Error: 0
            .saturating_add(Weight::from_ref_time(20_000).saturating_mul(c.into()))
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerReportBlock (r:1 w:1)
    // Storage: Balances Reserves (r:1 w:1)
    // Storage: PredictionMarkets Disputes (r:1 w:1)
    // Storage: MarketCommons MarketPool (r:1 w:0)
    fn admin_move_market_to_resolved_scalar_reported(r: u32) -> Weight {
        Weight::from_ref_time(40_650_000)
            // Standard Error: 0
            .saturating_add(Weight::from_ref_time(15_000).saturating_mul(r.into()))
            .saturating_add(T::DbWeight::get().reads(5))
            .saturating_add(T::DbWeight::get().writes(4))
    }
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerReportBlock (r:1 w:1)
    // Storage: Balances Reserves (r:1 w:1)
    // Storage: PredictionMarkets Disputes (r:1 w:1)
    // Storage: MarketCommons MarketPool (r:1 w:0)
    // Storage: Swaps Pools (r:1 w:1)
    fn admin_move_market_to_resolved_categorical_reported(r: u32) -> Weight {
        Weight::from_ref_time(76_526_000)
            // Standard Error: 0
            .saturating_add(Weight::from_ref_time(28_000).saturating_mul(r.into()))
            .saturating_add(T::DbWeight::get().reads(6))
            .saturating_add(T::DbWeight::get().writes(5))
    }
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: PredictionMarkets Disputes (r:1 w:1)
    // Storage: Authorized AuthorizedOutcomeReports (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerDisputeBlock (r:1 w:1)
    // Storage: Balances Reserves (r:2 w:2)
    // Storage: GlobalDisputes Winners (r:1 w:0)
    // Storage: System Account (r:1 w:1)
    // Storage: MarketCommons MarketPool (r:1 w:0)
    fn admin_move_market_to_resolved_scalar_disputed(r: u32) -> Weight {
        Weight::from_ref_time(56_260_000)
            // Standard Error: 0
            .saturating_add(Weight::from_ref_time(22_000).saturating_mul(r.into()))
            // Standard Error: 6_000
            .saturating_add(T::DbWeight::get().reads(8))
            .saturating_add(T::DbWeight::get().writes(5))
    }
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: PredictionMarkets Disputes (r:1 w:1)
    // Storage: Authorized AuthorizedOutcomeReports (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerDisputeBlock (r:1 w:1)
    // Storage: Balances Reserves (r:2 w:2)
    // Storage: GlobalDisputes Winners (r:1 w:0)
    // Storage: System Account (r:1 w:1)
    // Storage: MarketCommons MarketPool (r:1 w:0)
    // Storage: Swaps Pools (r:1 w:1)
    fn admin_move_market_to_resolved_categorical_disputed(r: u32) -> Weight {
        Weight::from_ref_time(93_329_000)
            // Standard Error: 0
            .saturating_add(Weight::from_ref_time(7_000).saturating_mul(r.into()))
            // Standard Error: 8_000
            .saturating_add(T::DbWeight::get().reads(9))
            .saturating_add(T::DbWeight::get().writes(6))
    }
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsForEdit (r:1 w:0)
    // Storage: Balances Reserves (r:1 w:1)
    fn approve_market() -> Weight {
        Weight::from_ref_time(26_968_000)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    // Storage: MarketCommons Markets (r:1 w:0)
    // Storage: PredictionMarkets MarketIdsForEdit (r:1 w:1)
    fn request_edit(_r: u32) -> Weight {
        Weight::from_ref_time(15_701_000)
            // Standard Error: 0
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    // Storage: MarketCommons Markets (r:1 w:0)
    // Storage: System Account (r:1 w:1)
    // Storage: Tokens Accounts (r:2 w:2)
    // Storage: Tokens TotalIssuance (r:2 w:2)
    fn buy_complete_set(a: u32) -> Weight {
        Weight::from_ref_time(34_178_000)
            // Standard Error: 1_000
            .saturating_add(Weight::from_ref_time(8_264_000).saturating_mul(a.into()))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().reads((2_u64).saturating_mul(a.into())))
            .saturating_add(T::DbWeight::get().writes(1))
            .saturating_add(T::DbWeight::get().writes((2_u64).saturating_mul(a.into())))
    }
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: Balances Reserves (r:1 w:1)
    // Storage: MarketCommons MarketCounter (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerCloseTimeFrame (r:1 w:1)
    // Storage: MarketCommons Markets (r:0 w:1)
    fn create_market(m: u32) -> Weight {
        Weight::from_ref_time(30_604_000)
            // Standard Error: 0
            .saturating_add(Weight::from_ref_time(17_000).saturating_mul(m.into()))
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(4))
    }
    // Storage: PredictionMarkets MarketIdsForEdit (r:1 w:1)
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerCloseTimeFrame (r:1 w:1)
    // Storage: Timestamp Now (r:1 w:0)
    fn edit_market(m: u32) -> Weight {
        Weight::from_ref_time(26_200_000)
            // Standard Error: 0
            .saturating_add(Weight::from_ref_time(27_000).saturating_mul(m.into()))
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    // Storage: MarketCommons Markets (r:1 w:0)
    // Storage: Swaps NextPoolId (r:1 w:1)
    // Storage: Tokens Accounts (r:5 w:5)
    // Storage: System Account (r:1 w:1)
    // Storage: Tokens TotalIssuance (r:1 w:1)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: PredictionMarkets MarketIdsPerOpenTimeFrame (r:1 w:1)
    // Storage: MarketCommons MarketPool (r:1 w:1)
    // Storage: Swaps Pools (r:0 w:1)
    fn deploy_swap_pool_for_market_future_pool(a: u32, o: u32) -> Weight {
        Weight::from_ref_time(61_235_000)
            // Standard Error: 1_000
            .saturating_add(Weight::from_ref_time(13_099_000).saturating_mul(a.into()))
            // Standard Error: 1_000
            .saturating_add(Weight::from_ref_time(36_000).saturating_mul(o.into()))
            .saturating_add(T::DbWeight::get().reads(8))
            .saturating_add(T::DbWeight::get().reads((2_u64).saturating_mul(a.into())))
            .saturating_add(T::DbWeight::get().writes(7))
            .saturating_add(T::DbWeight::get().writes((2_u64).saturating_mul(a.into())))
    }
    // Storage: MarketCommons Markets (r:1 w:0)
    // Storage: Swaps NextPoolId (r:1 w:1)
    // Storage: Tokens Accounts (r:5 w:5)
    // Storage: System Account (r:1 w:1)
    // Storage: Tokens TotalIssuance (r:1 w:1)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: MarketCommons MarketPool (r:1 w:1)
    // Storage: Swaps Pools (r:0 w:1)
    fn deploy_swap_pool_for_market_open_pool(a: u32) -> Weight {
        Weight::from_ref_time(60_792_000)
            // Standard Error: 3_000
            .saturating_add(Weight::from_ref_time(13_322_000).saturating_mul(a.into()))
            .saturating_add(T::DbWeight::get().reads(7))
            .saturating_add(T::DbWeight::get().reads((2_u64).saturating_mul(a.into())))
            .saturating_add(T::DbWeight::get().writes(6))
            .saturating_add(T::DbWeight::get().writes((2_u64).saturating_mul(a.into())))
    }
    // Storage: MarketCommons Markets (r:1 w:0)
    // Storage: PredictionMarkets Disputes (r:1 w:0)
    // Storage: GlobalDisputes Winners (r:1 w:1)
    // Storage: GlobalDisputes Outcomes (r:7 w:7)
    // Storage: PredictionMarkets MarketIdsPerDisputeBlock (r:2 w:2)
    fn start_global_dispute(_m: u32, _n: u32) -> Weight {
        Weight::from_ref_time(55_642_000)
            .saturating_add(T::DbWeight::get().reads(12))
            .saturating_add(T::DbWeight::get().writes(10))
    }
    // Storage: PredictionMarkets Disputes (r:1 w:1)
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: Balances Reserves (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerDisputeBlock (r:1 w:1)
    fn dispute_authorized() -> Weight {
        Weight::from_ref_time(34_018_000)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(4))
    }
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: Balances Reserves (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsForEdit (r:0 w:1)
    fn handle_expired_advised_market() -> Weight {
        Weight::from_ref_time(25_833_000)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: Balances Reserves (r:1 w:1)
    // Storage: PredictionMarkets Disputes (r:1 w:1)
    // Storage: MarketCommons MarketPool (r:1 w:0)
    // Storage: Swaps Pools (r:1 w:1)
    fn internal_resolve_categorical_reported() -> Weight {
        Weight::from_ref_time(66_879_000)
            .saturating_add(T::DbWeight::get().reads(5))
            .saturating_add(T::DbWeight::get().writes(4))
    }
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: Balances Reserves (r:1 w:1)
    // Storage: PredictionMarkets Disputes (r:1 w:1)
    // Storage: GlobalDisputes Winners (r:1 w:0)
    // Storage: Authorized AuthorizedOutcomeReports (r:1 w:1)
    // Storage: MarketCommons MarketPool (r:1 w:0)
    // Storage: Swaps Pools (r:1 w:1)
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: GlobalDisputes Winners (r:1 w:0)
    // Storage: Authorized AuthorizedOutcomeReports (r:1 w:0)
    // Storage: System Account (r:1 w:1)
    fn internal_resolve_categorical_disputed() -> Weight {
        Weight::from_ref_time(72_294_000)
            // Standard Error: 27_000
            .saturating_add(T::DbWeight::get().reads(6))
            .saturating_add(T::DbWeight::get().writes(4))
    }
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: Balances Reserves (r:1 w:1)
    // Storage: PredictionMarkets Disputes (r:1 w:1)
    // Storage: MarketCommons MarketPool (r:1 w:0)
    fn internal_resolve_scalar_reported() -> Weight {
        Weight::from_ref_time(30_320_000)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: Balances Reserves (r:1 w:1)
    // Storage: PredictionMarkets Disputes (r:1 w:1)
    // Storage: GlobalDisputes Winners (r:1 w:0)
    // Storage: Authorized AuthorizedOutcomeReports (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    fn internal_resolve_scalar_disputed() -> Weight {
        Weight::from_ref_time(36_011_000)
            // Standard Error: 27_000
            .saturating_add(T::DbWeight::get().reads(5))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: PredictionMarkets MarketsCollectingSubsidy (r:1 w:1)
    // Storage: PredictionMarkets LastTimeFrame (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerOpenBlock (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerOpenTimeFrame (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerCloseBlock (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerCloseTimeFrame (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerReportBlock (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerDisputeBlock (r:1 w:1)
    fn on_initialize_resolve_overhead() -> Weight {
        Weight::from_ref_time(13_886_000)
            .saturating_add(T::DbWeight::get().reads(9))
            .saturating_add(T::DbWeight::get().writes(8))
    }
    // Storage: PredictionMarkets MarketsCollectingSubsidy (r:1 w:1)
    fn process_subsidy_collecting_markets_raw(a: u32) -> Weight {
        Weight::from_ref_time(3_180_000)
            // Standard Error: 0
            .saturating_add(Weight::from_ref_time(95_000).saturating_mul(a.into()))
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    // Storage: MarketCommons Markets (r:1 w:0)
    // Storage: Tokens Accounts (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    // Storage: Tokens TotalIssuance (r:1 w:1)
    fn redeem_shares_categorical() -> Weight {
        Weight::from_ref_time(54_667_000)
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    // Storage: MarketCommons Markets (r:1 w:0)
    // Storage: Tokens Accounts (r:2 w:2)
    // Storage: System Account (r:1 w:1)
    // Storage: Tokens TotalIssuance (r:2 w:2)
    fn redeem_shares_scalar() -> Weight {
        Weight::from_ref_time(57_736_000)
            .saturating_add(T::DbWeight::get().reads(6))
            .saturating_add(T::DbWeight::get().writes(5))
    }
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerOpenTimeFrame (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerCloseTimeFrame (r:1 w:1)
    // Storage: Balances Reserves (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsForEdit (r:0 w:1)
    fn reject_market(c: u32, o: u32, r: u32) -> Weight {
        Weight::from_ref_time(38_253_000)
            // Standard Error: 0
            .saturating_add(Weight::from_ref_time(18_000).saturating_mul(c.into()))
            // Standard Error: 0
            .saturating_add(Weight::from_ref_time(13_000).saturating_mul(o.into()))
            // Standard Error: 0
            .saturating_add(Weight::from_ref_time(1_000).saturating_mul(r.into()))
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().writes(5))
    }
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: PredictionMarkets MarketIdsPerReportBlock (r:1 w:1)
    fn report(_m: u32) -> Weight {
        Weight::from_ref_time(22_398_000)
            // Standard Error: 0
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    // Storage: MarketCommons Markets (r:1 w:0)
    // Storage: System Account (r:1 w:1)
    // Storage: Tokens Accounts (r:2 w:2)
    // Storage: Tokens TotalIssuance (r:2 w:2)
    fn sell_complete_set(a: u32) -> Weight {
        Weight::from_ref_time(35_149_000)
            // Standard Error: 2_000
            .saturating_add(Weight::from_ref_time(10_429_000).saturating_mul(a.into()))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().reads((2_u64).saturating_mul(a.into())))
            .saturating_add(T::DbWeight::get().writes(1))
            .saturating_add(T::DbWeight::get().writes((2_u64).saturating_mul(a.into())))
    }
    // Storage: Swaps NextPoolId (r:1 w:1)
    // Storage: RikiddoSigmoidFeeMarketEma RikiddoPerPool (r:1 w:1)
    // Storage: MarketCommons MarketPool (r:1 w:1)
    // Storage: MarketCommons Markets (r:1 w:0)
    // Storage: PredictionMarkets MarketsCollectingSubsidy (r:1 w:1)
    // Storage: Swaps Pools (r:0 w:1)
    fn start_subsidy(a: u32) -> Weight {
        Weight::from_ref_time(22_465_000)
            // Standard Error: 0
            .saturating_add(Weight::from_ref_time(42_000).saturating_mul(a.into()))
            .saturating_add(T::DbWeight::get().reads(5))
            .saturating_add(T::DbWeight::get().writes(5))
    }
    // Storage: PredictionMarkets MarketIdsPerOpenBlock (r:1 w:1)
    // Storage: MarketCommons Markets (r:32 w:0)
    // Storage: PredictionMarkets MarketIdsPerOpenTimeFrame (r:1 w:1)
    fn market_status_manager(b: u32, f: u32) -> Weight {
        Weight::from_ref_time(25_231_000)
            // Standard Error: 0
            .saturating_add(Weight::from_ref_time(1_413_000).saturating_mul(b.into()))
            // Standard Error: 0
            .saturating_add(Weight::from_ref_time(1_399_000).saturating_mul(f.into()))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(b.into())))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(f.into())))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    // Storage: PredictionMarkets MarketIdsPerReportBlock (r:1 w:1)
    // Storage: MarketCommons Markets (r:32 w:0)
    // Storage: PredictionMarkets MarketIdsPerDisputeBlock (r:1 w:1)
    fn market_resolution_manager(r: u32, d: u32) -> Weight {
        Weight::from_ref_time(24_246_000)
            // Standard Error: 0
            .saturating_add(Weight::from_ref_time(1_414_000).saturating_mul(r.into()))
            // Standard Error: 0
            .saturating_add(Weight::from_ref_time(1_445_000).saturating_mul(d.into()))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(r.into())))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(d.into())))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    // Storage: PredictionMarkets MarketsCollectingSubsidy (r:1 w:1)
    fn process_subsidy_collecting_markets_dummy() -> Weight {
        Weight::from_ref_time(3_049_000)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
}
