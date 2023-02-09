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
//! DATE: 2023-02-08, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
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
    fn proxy(p: u32) -> Weight {
        Weight::from_ref_time(14_846_152)
            // Standard Error: 3_656
            .saturating_add(Weight::from_ref_time(20_035).saturating_mul(p.into()))
            .saturating_add(T::DbWeight::get().reads(1))
    }
    // Storage: Proxy Proxies (r:1 w:0)
    // Storage: Proxy Announcements (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    fn proxy_announced(a: u32, p: u32) -> Weight {
        Weight::from_ref_time(29_808_110)
            // Standard Error: 3_086
            .saturating_add(Weight::from_ref_time(68_700).saturating_mul(a.into()))
            // Standard Error: 3_189
            .saturating_add(Weight::from_ref_time(27_747).saturating_mul(p.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    // Storage: Proxy Announcements (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    fn remove_announcement(a: u32, _p: u32) -> Weight {
        Weight::from_ref_time(21_154_842)
            // Standard Error: 2_354
            .saturating_add(Weight::from_ref_time(67_662).saturating_mul(a.into()))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    // Storage: Proxy Announcements (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    fn reject_announcement(a: u32, _p: u32) -> Weight {
        Weight::from_ref_time(20_587_474)
            // Standard Error: 2_191
            .saturating_add(Weight::from_ref_time(91_044).saturating_mul(a.into()))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    // Storage: Proxy Proxies (r:1 w:0)
    // Storage: Proxy Announcements (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    fn announce(a: u32, p: u32) -> Weight {
        Weight::from_ref_time(25_960_647)
            // Standard Error: 3_453
            .saturating_add(Weight::from_ref_time(90_065).saturating_mul(a.into()))
            // Standard Error: 3_567
            .saturating_add(Weight::from_ref_time(54_574).saturating_mul(p.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    // Storage: Proxy Proxies (r:1 w:1)
    fn add_proxy(p: u32) -> Weight {
        Weight::from_ref_time(21_515_673)
            // Standard Error: 9_730
            .saturating_add(Weight::from_ref_time(27_609).saturating_mul(p.into()))
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    // Storage: Proxy Proxies (r:1 w:1)
    fn remove_proxy(p: u32) -> Weight {
        Weight::from_ref_time(20_632_234)
            // Standard Error: 2_374
            .saturating_add(Weight::from_ref_time(68_381).saturating_mul(p.into()))
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    // Storage: Proxy Proxies (r:1 w:1)
    fn remove_proxies(p: u32) -> Weight {
        Weight::from_ref_time(17_793_375)
            // Standard Error: 1_672
            .saturating_add(Weight::from_ref_time(42_726).saturating_mul(p.into()))
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    // Storage: unknown [0x3a65787472696e7369635f696e646578] (r:1 w:0)
    // Storage: Proxy Proxies (r:1 w:1)
    fn create_pure(p: u32) -> Weight {
        Weight::from_ref_time(22_473_896)
            // Standard Error: 1_953
            .saturating_add(Weight::from_ref_time(15_155).saturating_mul(p.into()))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    // Storage: Proxy Proxies (r:1 w:1)
    fn kill_pure(p: u32) -> Weight {
        Weight::from_ref_time(18_850_050)
            // Standard Error: 1_796
            .saturating_add(Weight::from_ref_time(42_672).saturating_mul(p.into()))
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
}
