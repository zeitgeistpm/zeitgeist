
//! Autogenerated weights for zrml_orderbook_v1
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 3.0.0
//! DATE: 2021-06-29, STEPS: `[0, ]`, REPEAT: 1000, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: None, DB CACHE: 128

// Executed Command:
// ./target/release/zeitgeist
// benchmark
// --dev
// --execution
// wasm
// --wasm-execution
// compiled
// --heap-pages=4096
// --pallet
// zrml-orderbook-v1
// --extrinsic
// *
// --steps
// 0
// --repeat
// 1000
// --template
// ./misc/weight_template.hbs
// --output
// ./zrml/orderbook-v1/src/weights.rs


#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

///  Trait containing the required functions for weight retrival within
/// zrml_orderbook_v1 (automatically generated)
pub trait WeightInfoZeitgeist {
	fn cancel_order_ask() -> Weight;
	fn cancel_order_bid() -> Weight;
	fn fill_order_ask() -> Weight;
	fn fill_order_bid() -> Weight;
	fn make_order_ask() -> Weight;
	fn make_order_bid() -> Weight;
}

/// Weight functions for zrml_orderbook_v1 (automatically generated)
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfoZeitgeist for WeightInfo<T> {
	fn cancel_order_ask() -> Weight {
		(52_301_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	fn cancel_order_bid() -> Weight {
		(65_701_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	fn fill_order_ask() -> Weight {
		(151_603_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	fn fill_order_bid() -> Weight {
		(164_903_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	fn make_order_ask() -> Weight {
		(92_702_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
	}
	fn make_order_bid() -> Weight {
		(81_802_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
}
