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

//! Autogenerated weights for pallet_parachain_staking
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-06-16, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/production/zeitgeist
// benchmark
// pallet
// --chain=dev
// --steps=50
// --repeat=20
// --pallet=pallet_parachain_staking
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

/// Weight functions for pallet_parachain_staking (automatically generated)
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_parachain_staking::weights::WeightInfo for WeightInfo<T> {
    // Storage: ParachainStaking InflationConfig (r:1 w:1)
    fn set_staking_expectations() -> Weight {
        Weight::from_ref_time(35_670_000)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    // Storage: ParachainStaking InflationConfig (r:1 w:1)
    // Storage: ParachainStaking Round (r:1 w:0)
    fn set_inflation() -> Weight {
        Weight::from_ref_time(87_440_000)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    // Storage: ParachainStaking ParachainBondInfo (r:1 w:1)
    fn set_parachain_bond_account() -> Weight {
        Weight::from_ref_time(34_790_000)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    // Storage: ParachainStaking ParachainBondInfo (r:1 w:1)
    fn set_parachain_bond_reserve_percent() -> Weight {
        Weight::from_ref_time(33_430_000)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    // Storage: ParachainStaking TotalSelected (r:1 w:1)
    // Storage: ParachainStaking Round (r:1 w:0)
    fn set_total_selected() -> Weight {
        Weight::from_ref_time(37_440_000)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    // Storage: ParachainStaking CollatorCommission (r:1 w:1)
    fn set_collator_commission() -> Weight {
        Weight::from_ref_time(32_340_000)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    // Storage: ParachainStaking Round (r:1 w:1)
    // Storage: ParachainStaking TotalSelected (r:1 w:0)
    // Storage: ParachainStaking InflationConfig (r:1 w:1)
    fn set_blocks_per_round() -> Weight {
        Weight::from_ref_time(93_860_000)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    // Storage: ParachainStaking CandidateInfo (r:1 w:1)
    // Storage: ParachainStaking DelegatorState (r:1 w:0)
    // Storage: ParachainStaking CandidatePool (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    // Storage: Balances Locks (r:1 w:1)
    // Storage: ParachainStaking Total (r:1 w:1)
    // Storage: ParachainStaking TopDelegations (r:0 w:1)
    // Storage: ParachainStaking BottomDelegations (r:0 w:1)
    fn join_candidates(x: u32) -> Weight {
        Weight::from_ref_time(123_056_992)
            // Standard Error: 3_370
            .saturating_add(Weight::from_ref_time(375_510).saturating_mul(x.into()))
            .saturating_add(T::DbWeight::get().reads(6))
            .saturating_add(T::DbWeight::get().writes(7))
    }
    // Storage: ParachainStaking CandidateInfo (r:1 w:1)
    // Storage: ParachainStaking Round (r:1 w:0)
    // Storage: ParachainStaking CandidatePool (r:1 w:1)
    fn schedule_leave_candidates(x: u32) -> Weight {
        Weight::from_ref_time(116_170_118)
            // Standard Error: 4_082
            .saturating_add(Weight::from_ref_time(332_657).saturating_mul(x.into()))
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    // Storage: ParachainStaking CandidateInfo (r:1 w:1)
    // Storage: ParachainStaking Round (r:1 w:0)
    // Storage: ParachainStaking TopDelegations (r:1 w:1)
    // Storage: ParachainStaking DelegatorState (r:1 w:1)
    // Storage: Balances Locks (r:2 w:2)
    // Storage: System Account (r:2 w:2)
    // Storage: ParachainStaking DelegationScheduledRequests (r:1 w:1)
    // Storage: ParachainStaking AutoCompoundingDelegations (r:1 w:1)
    // Storage: ParachainStaking BottomDelegations (r:1 w:1)
    // Storage: ParachainStaking Total (r:1 w:1)
    fn execute_leave_candidates(x: u32) -> Weight {
        Weight::from_ref_time(143_510_000)
            // Standard Error: 140_714
            .saturating_add(Weight::from_ref_time(42_799_568).saturating_mul(x.into()))
            .saturating_add(T::DbWeight::get().reads(6))
            .saturating_add(T::DbWeight::get().reads((3_u64).saturating_mul(x.into())))
            .saturating_add(T::DbWeight::get().writes(5))
            .saturating_add(T::DbWeight::get().writes((3_u64).saturating_mul(x.into())))
    }
    // Storage: ParachainStaking CandidateInfo (r:1 w:1)
    // Storage: ParachainStaking CandidatePool (r:1 w:1)
    fn cancel_leave_candidates(x: u32) -> Weight {
        Weight::from_ref_time(122_928_231)
            // Standard Error: 5_155
            .saturating_add(Weight::from_ref_time(356_447).saturating_mul(x.into()))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    // Storage: ParachainStaking CandidateInfo (r:1 w:1)
    // Storage: ParachainStaking CandidatePool (r:1 w:1)
    fn go_offline() -> Weight {
        Weight::from_ref_time(70_580_000)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    // Storage: ParachainStaking CandidateInfo (r:1 w:1)
    // Storage: ParachainStaking CandidatePool (r:1 w:1)
    fn go_online() -> Weight {
        Weight::from_ref_time(80_801_000)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    // Storage: ParachainStaking CandidateInfo (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    // Storage: ParachainStaking Total (r:1 w:1)
    // Storage: Balances Locks (r:1 w:1)
    // Storage: ParachainStaking CandidatePool (r:1 w:1)
    fn candidate_bond_more() -> Weight {
        Weight::from_ref_time(89_120_000)
            .saturating_add(T::DbWeight::get().reads(5))
            .saturating_add(T::DbWeight::get().writes(5))
    }
    // Storage: ParachainStaking CandidateInfo (r:1 w:1)
    // Storage: ParachainStaking Round (r:1 w:0)
    fn schedule_candidate_bond_less() -> Weight {
        Weight::from_ref_time(50_400_000)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    // Storage: ParachainStaking CandidateInfo (r:1 w:1)
    // Storage: ParachainStaking Round (r:1 w:0)
    // Storage: ParachainStaking Total (r:1 w:1)
    // Storage: Balances Locks (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    // Storage: ParachainStaking CandidatePool (r:1 w:1)
    fn execute_candidate_bond_less() -> Weight {
        Weight::from_ref_time(100_361_000)
            .saturating_add(T::DbWeight::get().reads(6))
            .saturating_add(T::DbWeight::get().writes(5))
    }
    // Storage: ParachainStaking CandidateInfo (r:1 w:1)
    fn cancel_candidate_bond_less() -> Weight {
        Weight::from_ref_time(45_510_000)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    // Storage: System Account (r:1 w:1)
    // Storage: ParachainStaking DelegatorState (r:1 w:1)
    // Storage: ParachainStaking CandidateInfo (r:1 w:1)
    // Storage: ParachainStaking TopDelegations (r:1 w:1)
    // Storage: ParachainStaking CandidatePool (r:1 w:1)
    // Storage: Balances Locks (r:1 w:1)
    // Storage: ParachainStaking Total (r:1 w:1)
    fn delegate(x: u32, y: u32) -> Weight {
        Weight::from_ref_time(186_651_756)
            // Standard Error: 55_002
            .saturating_add(Weight::from_ref_time(867_499).saturating_mul(x.into()))
            // Standard Error: 18_044
            .saturating_add(Weight::from_ref_time(553_824).saturating_mul(y.into()))
            .saturating_add(T::DbWeight::get().reads(7))
            .saturating_add(T::DbWeight::get().writes(7))
    }
    // Storage: ParachainStaking DelegatorState (r:1 w:1)
    // Storage: ParachainStaking Round (r:1 w:0)
    // Storage: ParachainStaking DelegationScheduledRequests (r:1 w:1)
    fn schedule_leave_delegators() -> Weight {
        Weight::from_ref_time(56_271_000)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    // Storage: ParachainStaking DelegatorState (r:1 w:1)
    // Storage: ParachainStaking Round (r:1 w:0)
    // Storage: ParachainStaking DelegationScheduledRequests (r:1 w:1)
    // Storage: ParachainStaking CandidateInfo (r:1 w:1)
    // Storage: ParachainStaking TopDelegations (r:1 w:1)
    // Storage: ParachainStaking CandidatePool (r:1 w:1)
    // Storage: ParachainStaking Total (r:1 w:1)
    // Storage: ParachainStaking AutoCompoundingDelegations (r:1 w:0)
    // Storage: Balances Locks (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    fn execute_leave_delegators(x: u32) -> Weight {
        Weight::from_ref_time(42_497_518)
            // Standard Error: 165_856
            .saturating_add(Weight::from_ref_time(35_902_041).saturating_mul(x.into()))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().reads((4_u64).saturating_mul(x.into())))
            .saturating_add(T::DbWeight::get().writes(2))
            .saturating_add(T::DbWeight::get().writes((3_u64).saturating_mul(x.into())))
    }
    // Storage: ParachainStaking DelegatorState (r:1 w:1)
    // Storage: ParachainStaking DelegationScheduledRequests (r:1 w:1)
    fn cancel_leave_delegators() -> Weight {
        Weight::from_ref_time(57_280_000)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    // Storage: ParachainStaking DelegatorState (r:1 w:1)
    // Storage: ParachainStaking DelegationScheduledRequests (r:1 w:1)
    // Storage: ParachainStaking Round (r:1 w:0)
    fn schedule_revoke_delegation() -> Weight {
        Weight::from_ref_time(68_190_000)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    // Storage: ParachainStaking DelegationScheduledRequests (r:1 w:0)
    // Storage: ParachainStaking DelegatorState (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    // Storage: Balances Locks (r:1 w:1)
    // Storage: ParachainStaking CandidateInfo (r:1 w:1)
    // Storage: ParachainStaking TopDelegations (r:1 w:1)
    // Storage: ParachainStaking CandidatePool (r:1 w:1)
    // Storage: ParachainStaking Total (r:1 w:1)
    fn delegator_bond_more() -> Weight {
        Weight::from_ref_time(118_230_000)
            .saturating_add(T::DbWeight::get().reads(8))
            .saturating_add(T::DbWeight::get().writes(7))
    }
    // Storage: ParachainStaking DelegatorState (r:1 w:1)
    // Storage: ParachainStaking DelegationScheduledRequests (r:1 w:1)
    // Storage: ParachainStaking Round (r:1 w:0)
    fn schedule_delegator_bond_less() -> Weight {
        Weight::from_ref_time(56_290_000)
            .saturating_add(T::DbWeight::get().reads(3))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    // Storage: ParachainStaking DelegatorState (r:1 w:1)
    // Storage: ParachainStaking DelegationScheduledRequests (r:1 w:1)
    // Storage: ParachainStaking Round (r:1 w:0)
    // Storage: Balances Locks (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    // Storage: ParachainStaking AutoCompoundingDelegations (r:1 w:0)
    // Storage: ParachainStaking CandidateInfo (r:1 w:1)
    // Storage: ParachainStaking TopDelegations (r:1 w:1)
    // Storage: ParachainStaking CandidatePool (r:1 w:1)
    // Storage: ParachainStaking Total (r:1 w:1)
    fn execute_revoke_delegation() -> Weight {
        Weight::from_ref_time(145_220_000)
            .saturating_add(T::DbWeight::get().reads(10))
            .saturating_add(T::DbWeight::get().writes(8))
    }
    // Storage: ParachainStaking DelegatorState (r:1 w:1)
    // Storage: ParachainStaking DelegationScheduledRequests (r:1 w:1)
    // Storage: ParachainStaking Round (r:1 w:0)
    // Storage: ParachainStaking CandidateInfo (r:1 w:1)
    // Storage: Balances Locks (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    // Storage: ParachainStaking TopDelegations (r:1 w:1)
    // Storage: ParachainStaking CandidatePool (r:1 w:1)
    // Storage: ParachainStaking Total (r:1 w:1)
    fn execute_delegator_bond_less() -> Weight {
        Weight::from_ref_time(126_790_000)
            .saturating_add(T::DbWeight::get().reads(9))
            .saturating_add(T::DbWeight::get().writes(8))
    }
    // Storage: ParachainStaking DelegatorState (r:1 w:1)
    // Storage: ParachainStaking DelegationScheduledRequests (r:1 w:1)
    fn cancel_revoke_delegation() -> Weight {
        Weight::from_ref_time(53_520_000)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    // Storage: ParachainStaking DelegatorState (r:1 w:1)
    // Storage: ParachainStaking DelegationScheduledRequests (r:1 w:1)
    fn cancel_delegator_bond_less() -> Weight {
        Weight::from_ref_time(62_760_000)
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(2))
    }
    // Storage: ParachainStaking Points (r:1 w:0)
    // Storage: ParachainStaking Staked (r:1 w:1)
    // Storage: ParachainStaking InflationConfig (r:1 w:0)
    // Storage: ParachainStaking ParachainBondInfo (r:1 w:0)
    // Storage: System Account (r:1 w:1)
    // Storage: ParachainStaking CollatorCommission (r:1 w:0)
    // Storage: ParachainStaking DelayedPayouts (r:0 w:1)
    fn prepare_staking_payouts() -> Weight {
        Weight::from_ref_time(75_670_000)
            .saturating_add(T::DbWeight::get().reads(6))
            .saturating_add(T::DbWeight::get().writes(3))
    }
    // Storage: ParachainStaking DelegationScheduledRequests (r:1 w:0)
    // Storage: ParachainStaking TopDelegations (r:1 w:0)
    fn get_rewardable_delegators(y: u32) -> Weight {
        Weight::from_ref_time(17_681_343)
            // Standard Error: 8_876
            .saturating_add(Weight::from_ref_time(502_117).saturating_mul(y.into()))
            .saturating_add(T::DbWeight::get().reads(2))
    }
    // Storage: ParachainStaking CandidatePool (r:1 w:0)
    // Storage: ParachainStaking TotalSelected (r:1 w:0)
    // Storage: ParachainStaking CandidateInfo (r:1 w:0)
    // Storage: ParachainStaking DelegationScheduledRequests (r:1 w:0)
    // Storage: ParachainStaking TopDelegations (r:1 w:0)
    // Storage: ParachainStaking AutoCompoundingDelegations (r:1 w:0)
    // Storage: ParachainStaking SelectedCandidates (r:0 w:1)
    // Storage: ParachainStaking AtStake (r:0 w:1)
    fn select_top_candidates(x: u32, y: u32) -> Weight {
        Weight::from_ref_time(49_680_000)
            // Standard Error: 1_177_642
            .saturating_add(Weight::from_ref_time(49_689_789).saturating_mul(x.into()))
            // Standard Error: 587_258
            .saturating_add(Weight::from_ref_time(12_322_486).saturating_mul(y.into()))
            .saturating_add(T::DbWeight::get().reads(6))
            .saturating_add(T::DbWeight::get().reads((4_u64).saturating_mul(x.into())))
            .saturating_add(T::DbWeight::get().writes(2))
            .saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(x.into())))
    }
    // Storage: ParachainStaking DelayedPayouts (r:1 w:0)
    // Storage: ParachainStaking Points (r:1 w:0)
    // Storage: ParachainStaking AtStake (r:2 w:1)
    // Storage: ParachainStaking AwardedPts (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    fn pay_one_collator_reward(y: u32) -> Weight {
        Weight::from_ref_time(106_371_824)
            // Standard Error: 93_457
            .saturating_add(Weight::from_ref_time(20_766_401).saturating_mul(y.into()))
            .saturating_add(T::DbWeight::get().reads(6))
            .saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(y.into())))
            .saturating_add(T::DbWeight::get().writes(3))
            .saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(y.into())))
    }
    // Storage: ParachainStaking Round (r:1 w:0)
    fn base_on_initialize() -> Weight {
        Weight::from_ref_time(16_070_000).saturating_add(T::DbWeight::get().reads(1))
    }
    // Storage: ParachainStaking DelegatorState (r:1 w:0)
    // Storage: ParachainStaking AutoCompoundingDelegations (r:1 w:1)
    fn set_auto_compound(x: u32, y: u32) -> Weight {
        Weight::from_ref_time(130_193_847)
            // Standard Error: 11_664
            .saturating_add(Weight::from_ref_time(641_058).saturating_mul(x.into()))
            // Standard Error: 34_919
            .saturating_add(Weight::from_ref_time(293_987).saturating_mul(y.into()))
            .saturating_add(T::DbWeight::get().reads(2))
            .saturating_add(T::DbWeight::get().writes(1))
    }
    // Storage: System Account (r:1 w:1)
    // Storage: ParachainStaking DelegatorState (r:1 w:1)
    // Storage: ParachainStaking CandidateInfo (r:1 w:1)
    // Storage: ParachainStaking AutoCompoundingDelegations (r:1 w:1)
    // Storage: ParachainStaking TopDelegations (r:1 w:1)
    // Storage: ParachainStaking CandidatePool (r:1 w:1)
    // Storage: Balances Locks (r:1 w:1)
    // Storage: ParachainStaking Total (r:1 w:1)
    // Storage: ParachainStaking BottomDelegations (r:1 w:1)
    fn delegate_with_auto_compound(x: u32, y: u32, _z: u32) -> Weight {
        Weight::from_ref_time(332_503_261)
            // Standard Error: 12_876
            .saturating_add(Weight::from_ref_time(301_412).saturating_mul(x.into()))
            // Standard Error: 12_876
            .saturating_add(Weight::from_ref_time(192_596).saturating_mul(y.into()))
            .saturating_add(T::DbWeight::get().reads(8))
            .saturating_add(T::DbWeight::get().writes(8))
    }
    // Storage: System Account (r:1 w:1)
    fn mint_collator_reward() -> Weight {
        Weight::from_ref_time(50_240_000)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }
}
