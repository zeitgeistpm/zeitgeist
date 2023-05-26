// Copyright 2022-2023 Forecasting Technologies LTD.
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

//! Autogenerated weights for zrml_swaps
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-02-08, STEPS: `10`, REPEAT: 1000, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/production/zeitgeist
// benchmark
// pallet
// --chain=dev
// --steps=10
// --repeat=1000
// --pallet=zrml_swaps
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --template=./misc/weight_template.hbs
// --output=./zrml/swaps/src/weights.rs

#![allow(unused_parens)]
#![allow(unused_imports)]

use core::marker::PhantomData;
use frame_support::{traits::Get, weights::Weight};

///  Trait containing the required functions for weight retrival within
/// zrml_swaps (automatically generated)
pub trait WeightInfoZeitgeist {
    fn admin_clean_up_pool_cpmm_categorical(a: u32) -> Weight;
    fn admin_clean_up_pool_cpmm_scalar() -> Weight;
    fn apply_to_cached_pools_execute_arbitrage(a: u32) -> Weight;
    fn apply_to_cached_pools_noop(a: u32) -> Weight;
    fn destroy_pool_in_subsidy_phase(a: u32) -> Weight;
    fn distribute_pool_share_rewards(a: u32, b: u32) -> Weight;
    fn end_subsidy_phase(a: u32, b: u32) -> Weight;
    fn execute_arbitrage_buy_burn(a: u32) -> Weight;
    fn execute_arbitrage_mint_sell(a: u32) -> Weight;
    fn execute_arbitrage_skipped(a: u32) -> Weight;
    fn pool_exit(a: u32) -> Weight;
    fn pool_exit_subsidy() -> Weight;
    fn pool_exit_with_exact_asset_amount() -> Weight;
    fn pool_exit_with_exact_pool_amount() -> Weight;
    fn pool_join(a: u32) -> Weight;
    fn pool_join_subsidy() -> Weight;
    fn pool_join_with_exact_asset_amount() -> Weight;
    fn pool_join_with_exact_pool_amount() -> Weight;
    fn clean_up_pool_categorical_without_reward_distribution(a: u32) -> Weight;
    fn swap_exact_amount_in_cpmm() -> Weight;
    fn swap_exact_amount_in_rikiddo(a: u32) -> Weight;
    fn swap_exact_amount_out_cpmm() -> Weight;
    fn swap_exact_amount_out_rikiddo(a: u32) -> Weight;
    fn open_pool(a: u32) -> Weight;
    fn close_pool(a: u32) -> Weight;
    fn destroy_pool(a: u32) -> Weight;
}

/// Weight functions for zrml_swaps (automatically generated)
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfoZeitgeist for WeightInfo<T> {
    // Storage: MarketCommons Markets (r:1 w:0)
    // Storage: MarketCommons MarketPool (r:1 w:0)
    // Storage: Swaps Pools (r:1 w:1)
    fn admin_clean_up_pool_cpmm_categorical(a: u32) -> Weight {
        Weight::from_ref_time(25_692_254)
            // Standard Error: 614
            .saturating_add(Weight::from_ref_time(284_413).saturating_mul(a.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    // Storage: MarketCommons Markets (r:1 w:0)
    // Storage: MarketCommons MarketPool (r:1 w:0)
    // Storage: Swaps Pools (r:1 w:1)
    fn admin_clean_up_pool_cpmm_scalar() -> Weight {
        Weight::from_ref_time(22_556_000)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    // Storage: Swaps PoolsCachedForArbitrage (r:8 w:7)
    // Storage: Swaps Pools (r:7 w:0)
    // Storage: Tokens Accounts (r:462 w:462)
    // Storage: System Account (r:7 w:0)
    // Storage: Tokens TotalIssuance (r:64 w:64)
    fn apply_to_cached_pools_execute_arbitrage(a: u32) -> Weight {
        Weight::from_ref_time(313_000)
            // Standard Error: 71_767
            .saturating_add(Weight::from_ref_time(912_062_521).saturating_mul(a.into()))
            .saturating_add(T::DbWeight::get().reads(43))
            .saturating_add(T::DbWeight::get().reads((70_u64).saturating_mul(a.into())))
            .saturating_add(T::DbWeight::get().writes(42))
            .saturating_add(T::DbWeight::get().writes((67_u64).saturating_mul(a.into())))
    }
    // Storage: Swaps PoolsCachedForArbitrage (r:8 w:7)
    fn apply_to_cached_pools_noop(a: u32) -> Weight {
        Weight::from_ref_time(313_000)
            // Standard Error: 999
            .saturating_add(Weight::from_ref_time(3_519_461).saturating_mul(a.into()))
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(a.into())))
            .saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(a.into())))
    }
    // Storage: Swaps Pools (r:1 w:1)
    // Storage: Swaps SubsidyProviders (r:1 w:0)
    // Storage: RikiddoSigmoidFeeMarketEma RikiddoPerPool (r:1 w:1)
    // Storage: Tokens Accounts (r:1 w:1)
    fn destroy_pool_in_subsidy_phase(a: u32) -> Weight {
        Weight::from_ref_time(20_746_815)
            // Standard Error: 6_870
            .saturating_add(Weight::from_ref_time(9_588_841).saturating_mul(a.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().reads((2_u64).saturating_mul(a.into())))
            .saturating_add(T::DbWeight::get().writes(2))
            .saturating_add(T::DbWeight::get().writes((2_u64).saturating_mul(a.into())))
    }
    // Storage: Tokens TotalIssuance (r:2 w:1)
    // Storage: Tokens Accounts (r:46 w:21)
    // Storage: System Account (r:11 w:10)
    fn distribute_pool_share_rewards(a: u32, b: u32) -> Weight {
        Weight::from_ref_time(22_860_268)
            // Standard Error: 18_875
            .saturating_add(Weight::from_ref_time(10_760_963).saturating_mul(a.into()))
            // Standard Error: 18_875
            .saturating_add(Weight::from_ref_time(19_119_205).saturating_mul(b.into()))
            .saturating_add(T::DbWeight::get().reads(7))
            .saturating_add(T::DbWeight::get().reads((3_u64).saturating_mul(a.into())))
            .saturating_add(T::DbWeight::get().reads((2_u64).saturating_mul(b.into())))
            .saturating_add(T::DbWeight::get().writes(1))
            .saturating_add(T::DbWeight::get().writes((3_u64).saturating_mul(b.into())))
    }
    // Storage: Swaps Pools (r:1 w:1)
    // Storage: Swaps SubsidyProviders (r:11 w:10)
    // Storage: Tokens Accounts (r:22 w:22)
    // Storage: System Account (r:11 w:11)
    // Storage: Tokens TotalIssuance (r:2 w:2)
    // Storage: RikiddoSigmoidFeeMarketEma RikiddoPerPool (r:1 w:0)
    fn end_subsidy_phase(a: u32, b: u32) -> Weight {
        Weight::from_ref_time(13_533_000)
            // Standard Error: 35_634
            .saturating_add(Weight::from_ref_time(8_213_437).saturating_mul(a.into()))
            // Standard Error: 236_657
            .saturating_add(Weight::from_ref_time(36_459_831).saturating_mul(b.into()))
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().reads((2_u64).saturating_mul(a.into())))
            .saturating_add(T::DbWeight::get().reads((6_u64).saturating_mul(b.into())))
            .saturating_add(T::DbWeight::get().writes((2_u64).saturating_mul(a.into())))
            .saturating_add(T::DbWeight::get().writes((6_u64).saturating_mul(b.into())))
    }
    // Storage: Swaps Pools (r:1 w:0)
    // Storage: Tokens Accounts (r:3 w:3)
    // Storage: System Account (r:1 w:0)
    // Storage: Tokens TotalIssuance (r:1 w:1)
    fn execute_arbitrage_buy_burn(a: u32) -> Weight {
        Weight::from_ref_time(33_878_547)
            // Standard Error: 9_283
            .saturating_add(Weight::from_ref_time(15_010_607).saturating_mul(a.into()))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().reads((2_u64).saturating_mul(a.into())))
            .saturating_add(T::DbWeight::get().writes(4))
            .saturating_add(T::DbWeight::get().writes((2_u64).saturating_mul(a.into())))
    }
    // Storage: Swaps Pools (r:1 w:0)
    // Storage: Tokens Accounts (r:3 w:3)
    // Storage: System Account (r:2 w:1)
    // Storage: Tokens TotalIssuance (r:1 w:1)
    fn execute_arbitrage_mint_sell(a: u32) -> Weight {
        Weight::from_ref_time(38_285_437)
            // Standard Error: 9_089
            .saturating_add(Weight::from_ref_time(13_998_473).saturating_mul(a.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().reads((2_u64).saturating_mul(a.into())))
            .saturating_add(T::DbWeight::get().writes(1))
            .saturating_add(T::DbWeight::get().writes((2_u64).saturating_mul(a.into())))
    }
    // Storage: Swaps Pools (r:1 w:0)
    // Storage: Tokens Accounts (r:2 w:0)
    fn execute_arbitrage_skipped(a: u32) -> Weight {
        Weight::from_ref_time(14_828_448)
            // Standard Error: 1_025
            .saturating_add(Weight::from_ref_time(2_204_192).saturating_mul(a.into()))
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(a.into())))
    }
    // Storage: Swaps Pools (r:1 w:0)
    // Storage: Tokens TotalIssuance (r:1 w:1)
    // Storage: Tokens Accounts (r:5 w:5)
    // Storage: System Account (r:1 w:0)
    fn pool_exit(a: u32) -> Weight {
        Weight::from_ref_time(37_507_291)
            // Standard Error: 2_130
            .saturating_add(Weight::from_ref_time(11_575_025).saturating_mul(a.into()))
            .saturating_add(T::DbWeight::get().reads(4))
            .saturating_add(T::DbWeight::get().reads((2_u64).saturating_mul(a.into())))
            .saturating_add(T::DbWeight::get().writes(2))
            .saturating_add(T::DbWeight::get().writes((2_u64).saturating_mul(a.into())))
    }
    // Storage: Swaps Pools (r:1 w:1)
    // Storage: Swaps SubsidyProviders (r:1 w:1)
    // Storage: Tokens Accounts (r:1 w:1)
    fn pool_exit_subsidy() -> Weight {
        Weight::from_ref_time(37_934_000)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    // Storage: Swaps Pools (r:1 w:0)
    // Storage: Tokens Accounts (r:3 w:3)
    // Storage: Tokens TotalIssuance (r:1 w:1)
    // Storage: System Account (r:1 w:0)
    // Storage: Swaps PoolsCachedForArbitrage (r:0 w:1)
    fn pool_exit_with_exact_asset_amount() -> Weight {
        Weight::from_ref_time(69_851_000)
            .saturating_add(T::DbWeight::get().reads(6))
            .saturating_add(T::DbWeight::get().writes(5))
    }
    // Storage: Swaps Pools (r:1 w:0)
    // Storage: Tokens TotalIssuance (r:1 w:1)
    // Storage: Tokens Accounts (r:3 w:3)
    // Storage: System Account (r:1 w:0)
    // Storage: Swaps PoolsCachedForArbitrage (r:0 w:1)
    fn pool_exit_with_exact_pool_amount() -> Weight {
        Weight::from_ref_time(69_798_000)
            .saturating_add(T::DbWeight::get().reads(6))
            .saturating_add(T::DbWeight::get().writes(5))
    }
    // Storage: Swaps Pools (r:1 w:0)
    // Storage: Tokens TotalIssuance (r:1 w:1)
    // Storage: Tokens Accounts (r:5 w:5)
    fn pool_join(a: u32) -> Weight {
        Weight::from_ref_time(33_804_359)
            // Standard Error: 2_236
            .saturating_add(Weight::from_ref_time(11_351_296).saturating_mul(a.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().reads((2_u64).saturating_mul(a.into())))
            .saturating_add(T::DbWeight::get().writes(2))
            .saturating_add(T::DbWeight::get().writes((2_u64).saturating_mul(a.into())))
    }
    // Storage: Swaps Pools (r:1 w:1)
    // Storage: Tokens Accounts (r:1 w:1)
    // Storage: Swaps SubsidyProviders (r:1 w:1)
    fn pool_join_subsidy() -> Weight {
        Weight::from_ref_time(38_045_000)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    // Storage: Swaps Pools (r:1 w:0)
    // Storage: Tokens TotalIssuance (r:1 w:1)
    // Storage: Tokens Accounts (r:3 w:3)
    // Storage: Swaps PoolsCachedForArbitrage (r:0 w:1)
    fn pool_join_with_exact_asset_amount() -> Weight {
        Weight::from_ref_time(63_348_000)
            .saturating_add(T::DbWeight::get().reads(5))
            .saturating_add(T::DbWeight::get().writes(5))
    }
    // Storage: Swaps Pools (r:1 w:0)
    // Storage: Tokens TotalIssuance (r:1 w:1)
    // Storage: Tokens Accounts (r:3 w:3)
    // Storage: Swaps PoolsCachedForArbitrage (r:0 w:1)
    fn pool_join_with_exact_pool_amount() -> Weight {
        Weight::from_ref_time(63_817_000)
            .saturating_add(T::DbWeight::get().reads(5))
            .saturating_add(T::DbWeight::get().writes(5))
    }
    // Storage: Swaps Pools (r:1 w:1)
    fn clean_up_pool_categorical_without_reward_distribution(a: u32) -> Weight {
        Weight::from_ref_time(5_597_769)
            // Standard Error: 274
            .saturating_add(Weight::from_ref_time(163_354).saturating_mul(a.into()))
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    // Storage: Swaps Pools (r:1 w:0)
    // Storage: Tokens Accounts (r:4 w:4)
    // Storage: System Account (r:1 w:0)
    // Storage: Swaps PoolsCachedForArbitrage (r:0 w:1)
    fn swap_exact_amount_in_cpmm() -> Weight {
        Weight::from_ref_time(81_657_000)
            .saturating_add(T::DbWeight::get().reads(6))
            .saturating_add(T::DbWeight::get().writes(5))
    }
    // Storage: Swaps Pools (r:1 w:0)
    // Storage: Tokens Accounts (r:3 w:3)
    // Storage: Tokens TotalIssuance (r:2 w:1)
    // Storage: RikiddoSigmoidFeeMarketEma RikiddoPerPool (r:1 w:1)
    // Storage: System Account (r:1 w:0)
    // Storage: Timestamp Now (r:1 w:0)
    fn swap_exact_amount_in_rikiddo(a: u32) -> Weight {
        Weight::from_ref_time(72_975_675)
            // Standard Error: 2_451
            .saturating_add(Weight::from_ref_time(7_367_384).saturating_mul(a.into()))
            .saturating_add(T::DbWeight::get().reads(6))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(a.into())))
            .saturating_add(T::DbWeight::get().writes(5))
    }
    // Storage: Swaps Pools (r:1 w:0)
    // Storage: Tokens Accounts (r:4 w:4)
    // Storage: System Account (r:1 w:0)
    // Storage: Swaps PoolsCachedForArbitrage (r:0 w:1)
    fn swap_exact_amount_out_cpmm() -> Weight {
        Weight::from_ref_time(81_923_000)
            .saturating_add(T::DbWeight::get().reads(6))
            .saturating_add(T::DbWeight::get().writes(5))
    }
    // Storage: Swaps Pools (r:1 w:0)
    // Storage: Tokens Accounts (r:4 w:3)
    // Storage: Tokens TotalIssuance (r:2 w:1)
    // Storage: RikiddoSigmoidFeeMarketEma RikiddoPerPool (r:1 w:1)
    // Storage: Timestamp Now (r:1 w:0)
    fn swap_exact_amount_out_rikiddo(a: u32) -> Weight {
        Weight::from_ref_time(52_010_503)
            // Standard Error: 3_021
            .saturating_add(Weight::from_ref_time(12_822_294).saturating_mul(a.into()))
            .saturating_add(T::DbWeight::get().reads(6))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(a.into())))
            .saturating_add(T::DbWeight::get().writes(5))
    }
    // Storage: Swaps Pools (r:1 w:1)
    fn open_pool(a: u32) -> Weight {
        Weight::from_ref_time(12_568_363)
            // Standard Error: 435
            .saturating_add(Weight::from_ref_time(244_095).saturating_mul(a.into()))
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    // Storage: Swaps Pools (r:1 w:1)
    fn close_pool(a: u32) -> Weight {
        Weight::from_ref_time(11_445_465)
            // Standard Error: 326
            .saturating_add(Weight::from_ref_time(192_402).saturating_mul(a.into()))
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    // Storage: Swaps Pools (r:1 w:1)
    // Storage: Tokens Accounts (r:2 w:2)
    // Storage: System Account (r:1 w:1)
    // Storage: Tokens TotalIssuance (r:2 w:2)
    fn destroy_pool(a: u32) -> Weight {
        Weight::from_ref_time(22_588_099)
            // Standard Error: 1_893
            .saturating_add(Weight::from_ref_time(10_954_266).saturating_mul(a.into()))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().reads((2_u64).saturating_mul(a.into())))
            .saturating_add(T::DbWeight::get().writes(2))
            .saturating_add(T::DbWeight::get().writes((2_u64).saturating_mul(a.into())))
    }
}
