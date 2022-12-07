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
//! DATE: 2022-11-25, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
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
        (30_011_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: ParachainStaking InflationConfig (r:1 w:1)
    // Storage: ParachainStaking Round (r:1 w:0)
    fn set_inflation() -> Weight {
        (86_350_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: ParachainStaking ParachainBondInfo (r:1 w:1)
    fn set_parachain_bond_account() -> Weight {
        (29_470_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: ParachainStaking ParachainBondInfo (r:1 w:1)
    fn set_parachain_bond_reserve_percent() -> Weight {
        (31_920_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: ParachainStaking TotalSelected (r:1 w:1)
    // Storage: ParachainStaking Round (r:1 w:0)
    fn set_total_selected() -> Weight {
        (32_160_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: ParachainStaking CollatorCommission (r:1 w:1)
    fn set_collator_commission() -> Weight {
        (31_010_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: ParachainStaking Round (r:1 w:1)
    // Storage: ParachainStaking TotalSelected (r:1 w:0)
    // Storage: ParachainStaking InflationConfig (r:1 w:1)
    fn set_blocks_per_round() -> Weight {
        (109_270_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
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
        (176_454_000 as Weight)
            // Standard Error: 4_000
            .saturating_add((181_000 as Weight).saturating_mul(x as Weight))
            .saturating_add(T::DbWeight::get().reads(6 as Weight))
            .saturating_add(T::DbWeight::get().writes(7 as Weight))
    }
    // Storage: ParachainStaking CandidateInfo (r:1 w:1)
    // Storage: ParachainStaking Round (r:1 w:0)
    // Storage: ParachainStaking CandidatePool (r:1 w:1)
    fn schedule_leave_candidates(x: u32) -> Weight {
        (142_707_000 as Weight)
            // Standard Error: 5_000
            .saturating_add((164_000 as Weight).saturating_mul(x as Weight))
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: ParachainStaking CandidateInfo (r:1 w:1)
    // Storage: ParachainStaking Round (r:1 w:0)
    // Storage: ParachainStaking TopDelegations (r:1 w:1)
    // Storage: ParachainStaking DelegatorState (r:1 w:1)
    // Storage: Balances Locks (r:2 w:2)
    // Storage: System Account (r:2 w:2)
    // Storage: ParachainStaking DelegationScheduledRequests (r:1 w:1)
    // Storage: ParachainStaking BottomDelegations (r:1 w:1)
    // Storage: ParachainStaking Total (r:1 w:1)
    fn execute_leave_candidates(x: u32) -> Weight {
        (0 as Weight)
            // Standard Error: 231_000
            .saturating_add((64_246_000 as Weight).saturating_mul(x as Weight))
            .saturating_add(T::DbWeight::get().reads(5 as Weight))
            .saturating_add(T::DbWeight::get().reads((3 as Weight).saturating_mul(x as Weight)))
            .saturating_add(T::DbWeight::get().writes(4 as Weight))
            .saturating_add(T::DbWeight::get().writes((3 as Weight).saturating_mul(x as Weight)))
    }
    // Storage: ParachainStaking CandidateInfo (r:1 w:1)
    // Storage: ParachainStaking CandidatePool (r:1 w:1)
    fn cancel_leave_candidates(x: u32) -> Weight {
        (140_788_000 as Weight)
            // Standard Error: 4_000
            .saturating_add((168_000 as Weight).saturating_mul(x as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: ParachainStaking CandidateInfo (r:1 w:1)
    // Storage: ParachainStaking CandidatePool (r:1 w:1)
    fn go_offline() -> Weight {
        (42_920_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: ParachainStaking CandidateInfo (r:1 w:1)
    // Storage: ParachainStaking CandidatePool (r:1 w:1)
    fn go_online() -> Weight {
        (41_760_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: ParachainStaking CandidateInfo (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    // Storage: ParachainStaking Total (r:1 w:1)
    // Storage: Balances Locks (r:1 w:1)
    // Storage: ParachainStaking CandidatePool (r:1 w:1)
    fn candidate_bond_more() -> Weight {
        (70_700_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(5 as Weight))
            .saturating_add(T::DbWeight::get().writes(5 as Weight))
    }
    // Storage: ParachainStaking CandidateInfo (r:1 w:1)
    // Storage: ParachainStaking Round (r:1 w:0)
    fn schedule_candidate_bond_less() -> Weight {
        (40_220_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: ParachainStaking CandidateInfo (r:1 w:1)
    // Storage: ParachainStaking Round (r:1 w:0)
    // Storage: ParachainStaking Total (r:1 w:1)
    // Storage: Balances Locks (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    // Storage: ParachainStaking CandidatePool (r:1 w:1)
    fn execute_candidate_bond_less() -> Weight {
        (72_820_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(6 as Weight))
            .saturating_add(T::DbWeight::get().writes(5 as Weight))
    }
    // Storage: ParachainStaking CandidateInfo (r:1 w:1)
    fn cancel_candidate_bond_less() -> Weight {
        (28_360_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: System Account (r:1 w:1)
    // Storage: ParachainStaking DelegatorState (r:1 w:1)
    // Storage: ParachainStaking CandidateInfo (r:1 w:1)
    // Storage: ParachainStaking TopDelegations (r:1 w:1)
    // Storage: ParachainStaking CandidatePool (r:1 w:1)
    // Storage: Balances Locks (r:1 w:1)
    // Storage: ParachainStaking Total (r:1 w:1)
    fn delegate(x: u32, y: u32) -> Weight {
        (275_442_000 as Weight)
            // Standard Error: 33_000
            .saturating_add((59_000 as Weight).saturating_mul(x as Weight))
            // Standard Error: 11_000
            .saturating_add((281_000 as Weight).saturating_mul(y as Weight))
            .saturating_add(T::DbWeight::get().reads(7 as Weight))
            .saturating_add(T::DbWeight::get().writes(7 as Weight))
    }
    // Storage: ParachainStaking DelegatorState (r:1 w:1)
    // Storage: ParachainStaking Round (r:1 w:0)
    // Storage: ParachainStaking DelegationScheduledRequests (r:1 w:1)
    fn schedule_leave_delegators() -> Weight {
        (50_240_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: ParachainStaking DelegatorState (r:1 w:1)
    // Storage: ParachainStaking Round (r:1 w:0)
    // Storage: ParachainStaking DelegationScheduledRequests (r:1 w:1)
    // Storage: ParachainStaking CandidateInfo (r:1 w:1)
    // Storage: ParachainStaking TopDelegations (r:1 w:1)
    // Storage: ParachainStaking CandidatePool (r:1 w:1)
    // Storage: ParachainStaking Total (r:1 w:1)
    // Storage: Balances Locks (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    fn execute_leave_delegators(x: u32) -> Weight {
        (0 as Weight)
            // Standard Error: 147_000
            .saturating_add((50_946_000 as Weight).saturating_mul(x as Weight))
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().reads((3 as Weight).saturating_mul(x as Weight)))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
            .saturating_add(T::DbWeight::get().writes((3 as Weight).saturating_mul(x as Weight)))
    }
    // Storage: ParachainStaking DelegatorState (r:1 w:1)
    // Storage: ParachainStaking DelegationScheduledRequests (r:1 w:1)
    fn cancel_leave_delegators() -> Weight {
        (51_670_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: ParachainStaking DelegatorState (r:1 w:1)
    // Storage: ParachainStaking DelegationScheduledRequests (r:1 w:1)
    // Storage: ParachainStaking Round (r:1 w:0)
    fn schedule_revoke_delegation() -> Weight {
        (57_460_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
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
        (99_120_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(8 as Weight))
            .saturating_add(T::DbWeight::get().writes(7 as Weight))
    }
    // Storage: ParachainStaking DelegatorState (r:1 w:1)
    // Storage: ParachainStaking DelegationScheduledRequests (r:1 w:1)
    // Storage: ParachainStaking Round (r:1 w:0)
    fn schedule_delegator_bond_less() -> Weight {
        (59_450_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: ParachainStaking DelegatorState (r:1 w:1)
    // Storage: ParachainStaking DelegationScheduledRequests (r:1 w:1)
    // Storage: ParachainStaking Round (r:1 w:0)
    // Storage: Balances Locks (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    // Storage: ParachainStaking CandidateInfo (r:1 w:1)
    // Storage: ParachainStaking TopDelegations (r:1 w:1)
    // Storage: ParachainStaking CandidatePool (r:1 w:1)
    // Storage: ParachainStaking Total (r:1 w:1)
    fn execute_revoke_delegation() -> Weight {
        (120_970_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(9 as Weight))
            .saturating_add(T::DbWeight::get().writes(8 as Weight))
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
        (110_711_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(9 as Weight))
            .saturating_add(T::DbWeight::get().writes(8 as Weight))
    }
    // Storage: ParachainStaking DelegatorState (r:1 w:1)
    // Storage: ParachainStaking DelegationScheduledRequests (r:1 w:1)
    fn cancel_revoke_delegation() -> Weight {
        (43_980_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: ParachainStaking DelegatorState (r:1 w:1)
    // Storage: ParachainStaking DelegationScheduledRequests (r:1 w:1)
    fn cancel_delegator_bond_less() -> Weight {
        (52_200_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: ParachainStaking Round (r:1 w:1)
    // Storage: ParachainStaking Points (r:1 w:0)
    // Storage: ParachainStaking Staked (r:1 w:2)
    // Storage: ParachainStaking InflationConfig (r:1 w:0)
    // Storage: ParachainStaking ParachainBondInfo (r:1 w:0)
    // Storage: System Account (r:302 w:301)
    // Storage: ParachainStaking CollatorCommission (r:1 w:0)
    // Storage: ParachainStaking CandidatePool (r:1 w:0)
    // Storage: ParachainStaking TotalSelected (r:1 w:0)
    // Storage: ParachainStaking CandidateInfo (r:9 w:0)
    // Storage: ParachainStaking DelegationScheduledRequests (r:9 w:0)
    // Storage: ParachainStaking TopDelegations (r:9 w:0)
    // Storage: ParachainStaking Total (r:1 w:0)
    // Storage: ParachainStaking AwardedPts (r:2 w:1)
    // Storage: ParachainStaking AtStake (r:1 w:10)
    // Storage: ParachainStaking SelectedCandidates (r:0 w:1)
    // Storage: ParachainStaking DelayedPayouts (r:0 w:1)
    fn round_transition_on_initialize(x: u32, y: u32) -> Weight {
        (0 as Weight)
            // Standard Error: 1_511_000
            .saturating_add((90_015_000 as Weight).saturating_mul(x as Weight))
            // Standard Error: 5_000
            .saturating_add((550_000 as Weight).saturating_mul(y as Weight))
            .saturating_add(T::DbWeight::get().reads((2 as Weight).saturating_mul(x as Weight)))
    }
    // Storage: ParachainStaking DelayedPayouts (r:1 w:0)
    // Storage: ParachainStaking Points (r:1 w:0)
    // Storage: ParachainStaking AwardedPts (r:2 w:1)
    // Storage: ParachainStaking AtStake (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    fn pay_one_collator_reward(y: u32) -> Weight {
        (8_729_000 as Weight)
            // Standard Error: 53_000
            .saturating_add((24_974_000 as Weight).saturating_mul(y as Weight))
            .saturating_add(T::DbWeight::get().reads(6 as Weight))
            .saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(y as Weight)))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
            .saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(y as Weight)))
    }
    // Storage: ParachainStaking Round (r:1 w:0)
    fn base_on_initialize() -> Weight {
        (7_010_000 as Weight).saturating_add(T::DbWeight::get().reads(1 as Weight))
    }
}
