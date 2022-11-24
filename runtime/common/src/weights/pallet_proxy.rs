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

//! Autogenerated weights for pallet_proxy
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-10-17, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/production/zeitgeist
// benchmark
// pallet
// --chain=dev
// --steps=50
// --repeat=20
// --pallet=pallet_proxy
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --template=./misc/frame_weight_template.hbs
// --output=./runtime/common/src/weights/

#![allow(unused_parens)]
#![allow(unused_imports)]

use core::marker::PhantomData;
use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, Weight},
};

/// Weight functions for pallet_proxy (automatically generated)
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_proxy::weights::WeightInfo for WeightInfo<T> {
    // Storage: Proxy Proxies (r:1 w:0)
    fn proxy(_p: u32) -> Weight {
        (27_909_000 as Weight).saturating_add(T::DbWeight::get().reads(1 as Weight))
    }
    // Storage: Proxy Proxies (r:1 w:0)
    // Storage: Proxy Announcements (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    fn proxy_announced(a: u32, p: u32) -> Weight {
        (44_326_000 as Weight)
            // Standard Error: 21_000
            .saturating_add((318_000 as Weight).saturating_mul(a as Weight))
            // Standard Error: 22_000
            .saturating_add((126_000 as Weight).saturating_mul(p as Weight))
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: Proxy Announcements (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    fn remove_announcement(a: u32, _p: u32) -> Weight {
        (35_854_000 as Weight)
            // Standard Error: 31_000
            .saturating_add((236_000 as Weight).saturating_mul(a as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: Proxy Announcements (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    fn reject_announcement(a: u32, p: u32) -> Weight {
        (26_915_000 as Weight)
            // Standard Error: 22_000
            .saturating_add((416_000 as Weight).saturating_mul(a as Weight))
            // Standard Error: 23_000
            .saturating_add((86_000 as Weight).saturating_mul(p as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: Proxy Proxies (r:1 w:0)
    // Storage: Proxy Announcements (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    fn announce(a: u32, p: u32) -> Weight {
        (39_845_000 as Weight)
            // Standard Error: 23_000
            .saturating_add((233_000 as Weight).saturating_mul(a as Weight))
            // Standard Error: 24_000
            .saturating_add((192_000 as Weight).saturating_mul(p as Weight))
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: Proxy Proxies (r:1 w:1)
    fn add_proxy(p: u32) -> Weight {
        (38_313_000 as Weight)
            // Standard Error: 23_000
            .saturating_add((112_000 as Weight).saturating_mul(p as Weight))
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: Proxy Proxies (r:1 w:1)
    fn remove_proxy(p: u32) -> Weight {
        (40_783_000 as Weight)
            // Standard Error: 23_000
            .saturating_add((70_000 as Weight).saturating_mul(p as Weight))
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: Proxy Proxies (r:1 w:1)
    fn remove_proxies(p: u32) -> Weight {
        (30_751_000 as Weight)
            // Standard Error: 19_000
            .saturating_add((134_000 as Weight).saturating_mul(p as Weight))
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: unknown [0x3a65787472696e7369635f696e646578] (r:1 w:0)
    // Storage: Proxy Proxies (r:1 w:1)
    fn anonymous(_p: u32) -> Weight {
        (55_336_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: Proxy Proxies (r:1 w:1)
    fn kill_anonymous(p: u32) -> Weight {
        (30_452_000 as Weight)
            // Standard Error: 35_000
            .saturating_add((256_000 as Weight).saturating_mul(p as Weight))
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
}
