//! Autogenerated weights for zrml_prediction_markets
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 3.0.0
//! DATE: 2021-04-19, STEPS: `[0, ]`, REPEAT: 5000, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("battery_park"), DB CACHE: 128

// Executed Command:
// ./target/release/zeitgeist
// benchmark
// --chain
// battery_park
// --execution
// wasm
// --wasm-execution
// compiled
// --pallet
// zrml-prediction-markets
// --extrinsic
// *
// --steps
// 8
// --repeat
// 5000
// --template
// ./templates/weight_template.hbs
// --output
// ./zrml/prediction-markets/src/weights.rs

#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

///  Trait containing the required functions for weight retrival within
/// zrml_prediction_markets (automatically generated)
pub trait WeightInfoZeitgeist {
    fn create_categorical_market() -> Weight;
    fn create_scalar_market() -> Weight;
    fn approve_market() -> Weight;
    fn reject_market() -> Weight;
    fn admin_destroy_market() -> Weight;
    fn cancel_pending_market() -> Weight;
    fn buy_complete_set(a: u32) -> Weight;
    fn admin_move_market_to_closed() -> Weight;
    fn sell_complete_set(a: u32) -> Weight;
    fn report() -> Weight;
    fn dispute(a: u32) -> Weight;
    fn deploy_swap_pool_for_market(a: u32) -> Weight;
    fn admin_destroy_disputed_market(a: u32, b: u32, c: u32) -> Weight;
    fn admin_destroy_reported_market(a: u32, b: u32, c: u32) -> Weight;
    fn on_initialize_resolve_overhead() -> Weight;
    fn internal_resolve_categorical_reported(a: u32, b: u32, c: u32) -> Weight;
    fn internal_resolve_categorical_disputed(a: u32, b: u32, c: u32, d: u32) -> Weight;
    fn internal_resolve_scalar_reported() -> Weight;
    fn internal_resolve_scalar_disputed(d: u32) -> Weight;
    fn admin_move_market_to_resolved_overhead() -> Weight;
}

/// Weight functions for zrml_prediction_markets (automatically generated)
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfoZeitgeist for WeightInfo<T> {
    fn create_categorical_market() -> Weight {
        (52_721_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    fn create_scalar_market() -> Weight {
        (50_706_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    fn approve_market() -> Weight {
        (52_460_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn reject_market() -> Weight {
        (46_138_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn admin_destroy_market() -> Weight {
        (74_503_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(9 as Weight))
    }
    fn cancel_pending_market() -> Weight {
        (50_246_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn buy_complete_set(a: u32) -> Weight {
        (90_106_000 as Weight)
            // Standard Error: 2_000
            .saturating_add((23_616_000 as Weight).saturating_mul(a as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().reads((2 as Weight).saturating_mul(a as Weight)))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
            .saturating_add(T::DbWeight::get().writes((2 as Weight).saturating_mul(a as Weight)))
    }
    fn admin_move_market_to_closed() -> Weight {
        (16_902_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn sell_complete_set(a: u32) -> Weight {
        (76_316_000 as Weight)
            // Standard Error: 2_000
            .saturating_add((31_238_000 as Weight).saturating_mul(a as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().reads((2 as Weight).saturating_mul(a as Weight)))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
            .saturating_add(T::DbWeight::get().writes((2 as Weight).saturating_mul(a as Weight)))
    }
    fn report() -> Weight {
        (37_331_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    fn dispute(a: u32) -> Weight {
        (70_534_000 as Weight)
            // Standard Error: 4_000
            .saturating_add((1_081_000 as Weight).saturating_mul(a as Weight))
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
    }
    fn deploy_swap_pool_for_market(a: u32) -> Weight {
        (148_158_000 as Weight)
            // Standard Error: 10_000
            .saturating_add((64_655_000 as Weight).saturating_mul(a as Weight))
            .saturating_add(T::DbWeight::get().reads(6 as Weight))
            .saturating_add(T::DbWeight::get().reads((2 as Weight).saturating_mul(a as Weight)))
            .saturating_add(T::DbWeight::get().writes(6 as Weight))
            .saturating_add(T::DbWeight::get().writes((2 as Weight).saturating_mul(a as Weight)))
    }
    fn admin_destroy_disputed_market(a: u32, b: u32, c: u32) -> Weight {
        (0 as Weight)
            // Standard Error: 18_000
            .saturating_add((57_623_000 as Weight).saturating_mul(a as Weight))
            // Standard Error: 18_000
            .saturating_add((5_284_000 as Weight).saturating_mul(b as Weight))
            // Standard Error: 24_000
            .saturating_add((72_600_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(a as Weight)))
            .saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(c as Weight)))
            .saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(a as Weight)))
            .saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(b as Weight)))
            .saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(c as Weight)))
    }
    fn admin_destroy_reported_market(a: u32, b: u32, c: u32) -> Weight {
        (0 as Weight)
            // Standard Error: 17_000
            .saturating_add((57_590_000 as Weight).saturating_mul(a as Weight))
            // Standard Error: 17_000
            .saturating_add((5_786_000 as Weight).saturating_mul(b as Weight))
            // Standard Error: 22_000
            .saturating_add((72_699_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(a as Weight)))
            .saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(c as Weight)))
            .saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(a as Weight)))
            .saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(b as Weight)))
            .saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(c as Weight)))
    }
    fn on_initialize_resolve_overhead() -> Weight {
        (8_557_000 as Weight).saturating_add(T::DbWeight::get().reads(2 as Weight))
    }
    fn internal_resolve_categorical_reported(a: u32, b: u32, c: u32) -> Weight {
        (0 as Weight)
            // Standard Error: 18_000
            .saturating_add((57_905_000 as Weight).saturating_mul(a as Weight))
            // Standard Error: 18_000
            .saturating_add((5_832_000 as Weight).saturating_mul(b as Weight))
            // Standard Error: 24_000
            .saturating_add((73_662_000 as Weight).saturating_mul(c as Weight))
            .saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(a as Weight)))
            .saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(c as Weight)))
            .saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(a as Weight)))
            .saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(b as Weight)))
            .saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(c as Weight)))
    }
    fn internal_resolve_categorical_disputed(a: u32, b: u32, c: u32, d: u32) -> Weight {
        (0 as Weight)
            // Standard Error: 14_000
            .saturating_add((49_617_000 as Weight).saturating_mul(a as Weight))
            // Standard Error: 14_000
            .saturating_add((4_248_000 as Weight).saturating_mul(b as Weight))
            // Standard Error: 19_000
            .saturating_add((85_021_000 as Weight).saturating_mul(c as Weight))
            // Standard Error: 27_000
            .saturating_add((10_665_000 as Weight).saturating_mul(d as Weight))
            .saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(a as Weight)))
            .saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(c as Weight)))
            .saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(a as Weight)))
            .saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(b as Weight)))
            .saturating_add(T::DbWeight::get().writes((3 as Weight).saturating_mul(c as Weight)))
    }
    fn internal_resolve_scalar_reported() -> Weight {
        (67_388_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn internal_resolve_scalar_disputed(d: u32) -> Weight {
        (82_488_000 as Weight)
            // Standard Error: 25_000
            .saturating_add((14_706_000 as Weight).saturating_mul(d as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    fn admin_move_market_to_resolved_overhead() -> Weight {
		(86_915_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
}
