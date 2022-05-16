//! Autogenerated weights for parachain_staking
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-05-15, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/zeitgeist
// benchmark
// pallet
// --chain=dev
// --steps=50
// --repeat=20
// --pallet=parachain_staking
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --heap-pages=4096
// --template=./misc/frame_weight_template.hbs
// --output=./runtime/src/weights/

#![allow(unused_parens)]
#![allow(unused_imports)]

use core::marker::PhantomData;
use frame_support::{
    traits::Get,
    weights::{constants::RocksDbWeight, Weight},
};

/// Weight functions for parachain_staking (automatically generated)
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> parachain_staking::weights::WeightInfo for WeightInfo<T> {
<<<<<<< HEAD
    // Storage: ParachainStaking DelegatorState (r:1 w:1)
    fn hotfix_remove_delegation_requests(x: u32) -> Weight {
        (0 as Weight)
            // Standard Error: 67_000
            .saturating_add((13_612_000 as Weight).saturating_mul(x as Weight))
            .saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(x as Weight)))
            .saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(x as Weight)))
    }
    // Storage: ParachainStaking CandidateInfo (r:4 w:0)
    // Storage: ParachainStaking CandidatePool (r:1 w:1)
    fn hotfix_update_candidate_pool_value(x: u32) -> Weight {
        (0 as Weight)
            // Standard Error: 185_000
            .saturating_add((26_964_000 as Weight).saturating_mul(x as Weight))
            .saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(x as Weight)))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: ParachainStaking InflationConfig (r:1 w:1)
    fn set_staking_expectations() -> Weight {
        (28_460_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: ParachainStaking InflationConfig (r:1 w:1)
    // Storage: ParachainStaking Round (r:1 w:0)
    fn set_inflation() -> Weight {
        (129_320_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: ParachainStaking ParachainBondInfo (r:1 w:1)
    fn set_parachain_bond_account() -> Weight {
        (60_701_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: ParachainStaking ParachainBondInfo (r:1 w:1)
    fn set_parachain_bond_reserve_percent() -> Weight {
        (51_530_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: ParachainStaking TotalSelected (r:1 w:1)
    // Storage: ParachainStaking Round (r:1 w:0)
    fn set_total_selected() -> Weight {
        (66_160_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: ParachainStaking CollatorCommission (r:1 w:1)
    fn set_collator_commission() -> Weight {
        (49_040_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: ParachainStaking Round (r:1 w:1)
    // Storage: ParachainStaking TotalSelected (r:1 w:0)
    // Storage: ParachainStaking InflationConfig (r:1 w:1)
    fn set_blocks_per_round() -> Weight {
        (108_091_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: ParachainStaking CandidateInfo (r:1 w:1)
    // Storage: ParachainStaking DelegatorState (r:1 w:0)
    // Storage: ParachainStaking CandidatePool (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    // Storage: ParachainStaking Total (r:1 w:1)
    // Storage: ParachainStaking TopDelegations (r:0 w:1)
    // Storage: ParachainStaking BottomDelegations (r:0 w:1)
    fn join_candidates(x: u32) -> Weight {
        (106_999_000 as Weight)
            // Standard Error: 2_000
            .saturating_add((179_000 as Weight).saturating_mul(x as Weight))
            .saturating_add(T::DbWeight::get().reads(5 as Weight))
            .saturating_add(T::DbWeight::get().writes(6 as Weight))
    }
    // Storage: ParachainStaking CandidateInfo (r:1 w:1)
    // Storage: ParachainStaking Round (r:1 w:0)
    // Storage: ParachainStaking CandidatePool (r:1 w:1)
    fn schedule_leave_candidates(x: u32) -> Weight {
        (85_760_000 as Weight)
            // Standard Error: 2_000
            .saturating_add((155_000 as Weight).saturating_mul(x as Weight))
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: ParachainStaking CandidateInfo (r:1 w:1)
    // Storage: ParachainStaking Round (r:1 w:0)
    // Storage: ParachainStaking TopDelegations (r:1 w:1)
    // Storage: System Account (r:2 w:2)
    // Storage: ParachainStaking DelegatorState (r:1 w:1)
    // Storage: ParachainStaking BottomDelegations (r:1 w:1)
    // Storage: ParachainStaking Total (r:1 w:1)
    fn execute_leave_candidates(x: u32) -> Weight {
        (0 as Weight)
            // Standard Error: 181_000
            .saturating_add((41_971_000 as Weight).saturating_mul(x as Weight))
            .saturating_add(T::DbWeight::get().reads(4 as Weight))
            .saturating_add(T::DbWeight::get().reads((2 as Weight).saturating_mul(x as Weight)))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
            .saturating_add(T::DbWeight::get().writes((2 as Weight).saturating_mul(x as Weight)))
    }
    // Storage: ParachainStaking CandidateInfo (r:1 w:1)
    // Storage: ParachainStaking CandidatePool (r:1 w:1)
    fn cancel_leave_candidates(x: u32) -> Weight {
        (87_739_000 as Weight)
            // Standard Error: 2_000
            .saturating_add((157_000 as Weight).saturating_mul(x as Weight))
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: ParachainStaking CandidateInfo (r:1 w:1)
    // Storage: ParachainStaking CandidatePool (r:1 w:1)
    fn go_offline() -> Weight {
        (47_051_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: ParachainStaking CandidateInfo (r:1 w:1)
    // Storage: ParachainStaking CandidatePool (r:1 w:1)
    fn go_online() -> Weight {
        (49_990_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
    }
    // Storage: ParachainStaking CandidateInfo (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    // Storage: ParachainStaking Total (r:1 w:1)
    // Storage: ParachainStaking CandidatePool (r:1 w:1)
    fn candidate_bond_more() -> Weight {
        (88_871_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(4 as Weight))
            .saturating_add(T::DbWeight::get().writes(4 as Weight))
    }
    // Storage: ParachainStaking CandidateInfo (r:1 w:1)
    // Storage: ParachainStaking Round (r:1 w:0)
    fn schedule_candidate_bond_less() -> Weight {
        (44_290_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: ParachainStaking CandidateInfo (r:1 w:1)
    // Storage: ParachainStaking Round (r:1 w:0)
    // Storage: System Account (r:1 w:1)
    // Storage: ParachainStaking Total (r:1 w:1)
    // Storage: ParachainStaking CandidatePool (r:1 w:1)
    fn execute_candidate_bond_less() -> Weight {
        (111_520_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(5 as Weight))
            .saturating_add(T::DbWeight::get().writes(4 as Weight))
    }
    // Storage: ParachainStaking CandidateInfo (r:1 w:1)
    fn cancel_candidate_bond_less() -> Weight {
        (45_320_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: System Account (r:1 w:1)
    // Storage: ParachainStaking DelegatorState (r:1 w:1)
    // Storage: ParachainStaking CandidateInfo (r:1 w:1)
    // Storage: ParachainStaking TopDelegations (r:1 w:1)
    // Storage: ParachainStaking CandidatePool (r:1 w:1)
    // Storage: ParachainStaking Total (r:1 w:1)
    fn delegate(x: u32, y: u32) -> Weight {
        (176_016_000 as Weight)
            // Standard Error: 20_000
            .saturating_add((112_000 as Weight).saturating_mul(x as Weight))
            // Standard Error: 7_000
            .saturating_add((291_000 as Weight).saturating_mul(y as Weight))
            .saturating_add(T::DbWeight::get().reads(6 as Weight))
            .saturating_add(T::DbWeight::get().writes(6 as Weight))
    }
    // Storage: ParachainStaking DelegatorState (r:1 w:1)
    // Storage: ParachainStaking Round (r:1 w:0)
    fn schedule_leave_delegators() -> Weight {
        (46_830_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: ParachainStaking DelegatorState (r:1 w:1)
    // Storage: ParachainStaking Round (r:1 w:0)
    // Storage: ParachainStaking CandidateInfo (r:1 w:1)
    // Storage: ParachainStaking TopDelegations (r:1 w:1)
    // Storage: ParachainStaking CandidatePool (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    // Storage: ParachainStaking Total (r:1 w:1)
    fn execute_leave_delegators(x: u32) -> Weight {
        (0 as Weight)
            // Standard Error: 231_000
            .saturating_add((66_549_000 as Weight).saturating_mul(x as Weight))
            .saturating_add(T::DbWeight::get().reads(3 as Weight))
            .saturating_add(T::DbWeight::get().reads((2 as Weight).saturating_mul(x as Weight)))
            .saturating_add(T::DbWeight::get().writes(2 as Weight))
            .saturating_add(T::DbWeight::get().writes((2 as Weight).saturating_mul(x as Weight)))
    }
    // Storage: ParachainStaking DelegatorState (r:1 w:1)
    fn cancel_leave_delegators() -> Weight {
        (40_640_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: ParachainStaking DelegatorState (r:1 w:1)
    // Storage: ParachainStaking Round (r:1 w:0)
    fn schedule_revoke_delegation() -> Weight {
        (49_160_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: ParachainStaking DelegatorState (r:1 w:1)
    // Storage: ParachainStaking CandidateInfo (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    // Storage: ParachainStaking TopDelegations (r:1 w:1)
    // Storage: ParachainStaking CandidatePool (r:1 w:1)
    // Storage: ParachainStaking Total (r:1 w:1)
    fn delegator_bond_more() -> Weight {
        (111_730_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(6 as Weight))
            .saturating_add(T::DbWeight::get().writes(6 as Weight))
    }
    // Storage: ParachainStaking DelegatorState (r:1 w:1)
    // Storage: ParachainStaking Round (r:1 w:0)
    fn schedule_delegator_bond_less() -> Weight {
        (48_470_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(2 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: ParachainStaking DelegatorState (r:1 w:1)
    // Storage: ParachainStaking Round (r:1 w:0)
    // Storage: ParachainStaking CandidateInfo (r:1 w:1)
    // Storage: ParachainStaking TopDelegations (r:1 w:1)
    // Storage: ParachainStaking CandidatePool (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    // Storage: ParachainStaking Total (r:1 w:1)
    fn execute_revoke_delegation() -> Weight {
        (136_670_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(7 as Weight))
            .saturating_add(T::DbWeight::get().writes(6 as Weight))
    }
    // Storage: ParachainStaking DelegatorState (r:1 w:1)
    // Storage: ParachainStaking Round (r:1 w:0)
    // Storage: ParachainStaking CandidateInfo (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    // Storage: ParachainStaking TopDelegations (r:1 w:1)
    // Storage: ParachainStaking CandidatePool (r:1 w:1)
    // Storage: ParachainStaking Total (r:1 w:1)
    fn execute_delegator_bond_less() -> Weight {
        (123_890_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(7 as Weight))
            .saturating_add(T::DbWeight::get().writes(6 as Weight))
    }
    // Storage: ParachainStaking DelegatorState (r:1 w:1)
    fn cancel_revoke_delegation() -> Weight {
        (42_580_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
    // Storage: ParachainStaking DelegatorState (r:1 w:1)
    fn cancel_delegator_bond_less() -> Weight {
        (51_820_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(1 as Weight))
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
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
    // Storage: ParachainStaking TopDelegations (r:9 w:0)
    // Storage: ParachainStaking Total (r:1 w:0)
    // Storage: ParachainStaking AwardedPts (r:2 w:1)
    // Storage: ParachainStaking AtStake (r:1 w:10)
    // Storage: ParachainStaking SelectedCandidates (r:0 w:1)
    // Storage: ParachainStaking DelayedPayouts (r:0 w:1)
    fn round_transition_on_initialize(x: u32, y: u32) -> Weight {
        (0 as Weight)
            // Standard Error: 1_969_000
            .saturating_add((67_958_000 as Weight).saturating_mul(x as Weight))
            // Standard Error: 6_000
            .saturating_add((589_000 as Weight).saturating_mul(y as Weight))
            .saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(x as Weight)))
    }
    // Storage: ParachainStaking DelayedPayouts (r:1 w:0)
    // Storage: ParachainStaking Points (r:1 w:0)
    // Storage: ParachainStaking AwardedPts (r:2 w:1)
    // Storage: ParachainStaking AtStake (r:1 w:1)
    // Storage: System Account (r:1 w:1)
    fn pay_one_collator_reward(y: u32) -> Weight {
        (100_788_000 as Weight)
            // Standard Error: 149_000
            .saturating_add((33_380_000 as Weight).saturating_mul(y as Weight))
            .saturating_add(T::DbWeight::get().reads(6 as Weight))
            .saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(y as Weight)))
            .saturating_add(T::DbWeight::get().writes(3 as Weight))
            .saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(y as Weight)))
    }
    // Storage: ParachainStaking Round (r:1 w:0)
    fn base_on_initialize() -> Weight {
        (8_240_000 as Weight).saturating_add(T::DbWeight::get().reads(1 as Weight))
    }
=======
	// Storage: ParachainStaking DelegatorState (r:1 w:1)
	#[rustfmt::skip]
	fn hotfix_remove_delegation_requests(x: u32, ) -> Weight {
		(0 as Weight)
			// Standard Error: 5_000
			.saturating_add((7_270_000 as Weight).saturating_mul(x as Weight))
			.saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(x as Weight)))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(x as Weight)))
	}
	// Storage: ParachainStaking CandidateInfo (r:4 w:0)
	// Storage: ParachainStaking CandidatePool (r:1 w:1)
	#[rustfmt::skip]
	fn hotfix_update_candidate_pool_value(x: u32, ) -> Weight {
		(0 as Weight)
			// Standard Error: 174_000
			.saturating_add((24_976_000 as Weight).saturating_mul(x as Weight))
			.saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(x as Weight)))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: ParachainStaking InflationConfig (r:1 w:1)
	#[rustfmt::skip]
	fn set_staking_expectations() -> Weight {
		(19_066_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: ParachainStaking InflationConfig (r:1 w:1)
	// Storage: ParachainStaking Round (r:1 w:0)
	#[rustfmt::skip]
	fn set_inflation() -> Weight {
		(55_185_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: ParachainStaking ParachainBondInfo (r:1 w:1)
	#[rustfmt::skip]
	fn set_parachain_bond_account() -> Weight {
		(18_702_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: ParachainStaking ParachainBondInfo (r:1 w:1)
	#[rustfmt::skip]
	fn set_parachain_bond_reserve_percent() -> Weight {
		(18_068_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: ParachainStaking TotalSelected (r:1 w:1)
	// Storage: ParachainStaking Round (r:1 w:0)
	#[rustfmt::skip]
	fn set_total_selected() -> Weight {
		(20_989_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: ParachainStaking CollatorCommission (r:1 w:1)
	#[rustfmt::skip]
	fn set_collator_commission() -> Weight {
		(17_218_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: ParachainStaking Round (r:1 w:1)
	// Storage: ParachainStaking TotalSelected (r:1 w:0)
	// Storage: ParachainStaking InflationConfig (r:1 w:1)
	#[rustfmt::skip]
	fn set_blocks_per_round() -> Weight {
		(60_675_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: ParachainStaking CandidateInfo (r:1 w:1)
	// Storage: ParachainStaking DelegatorState (r:1 w:0)
	// Storage: ParachainStaking CandidatePool (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	// Storage: ParachainStaking Total (r:1 w:1)
	// Storage: ParachainStaking TopDelegations (r:0 w:1)
	// Storage: ParachainStaking BottomDelegations (r:0 w:1)
	#[rustfmt::skip]
	fn join_candidates(x: u32, ) -> Weight {
		(65_336_000 as Weight)
			// Standard Error: 1_000
			.saturating_add((117_000 as Weight).saturating_mul(x as Weight))
			.saturating_add(T::DbWeight::get().reads(5 as Weight))
			.saturating_add(T::DbWeight::get().writes(6 as Weight))
	}
	// Storage: ParachainStaking CandidateInfo (r:1 w:1)
	// Storage: ParachainStaking Round (r:1 w:0)
	// Storage: ParachainStaking CandidatePool (r:1 w:1)
	#[rustfmt::skip]
	fn schedule_leave_candidates(x: u32, ) -> Weight {
		(53_788_000 as Weight)
			// Standard Error: 1_000
			.saturating_add((102_000 as Weight).saturating_mul(x as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: ParachainStaking CandidateInfo (r:1 w:1)
	// Storage: ParachainStaking Round (r:1 w:0)
	// Storage: ParachainStaking TopDelegations (r:1 w:1)
	// Storage: System Account (r:2 w:2)
	// Storage: ParachainStaking DelegatorState (r:1 w:1)
	// Storage: ParachainStaking BottomDelegations (r:1 w:1)
	// Storage: ParachainStaking Total (r:1 w:1)
	#[rustfmt::skip]
	fn execute_leave_candidates(x: u32, ) -> Weight {
		(11_502_000 as Weight)
			// Standard Error: 10_000
			.saturating_add((21_626_000 as Weight).saturating_mul(x as Weight))
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().reads((2 as Weight).saturating_mul(x as Weight)))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
			.saturating_add(T::DbWeight::get().writes((2 as Weight).saturating_mul(x as Weight)))
	}
	// Storage: ParachainStaking CandidateInfo (r:1 w:1)
	// Storage: ParachainStaking CandidatePool (r:1 w:1)
	#[rustfmt::skip]
	fn cancel_leave_candidates(x: u32, ) -> Weight {
		(60_402_000 as Weight)
			// Standard Error: 2_000
			.saturating_add((90_000 as Weight).saturating_mul(x as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: ParachainStaking CandidateInfo (r:1 w:1)
	// Storage: ParachainStaking CandidatePool (r:1 w:1)
	#[rustfmt::skip]
	fn go_offline() -> Weight {
		(28_307_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: ParachainStaking CandidateInfo (r:1 w:1)
	// Storage: ParachainStaking CandidatePool (r:1 w:1)
	#[rustfmt::skip]
	fn go_online() -> Weight {
		(28_102_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: ParachainStaking CandidateInfo (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	// Storage: ParachainStaking Total (r:1 w:1)
	// Storage: ParachainStaking CandidatePool (r:1 w:1)
	#[rustfmt::skip]
	fn candidate_bond_more() -> Weight {
		(46_424_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
	}
	// Storage: ParachainStaking CandidateInfo (r:1 w:1)
	// Storage: ParachainStaking Round (r:1 w:0)
	#[rustfmt::skip]
	fn schedule_candidate_bond_less() -> Weight {
		(26_422_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: ParachainStaking CandidateInfo (r:1 w:1)
	// Storage: ParachainStaking Round (r:1 w:0)
	// Storage: System Account (r:1 w:1)
	// Storage: ParachainStaking Total (r:1 w:1)
	// Storage: ParachainStaking CandidatePool (r:1 w:1)
	#[rustfmt::skip]
	fn execute_candidate_bond_less() -> Weight {
		(55_036_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(5 as Weight))
			.saturating_add(T::DbWeight::get().writes(4 as Weight))
	}
	// Storage: ParachainStaking CandidateInfo (r:1 w:1)
	#[rustfmt::skip]
	fn cancel_candidate_bond_less() -> Weight {
		(23_130_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: System Account (r:1 w:1)
	// Storage: ParachainStaking DelegatorState (r:1 w:1)
	// Storage: ParachainStaking CandidateInfo (r:1 w:1)
	// Storage: ParachainStaking TopDelegations (r:1 w:1)
	// Storage: ParachainStaking CandidatePool (r:1 w:1)
	// Storage: ParachainStaking Total (r:1 w:1)
	#[rustfmt::skip]
	fn delegate(x: u32, y: u32, ) -> Weight {
		(90_567_000 as Weight)
			// Standard Error: 3_000
			.saturating_add((310_000 as Weight).saturating_mul(x as Weight))
			// Standard Error: 1_000
			.saturating_add((204_000 as Weight).saturating_mul(y as Weight))
			.saturating_add(T::DbWeight::get().reads(6 as Weight))
			.saturating_add(T::DbWeight::get().writes(6 as Weight))
	}
	// Storage: ParachainStaking DelegatorState (r:1 w:1)
	// Storage: ParachainStaking Round (r:1 w:0)
	#[rustfmt::skip]
	fn schedule_leave_delegators() -> Weight {
		(27_353_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: ParachainStaking DelegatorState (r:1 w:1)
	// Storage: ParachainStaking Round (r:1 w:0)
	// Storage: ParachainStaking CandidateInfo (r:1 w:1)
	// Storage: ParachainStaking TopDelegations (r:1 w:1)
	// Storage: ParachainStaking CandidatePool (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	// Storage: ParachainStaking Total (r:1 w:1)
	#[rustfmt::skip]
	fn execute_leave_delegators(x: u32, ) -> Weight {
		(0 as Weight)
			// Standard Error: 76_000
			.saturating_add((39_542_000 as Weight).saturating_mul(x as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().reads((2 as Weight).saturating_mul(x as Weight)))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
			.saturating_add(T::DbWeight::get().writes((2 as Weight).saturating_mul(x as Weight)))
	}
	// Storage: ParachainStaking DelegatorState (r:1 w:1)
	#[rustfmt::skip]
	fn cancel_leave_delegators() -> Weight {
		(24_218_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: ParachainStaking DelegatorState (r:1 w:1)
	// Storage: ParachainStaking Round (r:1 w:0)
	#[rustfmt::skip]
	fn schedule_revoke_delegation() -> Weight {
		(28_053_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: ParachainStaking DelegatorState (r:1 w:1)
	// Storage: ParachainStaking CandidateInfo (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	// Storage: ParachainStaking TopDelegations (r:1 w:1)
	// Storage: ParachainStaking CandidatePool (r:1 w:1)
	// Storage: ParachainStaking Total (r:1 w:1)
	#[rustfmt::skip]
	fn delegator_bond_more() -> Weight {
		(64_504_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(6 as Weight))
			.saturating_add(T::DbWeight::get().writes(6 as Weight))
	}
	// Storage: ParachainStaking DelegatorState (r:1 w:1)
	// Storage: ParachainStaking Round (r:1 w:0)
	#[rustfmt::skip]
	fn schedule_delegator_bond_less() -> Weight {
		(29_017_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: ParachainStaking DelegatorState (r:1 w:1)
	// Storage: ParachainStaking Round (r:1 w:0)
	// Storage: ParachainStaking CandidateInfo (r:1 w:1)
	// Storage: ParachainStaking TopDelegations (r:1 w:1)
	// Storage: ParachainStaking CandidatePool (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	// Storage: ParachainStaking Total (r:1 w:1)
	#[rustfmt::skip]
	fn execute_revoke_delegation() -> Weight {
		(78_531_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(7 as Weight))
			.saturating_add(T::DbWeight::get().writes(6 as Weight))
	}
	// Storage: ParachainStaking DelegatorState (r:1 w:1)
	// Storage: ParachainStaking Round (r:1 w:0)
	// Storage: ParachainStaking CandidateInfo (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	// Storage: ParachainStaking TopDelegations (r:1 w:1)
	// Storage: ParachainStaking CandidatePool (r:1 w:1)
	// Storage: ParachainStaking Total (r:1 w:1)
	#[rustfmt::skip]
	fn execute_delegator_bond_less() -> Weight {
		(71_044_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(7 as Weight))
			.saturating_add(T::DbWeight::get().writes(6 as Weight))
	}
	// Storage: ParachainStaking DelegatorState (r:1 w:1)
	#[rustfmt::skip]
	fn cancel_revoke_delegation() -> Weight {
		(25_288_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: ParachainStaking DelegatorState (r:1 w:1)
	#[rustfmt::skip]
	fn cancel_delegator_bond_less() -> Weight {
		(30_562_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
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
	// Storage: ParachainStaking TopDelegations (r:9 w:0)
	// Storage: ParachainStaking Total (r:1 w:0)
	// Storage: ParachainStaking AwardedPts (r:2 w:1)
	// Storage: ParachainStaking AtStake (r:1 w:10)
	// Storage: ParachainStaking SelectedCandidates (r:0 w:1)
	// Storage: ParachainStaking DelayedPayouts (r:0 w:1)
	#[rustfmt::skip]
	fn round_transition_on_initialize(x: u32, y: u32, ) -> Weight {
		(0 as Weight)
			// Standard Error: 827_000
			.saturating_add((65_228_000 as Weight).saturating_mul(x as Weight))
			// Standard Error: 2_000
			.saturating_add((208_000 as Weight).saturating_mul(y as Weight))
			.saturating_add(T::DbWeight::get().reads(194 as Weight))
			.saturating_add(T::DbWeight::get().reads((2 as Weight).saturating_mul(x as Weight)))
			.saturating_add(T::DbWeight::get().writes(188 as Weight))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(x as Weight)))
	}
	// Storage: ParachainStaking DelayedPayouts (r:1 w:0)
	// Storage: ParachainStaking Points (r:1 w:0)
	// Storage: ParachainStaking AwardedPts (r:2 w:1)
	// Storage: ParachainStaking AtStake (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	#[rustfmt::skip]
	fn pay_one_collator_reward(y: u32, ) -> Weight {
		(43_761_000 as Weight)
			// Standard Error: 10_000
			.saturating_add((17_111_000 as Weight).saturating_mul(y as Weight))
			.saturating_add(T::DbWeight::get().reads(6 as Weight))
			.saturating_add(T::DbWeight::get().reads((1 as Weight).saturating_mul(y as Weight)))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
			.saturating_add(T::DbWeight::get().writes((1 as Weight).saturating_mul(y as Weight)))
	}
	// Storage: ParachainStaking Round (r:1 w:0)
	#[rustfmt::skip]
	fn base_on_initialize() -> Weight {
		(4_892_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
	}
>>>>>>> main
}
