//! Autogenerated weights for zrml_prediction_markets
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2021-11-26, STEPS: `2`, REPEAT: 2, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 128

// Executed Command:
// ./target/release/zeitgeist
// benchmark
// --chain=dev
// --steps=2
// --repeat=2
// --pallet=zrml_prediction_markets
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --template=./misc/weight_template.hbs
// --output=./zrml//prediction-markets/src/weights.rs

#![allow(unused_parens)]
#![allow(unused_imports)]

use core::marker::PhantomData;
use frame_support::{traits::Get, weights::Weight};

///  Trait containing the required functions for weight retrival within
/// zrml_prediction_markets (automatically generated)
pub trait WeightInfoZeitgeist {
    fn admin_destroy_disputed_market(a: u32, b: u32, c: u32) -> Weight;
    fn admin_destroy_reported_market(a: u32, b: u32, c: u32) -> Weight;
    fn admin_move_market_to_closed() -> Weight;
    fn admin_move_market_to_resolved_overhead() -> Weight;
    fn approve_market() -> Weight;
    fn buy_complete_set(a: u32) -> Weight;
    fn cancel_pending_market() -> Weight;
    fn create_categorical_market() -> Weight;
    fn create_scalar_market() -> Weight;
    fn deploy_swap_pool_for_market(a: u32) -> Weight;
    fn dispute(a: u32) -> Weight;
    fn internal_resolve_categorical_reported(a: u32, _b: u32, c: u32) -> Weight;
    fn internal_resolve_categorical_disputed(a: u32, _b: u32, _c: u32, d: u32) -> Weight;
    fn internal_resolve_scalar_reported() -> Weight;
    fn internal_resolve_scalar_disputed(d: u32) -> Weight;
    fn on_initialize_resolve_overhead() -> Weight;
    fn process_subsidy_collecting_markets_raw(a: u32) -> Weight;
    fn redeem_shares_categorical() -> Weight;
    fn redeem_shares_scalar() -> Weight;
    fn reject_market() -> Weight;
    fn report() -> Weight;
    fn sell_complete_set(a: u32) -> Weight;
    fn start_subsidy(a: u32) -> Weight;
}

/// Weight functions for zrml_prediction_markets (automatically generated)
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfoZeitgeist for WeightInfo<T> {
    fn admin_destroy_disputed_market(a: u32, b: u32, c: u32) -> Weight {
        (0 as Weight)
            // Standard Error: 2_267_000
            .saturating_add((137_865_000 as Weight).saturating_mul(a as Weight))
            // Standard Error: 2_267_000
            .saturating_add((9_868_000 as Weight).saturating_mul(b as Weight))
            // Standard Error: 2_834_000
            .saturating_add((147_716_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(a as Weight)))
            .saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(a as Weight)))
            .saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(b as Weight)))
            .saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(c as Weight)))
    }
    fn admin_destroy_reported_market(a: u32, b: u32, c: u32) -> Weight {
        (0 as Weight)
            // Standard Error: 2_851_000
            .saturating_add((135_648_000 as Weight).saturating_mul(a as Weight))
            // Standard Error: 2_851_000
            .saturating_add((7_890_000 as Weight).saturating_mul(b as Weight))
            // Standard Error: 3_564_000
            .saturating_add((136_842_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(a as Weight)))
            .saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(a as Weight)))
            .saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(b as Weight)))
            .saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(c as Weight)))
    }
    fn admin_move_market_to_closed() -> Weight {
        (30_860_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn admin_move_market_to_resolved_overhead() -> Weight {
        (283_500_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(7 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    fn approve_market() -> Weight {
        (125_531_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    fn buy_complete_set(a: u32) -> Weight {
        (204_206_000 as Weight)
            // Standard Error: 1_357_000
            .saturating_add((66_833_000 as Weight).saturating_mul(a as Weight))
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().reads((2 as Weight).saturating_mul(a as Weight)))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
            .saturating_add(T::DbWeight::get().writes((2 as Weight).saturating_mul(a as Weight)))
    }
    fn cancel_pending_market() -> Weight {
        (167_420_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    fn create_categorical_market() -> Weight {
        (145_389_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    fn create_scalar_market() -> Weight {
        (149_760_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    fn deploy_swap_pool_for_market(a: u32) -> Weight {
        (380_116_000 as Weight)
            // Standard Error: 5_946_000
            .saturating_add((125_495_000 as Weight).saturating_mul(a as Weight))
            .saturating_add(T::DbWeight::get().reads(7 as Weight))
            .saturating_add(T::DbWeight::get().reads((2 as Weight).saturating_mul(a as Weight)))
            .saturating_add(T::DbWeight::get().writes(6 as Weight))
            .saturating_add(T::DbWeight::get().writes((2 as Weight).saturating_mul(a as Weight)))
    }
    fn dispute(a: u32) -> Weight {
        (25_562_000 as Weight)
            // Standard Error: 417_000
            .saturating_add((553_000 as Weight).saturating_mul(a as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
    }
    fn internal_resolve_categorical_reported(a: u32, _b: u32, c: u32) -> Weight {
        (25_690_000 as Weight)
            // Standard Error: 70_000
            .saturating_add((171_000 as Weight).saturating_mul(a as Weight))
            // Standard Error: 88_000
            .saturating_add((43_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
    }
    fn internal_resolve_categorical_disputed(a: u32, _b: u32, _c: u32, d: u32) -> Weight {
        (26_007_000 as Weight)
            // Standard Error: 115_000
            .saturating_add((67_000 as Weight).saturating_mul(a as Weight))
            // Standard Error: 193_000
            .saturating_add((60_000 as Weight).saturating_mul(d as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
    }
    fn internal_resolve_scalar_reported() -> Weight {
        (27_910_000 as Weight).saturating_add(T::DbWeight::get().reads(2 as Weight))
    }
    fn internal_resolve_scalar_disputed(d: u32) -> Weight {
        (27_190_000 as Weight)
            // Standard Error: 182_000
            .saturating_add((94_000 as Weight).saturating_mul(d as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
    }
    fn on_initialize_resolve_overhead() -> Weight {
        (41_380_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(4 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn process_subsidy_collecting_markets_raw(a: u32) -> Weight {
        (16_692_000 as Weight)
            // Standard Error: 370_000
            .saturating_add((974_000 as Weight).saturating_mul(a as Weight))
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn redeem_shares_categorical() -> Weight {
        (210_271_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(4 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    fn redeem_shares_scalar() -> Weight {
        (306_311_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(6 as Weight))
            .saturating_add(T::DbWeight::get().writes(5 as Weight))
    }
    fn reject_market() -> Weight {
        (123_640_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    fn report() -> Weight {
        (86_460_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    fn sell_complete_set(a: u32) -> Weight {
        (162_032_000 as Weight)
            // Standard Error: 379_000
            .saturating_add((55_986_000 as Weight).saturating_mul(a as Weight))
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().reads((2 as Weight).saturating_mul(a as Weight)))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
            .saturating_add(T::DbWeight::get().writes((2 as Weight).saturating_mul(a as Weight)))
    }
    fn start_subsidy(a: u32) -> Weight {
        (123_966_000 as Weight)
            // Standard Error: 2_518_000
            .saturating_add((469_000 as Weight).saturating_mul(a as Weight))
            .saturating_add(T::DbWeight::get().reads(4 as Weight))
            .saturating_add(T::DbWeight::get().writes(5 as Weight))
    }
}
