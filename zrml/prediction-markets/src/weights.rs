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
//! DATE: 2022-09-09, STEPS: `10`, REPEAT: 1000, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/zeitgeist
// benchmark
// pallet
// --chain=dev
// --steps=1
// --repeat=10
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
    fn admin_destroy_disputed_market() -> Weight;
    fn admin_destroy_reported_market() -> Weight;
    fn admin_move_market_to_closed(o: u32, c: u32) -> Weight;
    fn admin_move_market_to_resolved_overhead() -> Weight;
    fn approve_market() -> Weight;
    fn buy_complete_set(a: u32) -> Weight;
    fn create_market(m: u32) -> Weight;
    fn deploy_swap_pool_for_market_future_pool(a: u32, o: u32) -> Weight;
    fn deploy_swap_pool_for_market_open_pool(a: u32) -> Weight;
    fn dispute_authorized(d: u32, b: u32) -> Weight;
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
    fn reject_market(c: u32, o: u32) -> Weight;
    fn report(m: u32) -> Weight;
    fn sell_complete_set(a: u32) -> Weight;
    fn start_subsidy(a: u32) -> Weight;
}

/// Weight functions for zrml_prediction_markets (automatically generated)
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfoZeitgeist for WeightInfo<T> {
    // Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: Balances Reserves (r:1 w:1)
    // Storage: System Account (r:1 w:0)
    // Storage: MarketCommons MarketPool (r:1 w:0)
    // Storage: PredictionMarkets MarketIdsPerReportBlock (r:1 w:1)
    // Storage: PredictionMarkets Disputes (r:0 w:1)
    fn admin_destroy_disputed_market() -> Weight {
        (95_370_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(6 as Weight))
            .saturating_add(T::DbWeight::get().writes(5 as Weight))
    }
    // Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: Balances Reserves (r:1 w:1)
    // Storage: System Account (r:1 w:0)
    // Storage: MarketCommons MarketPool (r:1 w:0)
    // Storage: PredictionMarkets MarketIdsPerReportBlock (r:1 w:1)
    // Storage: PredictionMarkets Disputes (r:0 w:1)
    fn admin_destroy_reported_market() -> Weight {
        (92_194_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(6 as Weight))
            .saturating_add(T::DbWeight::get().writes(5 as Weight))
    }
    // Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerOpenTimeFrame (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerCloseTimeFrame (r:1 w:1)
    // Storage: MarketCommons MarketPool (r:1 w:0)
    fn admin_move_market_to_closed(o: u32, c: u32) -> Weight {
        (35_355_000 as Weight)
            // Standard Error: 0
            .saturating_add((21_000 as Weight).saturating_mul(o as Weight))
            // Standard Error: 0
            .saturating_add((11_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(T::DbWeight::get().reads(5 as Weight))
            .saturating_add(T::DbWeight::get().writes(4 as Weight))
    }
    // Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerReportBlock (r:1 w:1)
    // Storage: Balances Reserves (r:1 w:1)
    // Storage: PredictionMarkets Disputes (r:1 w:1)
    // Storage: MarketCommons MarketPool (r:1 w:0)
    fn admin_move_market_to_resolved_overhead() -> Weight {
        (76_394_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(6 as Weight))
            .saturating_add(T::DbWeight::get().writes(5 as Weight))
    }
    // Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: Balances Reserves (r:1 w:1)
    fn approve_market() -> Weight {
        (43_342_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    // Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
    // Storage: MarketCommons Markets (r:1 w:0)
    // Storage: System Account (r:1 w:1)
    // Storage: Tokens Accounts (r:2 w:2)
    // Storage: Tokens TotalIssuance (r:2 w:2)
    fn buy_complete_set(a: u32) -> Weight {
        (54_733_000 as Weight)
            // Standard Error: 254_000
            .saturating_add((24_581_000 as Weight).saturating_mul(a as Weight))
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().reads((2 as Weight).saturating_mul(a as Weight)))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
            .saturating_add(T::DbWeight::get().writes((2 as Weight).saturating_mul(a as Weight)))
    }
    // Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: Balances Reserves (r:1 w:1)
    // Storage: MarketCommons MarketCounter (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerCloseTimeFrame (r:1 w:1)
    // Storage: MarketCommons Markets (r:0 w:1)
    fn create_market(m: u32) -> Weight {
        (41_434_000 as Weight)
            // Standard Error: 0
            .saturating_add((49_000 as Weight).saturating_mul(m as Weight))
            .saturating_add(T::DbWeight::get().reads(5 as Weight))
            .saturating_add(T::DbWeight::get().writes(5 as Weight))
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
    fn deploy_swap_pool_for_market_future_pool(a: u32, o: u32) -> Weight {
        (80_762_000 as Weight)
            // Standard Error: 17_000
            .saturating_add((26_803_000 as Weight).saturating_mul(a as Weight))
            // Standard Error: 2_000
            .saturating_add((65_000 as Weight).saturating_mul(o as Weight))
            .saturating_add(T::DbWeight::get().reads(9 as Weight))
            .saturating_add(T::DbWeight::get().reads((2 as Weight).saturating_mul(a as Weight)))
            .saturating_add(T::DbWeight::get().writes(8 as Weight))
            .saturating_add(T::DbWeight::get().writes((2 as Weight).saturating_mul(a as Weight)))
    }
    // Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
    // Storage: MarketCommons Markets (r:1 w:0)
    // Storage: Swaps NextPoolId (r:1 w:1)
    // Storage: Tokens Accounts (r:5 w:5)
    // Storage: System Account (r:1 w:1)
    // Storage: Tokens TotalIssuance (r:1 w:1)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: MarketCommons MarketPool (r:1 w:1)
    // Storage: Swaps Pools (r:0 w:1)
    fn deploy_swap_pool_for_market_open_pool(a: u32) -> Weight {
        (91_814_000 as Weight)
            // Standard Error: 17_000
            .saturating_add((27_275_000 as Weight).saturating_mul(a as Weight))
            .saturating_add(T::DbWeight::get().reads(8 as Weight))
            .saturating_add(T::DbWeight::get().reads((2 as Weight).saturating_mul(a as Weight)))
            .saturating_add(T::DbWeight::get().writes(7 as Weight))
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
    // Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
    // Storage: Balances Reserves (r:1 w:1)
    // Storage: PredictionMarkets Disputes (r:1 w:1)
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: Balances Reserves (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerDisputeBlock (r:1 w:1)
    fn dispute_authorized(d: u32, b: u32) -> Weight {
        (46_162_000 as Weight)
            // Standard Error: 10_000
            .saturating_add((990_000 as Weight).saturating_mul(d as Weight))
            // Standard Error: 0
            .saturating_add((33_000 as Weight).saturating_mul(b as Weight))
            .saturating_add(T::DbWeight::get().reads(5 as Weight))
            .saturating_add(T::DbWeight::get().writes(5 as Weight))
    }
    // Storage: Balances Reserves (r:1 w:1)
    // Storage: MarketCommons Markets (r:1 w:1)
    fn handle_expired_advised_market() -> Weight {
        (79_300_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: Balances Reserves (r:1 w:1)
    // Storage: PredictionMarkets Disputes (r:1 w:1)
    // Storage: MarketCommons MarketPool (r:1 w:0)
    fn internal_resolve_categorical_reported() -> Weight {
        (80_331_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(4 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: Balances Reserves (r:1 w:1)
    // Storage: PredictionMarkets Disputes (r:1 w:1)
    // Storage: MarketCommons MarketPool (r:1 w:0)
    // Storage: Authorized AuthorizedOutcomeReports (r:1 w:0)
    fn internal_resolve_categorical_disputed(d: u32) -> Weight {
        (63_214_000 as Weight)
            // Standard Error: 982_000
            .saturating_add((23_592_000 as Weight).saturating_mul(d as Weight))
            .saturating_add(T::DbWeight::get().reads(4 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: Balances Reserves (r:1 w:1)
    // Storage: PredictionMarkets Disputes (r:1 w:1)
    // Storage: MarketCommons MarketPool (r:1 w:0)
    fn internal_resolve_scalar_reported() -> Weight {
        (68_860_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(4 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: Balances Reserves (r:1 w:1)
    // Storage: PredictionMarkets Disputes (r:1 w:1)
    // Storage: MarketCommons MarketPool (r:1 w:0)
    // Storage: Authorized AuthorizedOutcomeReports (r:1 w:0)
    fn internal_resolve_scalar_disputed(d: u32) -> Weight {
        (71_186_000 as Weight)
            // Standard Error: 761_000
            .saturating_add((20_381_000 as Weight).saturating_mul(d as Weight))
            .saturating_add(T::DbWeight::get().reads(4 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: PredictionMarkets MarketsCollectingSubsidy (r:1 w:1)
    // Storage: PredictionMarkets LastTimeFrame (r:1 w:1)
    // Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerOpenBlock (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerOpenTimeFrame (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerCloseBlock (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerCloseTimeFrame (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerReportBlock (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerDisputeBlock (r:1 w:1)
    fn on_initialize_resolve_overhead() -> Weight {
        (36_339_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(10 as Weight))
            .saturating_add(T::DbWeight::get().writes(9 as Weight))
    }
    // Storage: PredictionMarkets MarketsCollectingSubsidy (r:1 w:1)
    fn process_subsidy_collecting_markets_raw(a: u32) -> Weight {
        (4_265_000 as Weight)
            // Standard Error: 10_000
            .saturating_add((257_000 as Weight).saturating_mul(a as Weight))
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
    // Storage: MarketCommons Markets (r:1 w:0)
    // Storage: Tokens Accounts (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    // Storage: Tokens TotalIssuance (r:1 w:1)
    fn redeem_shares_categorical() -> Weight {
        (104_056_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(5 as Weight))
            .saturating_add(T::DbWeight::get().writes(4 as Weight))
    }
    // Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
    // Storage: MarketCommons Markets (r:1 w:0)
    // Storage: Tokens Accounts (r:2 w:2)
    // Storage: System Account (r:1 w:1)
    // Storage: Tokens TotalIssuance (r:2 w:2)
    fn redeem_shares_scalar() -> Weight {
        (117_231_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(7 as Weight))
            .saturating_add(T::DbWeight::get().writes(6 as Weight))
    }
    // Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerOpenTimeFrame (r:1 w:1)
    // Storage: PredictionMarkets MarketIdsPerCloseTimeFrame (r:1 w:1)
    // Storage: Balances Reserves (r:1 w:1)
    fn reject_market(c: u32, o: u32) -> Weight {
        (72_530_000 as Weight)
            // Standard Error: 0
            .saturating_add((7_000 as Weight).saturating_mul(c as Weight))
            // Standard Error: 0
            .saturating_add((21_000 as Weight).saturating_mul(o as Weight))
            .saturating_add(T::DbWeight::get().reads(5 as Weight))
            .saturating_add(T::DbWeight::get().writes(5 as Weight))
    }
    // Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
    // Storage: MarketCommons Markets (r:1 w:1)
    // Storage: Timestamp Now (r:1 w:0)
    // Storage: PredictionMarkets MarketIdsPerReportBlock (r:1 w:1)
    fn report(m: u32) -> Weight {
        (25_962_000 as Weight)
            // Standard Error: 0
            .saturating_add((14_000 as Weight).saturating_mul(m as Weight))
            .saturating_add(T::DbWeight::get().reads(4 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    // Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
    // Storage: MarketCommons Markets (r:1 w:0)
    // Storage: System Account (r:1 w:1)
    // Storage: Tokens Accounts (r:2 w:2)
    // Storage: Tokens TotalIssuance (r:2 w:2)
    fn sell_complete_set(a: u32) -> Weight {
        (42_416_000 as Weight)
            // Standard Error: 16_000
            .saturating_add((21_826_000 as Weight).saturating_mul(a as Weight))
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().reads((2 as Weight).saturating_mul(a as Weight)))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
            .saturating_add(T::DbWeight::get().writes((2 as Weight).saturating_mul(a as Weight)))
    }
    // Storage: unknown [0x3a7472616e73616374696f6e5f6c6576656c3a] (r:1 w:1)
    // Storage: Swaps NextPoolId (r:1 w:1)
    // Storage: RikiddoSigmoidFeeMarketEma RikiddoPerPool (r:1 w:1)
    // Storage: MarketCommons MarketPool (r:1 w:1)
    // Storage: MarketCommons Markets (r:1 w:0)
    // Storage: PredictionMarkets MarketsCollectingSubsidy (r:1 w:1)
    // Storage: Swaps Pools (r:0 w:1)
    fn start_subsidy(a: u32) -> Weight {
        (42_981_000 as Weight)
            // Standard Error: 7_000
            .saturating_add((103_000 as Weight).saturating_mul(a as Weight))
            .saturating_add(T::DbWeight::get().reads(6 as Weight))
            .saturating_add(T::DbWeight::get().writes(6 as Weight))
    }
}
