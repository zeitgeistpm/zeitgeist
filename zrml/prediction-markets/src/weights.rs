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
//! DATE: 2022-11-25, STEPS: `10`, REPEAT: 1000, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/production/zeitgeist
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
// --template=./misc/weight_template.hbs
// --output=./zrml/prediction-markets/src/weights.rs

#![allow(unused_parens)]
#![allow(unused_imports)]

use core::marker::PhantomData;
use frame_support::{traits::Get, weights::Weight};

///  Trait containing the required functions for weight retrival within
/// zrml_prediction_markets (automatically generated)
pub trait WeightInfoZeitgeist {
    fn admin_destroy_disputed_market(a: u32, d: u32, o: u32, c: u32, r: u32) -> Weight;
    fn admin_destroy_reported_market(a: u32, o: u32, c: u32, r: u32) -> Weight;
    fn admin_move_market_to_closed(o: u32, c: u32) -> Weight;
    fn admin_move_market_to_resolved_scalar_reported(r: u32) -> Weight;
    fn admin_move_market_to_resolved_categorical_reported(r: u32) -> Weight;
    fn admin_move_market_to_resolved_scalar_disputed(r: u32, d: u32) -> Weight;
    fn admin_move_market_to_resolved_categorical_disputed(r: u32, d: u32) -> Weight;
    fn approve_market() -> Weight;
    fn request_edit(r: u32) -> Weight;
    fn buy_complete_set(a: u32) -> Weight;
    fn create_market(m: u32) -> Weight;
    fn edit_market(m: u32) -> Weight;
    fn deploy_swap_pool_for_market_future_pool(a: u32, o: u32) -> Weight;
    fn deploy_swap_pool_for_market_open_pool(a: u32) -> Weight;
    fn start_global_dispute(m: u32) -> Weight;
    fn dispute_authorized() -> Weight;
    fn resolve_expired_mdm_authorized_scalar() -> Weight;
    fn resolve_expired_mdm_authorized_categorical() -> Weight;
    fn handle_expired_advised_market() -> Weight;
    fn internal_resolve_categorical_reported() -> Weight;
    fn internal_resolve_categorical_disputed(d: u32) -> Weight;
    fn internal_resolve_scalar_reported() -> Weight;
    fn internal_resolve_scalar_disputed(d: u32) -> Weight;
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
    // Storage: Balances Reserves (r:1 w:1)
    // Storage: System Account (r:2 w:2)
    // Storage: MarketCommons MarketPool (r:1 w:1)
    // Storage: Swaps Pools (r:1 w:1)
    // Storage: Tokens Accounts (r:2 w:2)
    // Storage: Tokens TotalIssuance (r:2 w:2)
    // Storage: PredictionMarkets Disputes (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerDisputeBlock (r:1 w:1)
    fn admin_destroy_disputed_market(a: u32, _d: u32, _o: u32, _c: u32, r: u32) -> Weight {
        (207_687_000 as Weight)
            // Standard Error: 53_000
            .saturating_add((22_986_000 as Weight).saturating_mul(a as Weight))
            // Standard Error: 52_000
            .saturating_add((276_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(T::DbWeight::get().reads(8 as Weight))
            .saturating_add(T::DbWeight::get().reads((2 as Weight).saturating_mul(a as Weight)))
            .saturating_add(T::DbWeight::get().writes(8 as Weight))
            .saturating_add(T::DbWeight::get().writes((2 as Weight).saturating_mul(a as Weight)))
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
    fn admin_destroy_reported_market(a: u32, _o: u32, c: u32, _r: u32) -> Weight {
        (122_080_000 as Weight)
            // Standard Error: 11_000
            .saturating_add((22_824_000 as Weight).saturating_mul(a as Weight))
            // Standard Error: 10_000
            .saturating_add((56_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(T::DbWeight::get().reads(7 as Weight))
            .saturating_add(T::DbWeight::get().reads((2 as Weight).saturating_mul(a as Weight)))
            .saturating_add(T::DbWeight::get().writes(8 as Weight))
            .saturating_add(T::DbWeight::get().writes((2 as Weight).saturating_mul(a as Weight)))
    }
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerOpenTimeFrame (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerCloseTimeFrame (r:1 w:1)
    // Storage: MarketCommons MarketPool (r:1 w:0)
    fn admin_move_market_to_closed(o: u32, c: u32) -> Weight {
        (37_778_000 as Weight)
            // Standard Error: 1_000
            .saturating_add((10_000 as Weight).saturating_mul(o as Weight))
            // Standard Error: 1_000
            .saturating_add((18_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(T::DbWeight::get().reads(4 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerReportBlock (r:1 w:1)
    // Storage: Balances Reserves (r:1 w:1)
    // Storage: PredictionMarkets Disputes (r:1 w:1)
    // Storage: MarketCommons MarketPool (r:1 w:0)
    fn admin_move_market_to_resolved_scalar_reported(r: u32) -> Weight {
        (65_196_000 as Weight)
            // Standard Error: 2_000
            .saturating_add((35_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(T::DbWeight::get().reads(5 as Weight))
            .saturating_add(T::DbWeight::get().writes(4 as Weight))
    }
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerReportBlock (r:1 w:1)
    // Storage: Balances Reserves (r:1 w:1)
    // Storage: PredictionMarkets Disputes (r:1 w:1)
    // Storage: MarketCommons MarketPool (r:1 w:0)
    // Storage: Swaps Pools (r:1 w:1)
    fn admin_move_market_to_resolved_categorical_reported(r: u32) -> Weight {
        (93_157_000 as Weight)
            // Standard Error: 6_000
            .saturating_add((61_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(T::DbWeight::get().reads(6 as Weight))
            .saturating_add(T::DbWeight::get().writes(5 as Weight))
    }
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: PredictionMarkets Disputes (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerDisputeBlock (r:1 w:1)
    // Storage: Balances Reserves (r:7 w:7)
    // Storage: System Account (r:6 w:6)
    // Storage: MarketCommons MarketPool (r:1 w:0)
    fn admin_move_market_to_resolved_scalar_disputed(r: u32, d: u32) -> Weight {
        (79_803_000 as Weight)
            // Standard Error: 4_000
            .saturating_add((34_000 as Weight).saturating_mul(r as Weight))
            // Standard Error: 63_000
            .saturating_add((21_525_000 as Weight).saturating_mul(d as Weight))
            .saturating_add(T::DbWeight::get().reads(5 as Weight))
            .saturating_add(T::DbWeight::get().reads((2 as Weight).saturating_mul(d as Weight)))
            .saturating_add(T::DbWeight::get().writes(4 as Weight))
            .saturating_add(T::DbWeight::get().writes((2 as Weight).saturating_mul(d as Weight)))
    }
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: PredictionMarkets Disputes (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerDisputeBlock (r:1 w:1)
    // Storage: Balances Reserves (r:7 w:7)
    // Storage: System Account (r:6 w:6)
    // Storage: MarketCommons MarketPool (r:1 w:0)
    // Storage: Swaps Pools (r:1 w:1)
    fn admin_move_market_to_resolved_categorical_disputed(r: u32, d: u32) -> Weight {
        (92_016_000 as Weight)
            // Standard Error: 21_000
            .saturating_add((258_000 as Weight).saturating_mul(r as Weight))
            // Standard Error: 301_000
            .saturating_add((23_377_000 as Weight).saturating_mul(d as Weight))
            .saturating_add(T::DbWeight::get().reads(6 as Weight))
            .saturating_add(T::DbWeight::get().reads((2 as Weight).saturating_mul(d as Weight)))
            .saturating_add(T::DbWeight::get().writes(5 as Weight))
            .saturating_add(T::DbWeight::get().writes((2 as Weight).saturating_mul(d as Weight)))
    }
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsForEdit (r:1 w:0)
    // Storage: Balances Reserves (r:1 w:1)
    fn approve_market() -> Weight {
        (41_000_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: MarketCommons Markets (r:1 w:0)
    // Storage: PredictionMarkets MarketIdsForEdit (r:1 w:1)
    fn request_edit(r: u32) -> Weight {
        (23_960_000 as Weight)
            // Standard Error: 0
            .saturating_add((1_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: MarketCommons Markets (r:1 w:0)
    // Storage: System Account (r:1 w:1)
    // Storage: Tokens Accounts (r:2 w:2)
    // Storage: Tokens TotalIssuance (r:2 w:2)
    fn buy_complete_set(a: u32) -> Weight {
        (55_744_000 as Weight)
            // Standard Error: 16_000
            .saturating_add((17_578_000 as Weight).saturating_mul(a as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().reads((2 as Weight).saturating_mul(a as Weight)))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
            .saturating_add(T::DbWeight::get().writes((2 as Weight).saturating_mul(a as Weight)))
    }
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: Balances Reserves (r:1 w:1)
    // Storage: MarketCommons MarketCounter (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerCloseTimeFrame (r:1 w:1)
    // Storage: MarketCommons Markets (r:0 w:1)
    fn create_market(m: u32) -> Weight {
        (47_383_000 as Weight)
            // Standard Error: 2_000
            .saturating_add((41_000 as Weight).saturating_mul(m as Weight))
            .saturating_add(T::DbWeight::get().reads(4 as Weight))
            .saturating_add(T::DbWeight::get().writes(4 as Weight))
    }
    // Storage: PredictionMarkets MarketIdsForEdit (r:1 w:1)
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerCloseTimeFrame (r:1 w:1)
    // Storage: Timestamp Now (r:1 w:0)
    fn edit_market(m: u32) -> Weight {
        (40_572_000 as Weight)
            // Standard Error: 1_000
            .saturating_add((34_000 as Weight).saturating_mul(m as Weight))
            .saturating_add(T::DbWeight::get().reads(4 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
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
        (59_265_000 as Weight)
            // Standard Error: 63_000
            .saturating_add((27_729_000 as Weight).saturating_mul(a as Weight))
            // Standard Error: 62_000
            .saturating_add((391_000 as Weight).saturating_mul(o as Weight))
            .saturating_add(T::DbWeight::get().reads(8 as Weight))
            .saturating_add(T::DbWeight::get().reads((2 as Weight).saturating_mul(a as Weight)))
            .saturating_add(T::DbWeight::get().writes(7 as Weight))
            .saturating_add(T::DbWeight::get().writes((2 as Weight).saturating_mul(a as Weight)))
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
        (72_610_000 as Weight)
            // Standard Error: 135_000
            .saturating_add((28_739_000 as Weight).saturating_mul(a as Weight))
            .saturating_add(T::DbWeight::get().reads(7 as Weight))
            .saturating_add(T::DbWeight::get().reads((2 as Weight).saturating_mul(a as Weight)))
            .saturating_add(T::DbWeight::get().writes(6 as Weight))
            .saturating_add(T::DbWeight::get().writes((2 as Weight).saturating_mul(a as Weight)))
    }
    fn start_global_dispute(m: u32) -> Weight {
        (6_559_000 as Weight)
            // Standard Error: 1_000
            .saturating_add((2_000 as Weight).saturating_mul(m as Weight))
    }
    // Storage: PredictionMarkets Disputes (r:1 w:1)
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: Balances Reserves (r:1 w:1)
    fn dispute_authorized() -> Weight {
        (47_000_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: PredictionMarkets Disputes (r:1 w:1)
    // Storage: Authorized AuthorizedOutcomeReports (r:1 w:0)
    // Storage: Balances Reserves (r:2 w:2)
    // Storage: System Account (r:2 w:2)
    // Storage: MarketCommons MarketPool (r:1 w:0)
    fn resolve_expired_mdm_authorized_scalar() -> Weight {
        (105_000_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(8 as Weight))
            .saturating_add(T::DbWeight::get().writes(6 as Weight))
    }
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: PredictionMarkets Disputes (r:1 w:1)
    // Storage: Authorized AuthorizedOutcomeReports (r:1 w:0)
    // Storage: Balances Reserves (r:2 w:2)
    // Storage: System Account (r:2 w:2)
    // Storage: MarketCommons MarketPool (r:1 w:0)
    // Storage: Swaps Pools (r:1 w:1)
    fn resolve_expired_mdm_authorized_categorical() -> Weight {
        (135_000_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(9 as Weight))
            .saturating_add(T::DbWeight::get().writes(7 as Weight))
    }
    // Storage: Balances Reserves (r:1 w:1)
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsForEdit (r:0 w:1)
    fn handle_expired_advised_market() -> Weight {
        (44_000_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    // Storage: Balances Reserves (r:1 w:1)
    // Storage: PredictionMarkets Disputes (r:1 w:1)
    // Storage: MarketCommons MarketPool (r:1 w:0)
    // Storage: Swaps Pools (r:1 w:1)
    // Storage: MarketCommons Markets (r:1 w:1)
    fn internal_resolve_categorical_reported() -> Weight {
        (83_000_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(5 as Weight))
            .saturating_add(T::DbWeight::get().writes(4 as Weight))
    }
    // Storage: Balances Reserves (r:1 w:1)
    // Storage: PredictionMarkets Disputes (r:1 w:1)
    // Storage: MarketCommons MarketPool (r:1 w:0)
    // Storage: Swaps Pools (r:1 w:1)
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    fn internal_resolve_categorical_disputed(d: u32) -> Weight {
        (89_440_000 as Weight)
            // Standard Error: 295_000
            .saturating_add((16_181_000 as Weight).saturating_mul(d as Weight))
            .saturating_add(T::DbWeight::get().reads(5 as Weight))
            .saturating_add(T::DbWeight::get().writes(4 as Weight))
    }
    // Storage: Balances Reserves (r:1 w:1)
    // Storage: PredictionMarkets Disputes (r:1 w:1)
    // Storage: MarketCommons MarketPool (r:1 w:0)
    // Storage: MarketCommons Markets (r:1 w:1)
    fn internal_resolve_scalar_reported() -> Weight {
        (50_000_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(4 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    // Storage: Balances Reserves (r:1 w:1)
    // Storage: PredictionMarkets Disputes (r:1 w:1)
    // Storage: MarketCommons MarketPool (r:1 w:0)
    // Storage: MarketCommons Markets (r:1 w:1)
    fn internal_resolve_scalar_disputed(d: u32) -> Weight {
        (56_385_000 as Weight)
            // Standard Error: 131_000
            .saturating_add((15_225_000 as Weight).saturating_mul(d as Weight))
            .saturating_add(T::DbWeight::get().reads(4 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
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
        (30_000_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(9 as Weight))
            .saturating_add(T::DbWeight::get().writes(8 as Weight))
    }
    // Storage: PredictionMarkets MarketsCollectingSubsidy (r:1 w:1)
    fn process_subsidy_collecting_markets_raw(a: u32) -> Weight {
        (3_535_000 as Weight)
            // Standard Error: 6_000
            .saturating_add((176_000 as Weight).saturating_mul(a as Weight))
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: MarketCommons Markets (r:1 w:0)
    // Storage: Tokens Accounts (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    // Storage: Tokens TotalIssuance (r:1 w:1)
    fn redeem_shares_categorical() -> Weight {
        (73_000_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(4 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    // Storage: MarketCommons Markets (r:1 w:0)
    // Storage: Tokens Accounts (r:2 w:2)
    // Storage: System Account (r:1 w:1)
    // Storage: Tokens TotalIssuance (r:2 w:2)
    fn redeem_shares_scalar() -> Weight {
        (98_000_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(6 as Weight))
            .saturating_add(T::DbWeight::get().writes(5 as Weight))
    }
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerOpenTimeFrame (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerCloseTimeFrame (r:1 w:1)
    // Storage: Balances Reserves (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsForEdit (r:0 w:1)
    fn reject_market(c: u32, o: u32, _r: u32) -> Weight {
        (60_929_000 as Weight)
            // Standard Error: 1_000
            .saturating_add((8_000 as Weight).saturating_mul(c as Weight))
            // Standard Error: 1_000
            .saturating_add((15_000 as Weight).saturating_mul(o as Weight))
            .saturating_add(T::DbWeight::get().reads(4 as Weight))
            .saturating_add(T::DbWeight::get().writes(5 as Weight))
    }
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: PredictionMarkets MarketIdsPerReportBlock (r:1 w:1)
    fn report(_m: u32) -> Weight {
        (31_154_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: MarketCommons Markets (r:1 w:0)
    // Storage: System Account (r:1 w:1)
    // Storage: Tokens Accounts (r:2 w:2)
    // Storage: Tokens TotalIssuance (r:2 w:2)
    fn sell_complete_set(a: u32) -> Weight {
        (44_770_000 as Weight)
            // Standard Error: 12_000
            .saturating_add((21_432_000 as Weight).saturating_mul(a as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().reads((2 as Weight).saturating_mul(a as Weight)))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
            .saturating_add(T::DbWeight::get().writes((2 as Weight).saturating_mul(a as Weight)))
    }
    // Storage: Swaps NextPoolId (r:1 w:1)
    // Storage: RikiddoSigmoidFeeMarketEma RikiddoPerPool (r:1 w:1)
    // Storage: MarketCommons MarketPool (r:1 w:1)
    // Storage: MarketCommons Markets (r:1 w:0)
    // Storage: PredictionMarkets MarketsCollectingSubsidy (r:1 w:1)
    // Storage: Swaps Pools (r:0 w:1)
    fn start_subsidy(a: u32) -> Weight {
        (34_382_000 as Weight)
            // Standard Error: 1_000
            .saturating_add((36_000 as Weight).saturating_mul(a as Weight))
            .saturating_add(T::DbWeight::get().reads(5 as Weight))
            .saturating_add(T::DbWeight::get().writes(5 as Weight))
    }
    // Storage: PredictionMarkets MarketIdsPerOpenBlock (r:1 w:1)
    // Storage: MarketCommons Markets (r:32 w:0)
    // Storage: PredictionMarkets MarketIdsPerOpenTimeFrame (r:1 w:1)
    fn market_status_manager(b: u32, f: u32) -> Weight {
        (7_909_000 as Weight)
            // Standard Error: 10_000
            .saturating_add((4_309_000 as Weight).saturating_mul(b as Weight))
            // Standard Error: 10_000
            .saturating_add((4_287_000 as Weight).saturating_mul(f as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(b as Weight)))
            .saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(f as Weight)))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: PredictionMarkets MarketIdsPerReportBlock (r:1 w:1)
    // Storage: MarketCommons Markets (r:32 w:0)
    // Storage: PredictionMarkets MarketIdsPerDisputeBlock (r:1 w:1)
    fn market_resolution_manager(r: u32, d: u32) -> Weight {
        (8_808_000 as Weight)
            // Standard Error: 11_000
            .saturating_add((4_290_000 as Weight).saturating_mul(r as Weight))
            // Standard Error: 11_000
            .saturating_add((4_285_000 as Weight).saturating_mul(d as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(r as Weight)))
            .saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(d as Weight)))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: PredictionMarkets MarketsCollectingSubsidy (r:1 w:1)
    fn process_subsidy_collecting_markets_dummy() -> Weight {
        (3_000_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
}
