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
//! DATE: 2022-10-17, STEPS: `10`, REPEAT: 1000, LOW RANGE: `[]`, HIGH RANGE: `[]`
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
    fn dispute_authorized(d: u32) -> Weight;
    fn resolve_failed_mdm_authorized_scalar(d: u32) -> Weight;
    fn resolve_failed_mdm_authorized_categorical(d: u32) -> Weight;
    fn start_global_dispute(m: u32) -> Weight;
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
    fn admin_destroy_disputed_market(a: u32, _d: u32, o: u32, c: u32, r: u32) -> Weight {
        (0 as Weight)
            // Standard Error: 54_000
            .saturating_add((32_618_000 as Weight).saturating_mul(a as Weight))
            // Standard Error: 53_000
            .saturating_add((445_000 as Weight).saturating_mul(o as Weight))
            // Standard Error: 53_000
            .saturating_add((933_000 as Weight).saturating_mul(c as Weight))
            // Standard Error: 53_000
            .saturating_add((1_231_000 as Weight).saturating_mul(r as Weight))
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
    fn admin_destroy_reported_market(a: u32, o: u32, c: u32, r: u32) -> Weight {
        (0 as Weight)
            // Standard Error: 51_000
            .saturating_add((32_741_000 as Weight).saturating_mul(a as Weight))
            // Standard Error: 50_000
            .saturating_add((1_465_000 as Weight).saturating_mul(o as Weight))
            // Standard Error: 50_000
            .saturating_add((1_069_000 as Weight).saturating_mul(c as Weight))
            // Standard Error: 50_000
            .saturating_add((210_000 as Weight).saturating_mul(r as Weight))
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
        (53_510_000 as Weight)
            // Standard Error: 2_000
            .saturating_add((2_000 as Weight).saturating_mul(o as Weight))
            // Standard Error: 2_000
            .saturating_add((24_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(T::DbWeight::get().reads(4 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerReportBlock (r:1 w:1)
    // Storage: Balances Reserves (r:1 w:1)
    // Storage: PredictionMarkets Disputes (r:1 w:1)
    // Storage: MarketCommons MarketPool (r:1 w:0)
    fn admin_move_market_to_resolved_scalar_reported(r: u32) -> Weight {
        (89_742_000 as Weight)
            // Standard Error: 5_000
            .saturating_add((36_000 as Weight).saturating_mul(r as Weight))
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
        (155_062_000 as Weight)
            // Standard Error: 8_000
            .saturating_add((155_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(T::DbWeight::get().reads(6 as Weight))
            .saturating_add(T::DbWeight::get().writes(5 as Weight))
    }
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: PredictionMarkets Disputes (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerDisputeBlock (r:1 w:1)
    // Storage: Balances Reserves (r:7 w:7)
    // Storage: Authorized AuthorizedOutcomeReports (r:1 w:0)
    // Storage: System Account (r:7 w:7)
    // Storage: MarketCommons MarketPool (r:1 w:0)
    fn admin_move_market_to_resolved_scalar_disputed(_r: u32, d: u32) -> Weight {
        (119_367_000 as Weight)
            // Standard Error: 162_000
            .saturating_add((28_533_000 as Weight).saturating_mul(d as Weight))
            .saturating_add(T::DbWeight::get().reads(7 as Weight))
            .saturating_add(T::DbWeight::get().reads((2 as Weight).saturating_mul(d as Weight)))
            .saturating_add(T::DbWeight::get().writes(6 as Weight))
            .saturating_add(T::DbWeight::get().writes((2 as Weight).saturating_mul(d as Weight)))
    }
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: PredictionMarkets Disputes (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerDisputeBlock (r:1 w:1)
    // Storage: Balances Reserves (r:7 w:7)
    // Storage: Authorized AuthorizedOutcomeReports (r:1 w:0)
    // Storage: System Account (r:6 w:6)
    // Storage: MarketCommons MarketPool (r:1 w:0)
    // Storage: Swaps Pools (r:1 w:1)
    fn admin_move_market_to_resolved_categorical_disputed(r: u32, d: u32) -> Weight {
        (166_932_000 as Weight)
            // Standard Error: 14_000
            .saturating_add((348_000 as Weight).saturating_mul(r as Weight))
            // Standard Error: 198_000
            .saturating_add((30_863_000 as Weight).saturating_mul(d as Weight))
            .saturating_add(T::DbWeight::get().reads(8 as Weight))
            .saturating_add(T::DbWeight::get().reads((2 as Weight).saturating_mul(d as Weight)))
            .saturating_add(T::DbWeight::get().writes(7 as Weight))
            .saturating_add(T::DbWeight::get().writes((2 as Weight).saturating_mul(d as Weight)))
    }
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: Balances Reserves (r:1 w:1)
    fn approve_market() -> Weight {
        (51_730_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: MarketCommons Markets (r:1 w:0)
    // Storage: PredictionMarkets MarketIdsForEdit (r:1 w:1)
    fn request_edit(_r: u32) -> Weight {
        (41_274_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
    // Storage: MarketCommons Markets (r:1 w:0)
    // Storage: System Account (r:1 w:1)
    // Storage: Tokens Accounts (r:2 w:2)
    // Storage: Tokens TotalIssuance (r:2 w:2)
    fn buy_complete_set(a: u32) -> Weight {
        (78_198_000 as Weight)
            // Standard Error: 30_000
            .saturating_add((24_549_000 as Weight).saturating_mul(a as Weight))
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
    fn create_market(_m: u32) -> Weight {
        (88_764_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(5 as Weight))
            .saturating_add(T::DbWeight::get().writes(5 as Weight))
    }
    // Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsForEdit (r:1 w:1)
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: PredictionMarkets MarketIdsPerCloseTimeFrame (r:1 w:1)
    fn edit_market(_m: u32) -> Weight {
        (62_015_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(5 as Weight))
            .saturating_add(T::DbWeight::get().writes(4 as Weight))
    }
    // Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
    // Storage: MarketCommons Markets (r:1 w:0)
    // Storage: Swaps NextPoolId (r:1 w:1)
    // Storage: Tokens Accounts (r:5 w:5)
    // Storage: System Account (r:1 w:1)
    // Storage: Tokens TotalIssuance (r:1 w:1)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: PredictionMarkets MarketIdsPerOpenTimeFrame (r:1 w:1)
    // Storage: MarketCommons MarketPool (r:1 w:1)
    // Storage: Swaps Pools (r:0 w:1)
    fn deploy_swap_pool_for_market_future_pool(a: u32, _o: u32) -> Weight {
        (122_276_000 as Weight)
            // Standard Error: 62_000
            .saturating_add((40_018_000 as Weight).saturating_mul(a as Weight))
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
        (157_360_000 as Weight)
            // Standard Error: 56_000
            .saturating_add((39_371_000 as Weight).saturating_mul(a as Weight))
            .saturating_add(T::DbWeight::get().reads(7 as Weight))
            .saturating_add(T::DbWeight::get().reads((2 as Weight).saturating_mul(a as Weight)))
            .saturating_add(T::DbWeight::get().writes(6 as Weight))
            .saturating_add(T::DbWeight::get().writes((2 as Weight).saturating_mul(a as Weight)))
    }
    // Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
    // Storage: MarketCommons Markets (r:1 w:0)
    // Storage: PredictionMarkets Disputes (r:1 w:0)
    // Storage: GlobalDisputes Winners (r:1 w:1)
    // Storage: GlobalDisputes Outcomes (r:7 w:7)
    // Storage: PredictionMarkets MarketIdsPerDisputeBlock (r:2 w:2)
    fn start_global_dispute(m: u32) -> Weight {
        (89_164_000 as Weight)
            // Standard Error: 0
            .saturating_add((21_000 as Weight).saturating_mul(m as Weight))
            .saturating_add(T::DbWeight::get().reads(13 as Weight))
            .saturating_add(T::DbWeight::get().writes(11 as Weight))
    }
    // Storage: PredictionMarkets Disputes (r:1 w:1)
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: Balances Reserves (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerDisputeBlock (r:1 w:1)
    fn dispute_authorized(_d: u32) -> Weight {
        (77_511_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(4 as Weight))
            .saturating_add(T::DbWeight::get().writes(4 as Weight))
    }
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: PredictionMarkets Disputes (r:1 w:1)
    // Storage: Authorized AuthorizedOutcomeReports (r:1 w:0)
    // Storage: Balances Reserves (r:2 w:2)
    // Storage: System Account (r:2 w:2)
    // Storage: MarketCommons MarketPool (r:1 w:0)
    fn resolve_failed_mdm_authorized_scalar(d: u32) -> Weight {
        (80_390_000 as Weight)
            // Standard Error: 25_000
            .saturating_add((20_931_000 as Weight).saturating_mul(d as Weight))
            .saturating_add(T::DbWeight::get().reads(6 as Weight))
            .saturating_add(T::DbWeight::get().reads((2 as Weight).saturating_mul(d as Weight)))
            .saturating_add(T::DbWeight::get().writes(4 as Weight))
            .saturating_add(T::DbWeight::get().writes((2 as Weight).saturating_mul(d as Weight)))
    }
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: PredictionMarkets Disputes (r:1 w:1)
    // Storage: Authorized AuthorizedOutcomeReports (r:1 w:0)
    // Storage: Balances Reserves (r:2 w:2)
    // Storage: System Account (r:2 w:2)
    // Storage: MarketCommons MarketPool (r:1 w:0)
    // Storage: Swaps Pools (r:1 w:1)
    fn resolve_failed_mdm_authorized_categorical(d: u32) -> Weight {
        (103_383_000 as Weight)
            // Standard Error: 31_000
            .saturating_add((20_787_000 as Weight).saturating_mul(d as Weight))
            .saturating_add(T::DbWeight::get().reads(7 as Weight))
            .saturating_add(T::DbWeight::get().reads((2 as Weight).saturating_mul(d as Weight)))
            .saturating_add(T::DbWeight::get().writes(5 as Weight))
            .saturating_add(T::DbWeight::get().writes((2 as Weight).saturating_mul(d as Weight)))
    }
    // Storage: Balances Reserves (r:1 w:1)
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsForEdit (r:0 w:1)
    fn handle_expired_advised_market() -> Weight {
        (60_400_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    // Storage: Balances Reserves (r:1 w:1)
    // Storage: PredictionMarkets Disputes (r:1 w:1)
    // Storage: MarketCommons MarketPool (r:1 w:0)
    // Storage: Swaps Pools (r:1 w:1)
    // Storage: MarketCommons Markets (r:1 w:1)
    fn internal_resolve_categorical_reported() -> Weight {
        (159_021_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(5 as Weight))
            .saturating_add(T::DbWeight::get().writes(4 as Weight))
    }
    // Storage: Balances Reserves (r:1 w:1)
    // Storage: PredictionMarkets Disputes (r:1 w:1)
    // Storage: MarketCommons MarketPool (r:1 w:0)
    // Storage: Swaps Pools (r:1 w:1)
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: Authorized AuthorizedOutcomeReports (r:1 w:0)
    // Storage: System Account (r:1 w:1)
    fn internal_resolve_categorical_disputed(d: u32) -> Weight {
        (166_292_000 as Weight)
            // Standard Error: 141_000
            .saturating_add((18_708_000 as Weight).saturating_mul(d as Weight))
            .saturating_add(T::DbWeight::get().reads(6 as Weight))
            .saturating_add(T::DbWeight::get().writes(4 as Weight))
    }
    // Storage: Balances Reserves (r:1 w:1)
    // Storage: PredictionMarkets Disputes (r:1 w:1)
    // Storage: MarketCommons MarketPool (r:1 w:0)
    // Storage: MarketCommons Markets (r:1 w:1)
    fn internal_resolve_scalar_reported() -> Weight {
        (74_830_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(4 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    // Storage: Balances Reserves (r:1 w:1)
    // Storage: PredictionMarkets Disputes (r:1 w:1)
    // Storage: MarketCommons MarketPool (r:1 w:0)
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: Authorized AuthorizedOutcomeReports (r:1 w:0)
    // Storage: System Account (r:1 w:1)
    fn internal_resolve_scalar_disputed(d: u32) -> Weight {
        (89_741_000 as Weight)
            // Standard Error: 78_000
            .saturating_add((20_344_000 as Weight).saturating_mul(d as Weight))
            .saturating_add(T::DbWeight::get().reads(5 as Weight))
            .saturating_add(T::DbWeight::get().writes(4 as Weight))
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
        (43_390_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(9 as Weight))
            .saturating_add(T::DbWeight::get().writes(8 as Weight))
    }
    // Storage: PredictionMarkets MarketsCollectingSubsidy (r:1 w:1)
    fn process_subsidy_collecting_markets_raw(a: u32) -> Weight {
        (5_033_000 as Weight)
            // Standard Error: 1_000
            .saturating_add((258_000 as Weight).saturating_mul(a as Weight))
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: MarketCommons Markets (r:1 w:0)
    // Storage: Tokens Accounts (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    // Storage: Tokens TotalIssuance (r:1 w:1)
    fn redeem_shares_categorical() -> Weight {
        (118_300_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(4 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    // Storage: MarketCommons Markets (r:1 w:0)
    // Storage: Tokens Accounts (r:2 w:2)
    // Storage: System Account (r:1 w:1)
    // Storage: Tokens TotalIssuance (r:2 w:2)
    fn redeem_shares_scalar() -> Weight {
        (140_541_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(6 as Weight))
            .saturating_add(T::DbWeight::get().writes(5 as Weight))
    }
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerOpenTimeFrame (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerCloseTimeFrame (r:1 w:1)
    // Storage: Balances Reserves (r:1 w:1)
    fn reject_market(c: u32, o: u32, r: u32) -> Weight {
        (70_584_000 as Weight)
            // Standard Error: 275_000
            .saturating_add((93_000 as Weight).saturating_mul(c as Weight))
            // Standard Error: 275_000
            .saturating_add((118_000 as Weight).saturating_mul(o as Weight))
            // Standard Error: 16_000
            .saturating_add((12_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(T::DbWeight::get().reads(5 as Weight))
            .saturating_add(T::DbWeight::get().writes(6 as Weight))
    }
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: PredictionMarkets MarketIdsPerReportBlock (r:1 w:1)
    fn report(m: u32) -> Weight {
        (44_948_000 as Weight)
            // Standard Error: 1_000
            .saturating_add((29_000 as Weight).saturating_mul(m as Weight))
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: MarketCommons Markets (r:1 w:0)
    // Storage: System Account (r:1 w:1)
    // Storage: Tokens Accounts (r:2 w:2)
    // Storage: Tokens TotalIssuance (r:2 w:2)
    fn sell_complete_set(a: u32) -> Weight {
        (37_982_000 as Weight)
            // Standard Error: 44_000
            .saturating_add((31_579_000 as Weight).saturating_mul(a as Weight))
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
        (52_957_000 as Weight)
            // Standard Error: 3_000
            .saturating_add((207_000 as Weight).saturating_mul(a as Weight))
            .saturating_add(T::DbWeight::get().reads(5 as Weight))
            .saturating_add(T::DbWeight::get().writes(5 as Weight))
    }
    // Storage: PredictionMarkets MarketIdsPerOpenBlock (r:1 w:1)
    // Storage: MarketCommons Markets (r:32 w:0)
    // Storage: PredictionMarkets MarketIdsPerOpenTimeFrame (r:1 w:1)
    fn market_status_manager(b: u32, f: u32) -> Weight {
        (6_247_000 as Weight)
            // Standard Error: 25_000
            .saturating_add((6_396_000 as Weight).saturating_mul(b as Weight))
            // Standard Error: 25_000
            .saturating_add((6_820_000 as Weight).saturating_mul(f as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(b as Weight)))
            .saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(f as Weight)))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: PredictionMarkets MarketIdsPerReportBlock (r:1 w:1)
    // Storage: MarketCommons Markets (r:32 w:0)
    // Storage: PredictionMarkets MarketIdsPerDisputeBlock (r:1 w:1)
    fn market_resolution_manager(r: u32, d: u32) -> Weight {
        (29_193_000 as Weight)
            // Standard Error: 23_000
            .saturating_add((6_200_000 as Weight).saturating_mul(r as Weight))
            // Standard Error: 23_000
            .saturating_add((6_182_000 as Weight).saturating_mul(d as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(r as Weight)))
            .saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(d as Weight)))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: PredictionMarkets MarketsCollectingSubsidy (r:1 w:1)
    fn process_subsidy_collecting_markets_dummy() -> Weight {
        (4_720_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
}
