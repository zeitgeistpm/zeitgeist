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
    fn dispute_authorized(d: u32, b: u32) -> Weight;
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
    fn admin_destroy_disputed_market(a: u32, d: u32, _o: u32, _c: u32, _r: u32) -> Weight {
        (176_251_000 as Weight)
            // Standard Error: 14_000
            .saturating_add((37_179_000 as Weight).saturating_mul(a as Weight))
            // Standard Error: 218_000
            .saturating_add((6_261_000 as Weight).saturating_mul(d as Weight))
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
        (198_500_000 as Weight)
            // Standard Error: 16_000
            .saturating_add((37_042_000 as Weight).saturating_mul(a as Weight))
            // Standard Error: 15_000
            .saturating_add((218_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(T::DbWeight::get().reads(7 as Weight))
            .saturating_add(T::DbWeight::get().reads((2 as Weight).saturating_mul(a as Weight)))
            .saturating_add(T::DbWeight::get().writes(8 as Weight))
            .saturating_add(T::DbWeight::get().writes((2 as Weight).saturating_mul(a as Weight)))
    }
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerOpenTimeFrame (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerCloseTimeFrame (r:1 w:1)
    // Storage: MarketCommons MarketPool (r:1 w:0)
    fn admin_move_market_to_closed(_o: u32, c: u32) -> Weight {
        (62_877_000 as Weight)
            // Standard Error: 0
            .saturating_add((71_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(T::DbWeight::get().reads(4 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerReportBlock (r:1 w:1)
    // Storage: Balances Reserves (r:1 w:1)
    // Storage: PredictionMarkets Disputes (r:1 w:1)
    // Storage: MarketCommons MarketPool (r:1 w:0)
    fn admin_move_market_to_resolved_scalar_reported(r: u32) -> Weight {
        (100_354_000 as Weight)
            // Standard Error: 0
            .saturating_add((80_000 as Weight).saturating_mul(r as Weight))
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
        (201_548_000 as Weight)
            // Standard Error: 3_000
            .saturating_add((100_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(T::DbWeight::get().reads(6 as Weight))
            .saturating_add(T::DbWeight::get().writes(5 as Weight))
    }
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: PredictionMarkets Disputes (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerDisputeBlock (r:1 w:1)
    // Storage: Balances Reserves (r:7 w:7)
    // Storage: GlobalDisputes Winners (r:1 w:0)
    // Storage: Authorized AuthorizedOutcomeReports (r:1 w:0)
    // Storage: System Account (r:7 w:7)
    // Storage: MarketCommons MarketPool (r:1 w:0)
    fn admin_move_market_to_resolved_scalar_disputed(r: u32, d: u32) -> Weight {
        (132_066_000 as Weight)
            // Standard Error: 1_000
            .saturating_add((120_000 as Weight).saturating_mul(r as Weight))
            // Standard Error: 21_000
            .saturating_add((30_950_000 as Weight).saturating_mul(d as Weight))
            .saturating_add(T::DbWeight::get().reads(8 as Weight))
            .saturating_add(T::DbWeight::get().reads((2 as Weight).saturating_mul(d as Weight)))
            .saturating_add(T::DbWeight::get().writes(5 as Weight))
            .saturating_add(T::DbWeight::get().writes((2 as Weight).saturating_mul(d as Weight)))
    }
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: PredictionMarkets Disputes (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerDisputeBlock (r:1 w:1)
    // Storage: Balances Reserves (r:7 w:7)
    // Storage: GlobalDisputes Winners (r:1 w:0)
    // Storage: Authorized AuthorizedOutcomeReports (r:1 w:0)
    // Storage: System Account (r:6 w:6)
    // Storage: MarketCommons MarketPool (r:1 w:0)
    // Storage: Swaps Pools (r:1 w:1)
    fn admin_move_market_to_resolved_categorical_disputed(r: u32, d: u32) -> Weight {
        (227_185_000 as Weight)
            // Standard Error: 6_000
            .saturating_add((69_000 as Weight).saturating_mul(r as Weight))
            // Standard Error: 90_000
            .saturating_add((33_824_000 as Weight).saturating_mul(d as Weight))
            .saturating_add(T::DbWeight::get().reads(9 as Weight))
            .saturating_add(T::DbWeight::get().reads((2 as Weight).saturating_mul(d as Weight)))
            .saturating_add(T::DbWeight::get().writes(6 as Weight))
            .saturating_add(T::DbWeight::get().writes((2 as Weight).saturating_mul(d as Weight)))
    }
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsForEdit (r:1 w:0)
    // Storage: Balances Reserves (r:1 w:1)
    fn approve_market() -> Weight {
        (62_900_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: MarketCommons Markets (r:1 w:0)
    // Storage: PredictionMarkets MarketIdsForEdit (r:1 w:1)
    fn request_edit(r: u32) -> Weight {
        (37_307_000 as Weight)
            // Standard Error: 0
            .saturating_add((3_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: MarketCommons Markets (r:1 w:0)
    // Storage: System Account (r:1 w:1)
    // Storage: Tokens Accounts (r:2 w:2)
    // Storage: Tokens TotalIssuance (r:2 w:2)
    fn buy_complete_set(a: u32) -> Weight {
        (63_624_000 as Weight)
            // Standard Error: 12_000
            .saturating_add((26_565_000 as Weight).saturating_mul(a as Weight))
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
        (75_202_000 as Weight)
            // Standard Error: 3_000
            .saturating_add((53_000 as Weight).saturating_mul(m as Weight))
            .saturating_add(T::DbWeight::get().reads(4 as Weight))
            .saturating_add(T::DbWeight::get().writes(4 as Weight))
    }
    // Storage: PredictionMarkets MarketIdsForEdit (r:1 w:1)
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerCloseTimeFrame (r:1 w:1)
    // Storage: Timestamp Now (r:1 w:0)
    fn edit_market(m: u32) -> Weight {
        (64_080_000 as Weight)
            // Standard Error: 0
            .saturating_add((117_000 as Weight).saturating_mul(m as Weight))
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
        (108_283_000 as Weight)
            // Standard Error: 14_000
            .saturating_add((42_613_000 as Weight).saturating_mul(a as Weight))
            // Standard Error: 13_000
            .saturating_add((322_000 as Weight).saturating_mul(o as Weight))
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
        (136_673_000 as Weight)
            // Standard Error: 16_000
            .saturating_add((42_795_000 as Weight).saturating_mul(a as Weight))
            .saturating_add(T::DbWeight::get().reads(7 as Weight))
            .saturating_add(T::DbWeight::get().reads((2 as Weight).saturating_mul(a as Weight)))
            .saturating_add(T::DbWeight::get().writes(6 as Weight))
            .saturating_add(T::DbWeight::get().writes((2 as Weight).saturating_mul(a as Weight)))
    }
    // Storage: MarketCommons Markets (r:1 w:0)
    // Storage: PredictionMarkets Disputes (r:1 w:0)
    // Storage: GlobalDisputes Winners (r:1 w:1)
    // Storage: GlobalDisputes Outcomes (r:7 w:7)
    // Storage: PredictionMarkets MarketIdsPerDisputeBlock (r:2 w:2)
    fn start_global_dispute(_m: u32) -> Weight {
        (143_932_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(12 as Weight))
            .saturating_add(T::DbWeight::get().writes(10 as Weight))
    }
    // Storage: PredictionMarkets Disputes (r:1 w:1)
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: Balances Reserves (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerDisputeBlock (r:1 w:1)
    fn dispute_authorized(d: u32, b: u32) -> Weight {
        (79_841_000 as Weight)
            // Standard Error: 17_000
            .saturating_add((1_894_000 as Weight).saturating_mul(d as Weight))
            // Standard Error: 1_000
            .saturating_add((82_000 as Weight).saturating_mul(b as Weight))
            .saturating_add(T::DbWeight::get().reads(4 as Weight))
            .saturating_add(T::DbWeight::get().writes(4 as Weight))
    }
    // Storage: Balances Reserves (r:1 w:1)
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsForEdit (r:0 w:1)
    fn handle_expired_advised_market() -> Weight {
        (61_850_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    // Storage: Balances Reserves (r:1 w:1)
    // Storage: PredictionMarkets Disputes (r:1 w:1)
    // Storage: MarketCommons MarketPool (r:1 w:0)
    // Storage: Swaps Pools (r:1 w:1)
    // Storage: MarketCommons Markets (r:1 w:1)
    fn internal_resolve_categorical_reported() -> Weight {
        (172_370_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(5 as Weight))
            .saturating_add(T::DbWeight::get().writes(4 as Weight))
    }
    // Storage: Balances Reserves (r:1 w:1)
    // Storage: PredictionMarkets Disputes (r:1 w:1)
    // Storage: MarketCommons MarketPool (r:1 w:0)
    // Storage: Swaps Pools (r:1 w:1)
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: GlobalDisputes Winners (r:1 w:0)
    // Storage: Authorized AuthorizedOutcomeReports (r:1 w:0)
    // Storage: System Account (r:1 w:1)
    fn internal_resolve_categorical_disputed(d: u32) -> Weight {
        (183_852_000 as Weight)
            // Standard Error: 82_000
            .saturating_add((24_286_000 as Weight).saturating_mul(d as Weight))
            .saturating_add(T::DbWeight::get().reads(6 as Weight))
            .saturating_add(T::DbWeight::get().writes(4 as Weight))
    }
    // Storage: Balances Reserves (r:1 w:1)
    // Storage: PredictionMarkets Disputes (r:1 w:1)
    // Storage: MarketCommons MarketPool (r:1 w:0)
    // Storage: MarketCommons Markets (r:1 w:1)
    fn internal_resolve_scalar_reported() -> Weight {
        (73_231_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(4 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    // Storage: Balances Reserves (r:1 w:1)
    // Storage: PredictionMarkets Disputes (r:1 w:1)
    // Storage: MarketCommons MarketPool (r:1 w:0)
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: GlobalDisputes Winners (r:1 w:0)
    // Storage: Authorized AuthorizedOutcomeReports (r:1 w:0)
    // Storage: System Account (r:1 w:1)
    fn internal_resolve_scalar_disputed(d: u32) -> Weight {
        (90_052_000 as Weight)
            // Standard Error: 74_000
            .saturating_add((21_265_000 as Weight).saturating_mul(d as Weight))
            .saturating_add(T::DbWeight::get().reads(6 as Weight))
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
        (45_170_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(9 as Weight))
            .saturating_add(T::DbWeight::get().writes(8 as Weight))
    }
    // Storage: PredictionMarkets MarketsCollectingSubsidy (r:1 w:1)
    fn process_subsidy_collecting_markets_raw(a: u32) -> Weight {
        (5_577_000 as Weight)
            // Standard Error: 8_000
            .saturating_add((507_000 as Weight).saturating_mul(a as Weight))
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: MarketCommons Markets (r:1 w:0)
    // Storage: Tokens Accounts (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    // Storage: Tokens TotalIssuance (r:1 w:1)
    fn redeem_shares_categorical() -> Weight {
        (130_241_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(4 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    // Storage: MarketCommons Markets (r:1 w:0)
    // Storage: Tokens Accounts (r:2 w:2)
    // Storage: System Account (r:1 w:1)
    // Storage: Tokens TotalIssuance (r:2 w:2)
    fn redeem_shares_scalar() -> Weight {
        (144_640_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(6 as Weight))
            .saturating_add(T::DbWeight::get().writes(5 as Weight))
    }
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerOpenTimeFrame (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerCloseTimeFrame (r:1 w:1)
    // Storage: Balances Reserves (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsForEdit (r:0 w:1)
    fn reject_market(c: u32, o: u32, r: u32) -> Weight {
        (89_918_000 as Weight)
            // Standard Error: 0
            .saturating_add((76_000 as Weight).saturating_mul(c as Weight))
            // Standard Error: 0
            .saturating_add((81_000 as Weight).saturating_mul(o as Weight))
            // Standard Error: 0
            .saturating_add((3_000 as Weight).saturating_mul(r as Weight))
            .saturating_add(T::DbWeight::get().reads(4 as Weight))
            .saturating_add(T::DbWeight::get().writes(5 as Weight))
    }
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: PredictionMarkets MarketIdsPerReportBlock (r:1 w:1)
    fn report(m: u32) -> Weight {
        (50_223_000 as Weight)
            // Standard Error: 0
            .saturating_add((16_000 as Weight).saturating_mul(m as Weight))
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: MarketCommons Markets (r:1 w:0)
    // Storage: System Account (r:1 w:1)
    // Storage: Tokens Accounts (r:2 w:2)
    // Storage: Tokens TotalIssuance (r:2 w:2)
    fn sell_complete_set(a: u32) -> Weight {
        (48_611_000 as Weight)
            // Standard Error: 12_000
            .saturating_add((33_331_000 as Weight).saturating_mul(a as Weight))
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
        (57_592_000 as Weight)
            // Standard Error: 2_000
            .saturating_add((53_000 as Weight).saturating_mul(a as Weight))
            .saturating_add(T::DbWeight::get().reads(5 as Weight))
            .saturating_add(T::DbWeight::get().writes(5 as Weight))
    }
    // Storage: PredictionMarkets MarketIdsPerOpenBlock (r:1 w:1)
    // Storage: MarketCommons Markets (r:32 w:0)
    // Storage: PredictionMarkets MarketIdsPerOpenTimeFrame (r:1 w:1)
    fn market_status_manager(b: u32, f: u32) -> Weight {
        (27_651_000 as Weight)
            // Standard Error: 3_000
            .saturating_add((6_365_000 as Weight).saturating_mul(b as Weight))
            // Standard Error: 3_000
            .saturating_add((6_378_000 as Weight).saturating_mul(f as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(b as Weight)))
            .saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(f as Weight)))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: PredictionMarkets MarketIdsPerReportBlock (r:1 w:1)
    // Storage: MarketCommons Markets (r:32 w:0)
    // Storage: PredictionMarkets MarketIdsPerDisputeBlock (r:1 w:1)
    fn market_resolution_manager(r: u32, d: u32) -> Weight {
        (29_736_000 as Weight)
            // Standard Error: 3_000
            .saturating_add((6_353_000 as Weight).saturating_mul(r as Weight))
            // Standard Error: 3_000
            .saturating_add((6_344_000 as Weight).saturating_mul(d as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(r as Weight)))
            .saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(d as Weight)))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: PredictionMarkets MarketsCollectingSubsidy (r:1 w:1)
    fn process_subsidy_collecting_markets_dummy() -> Weight {
        (5_470_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
}
