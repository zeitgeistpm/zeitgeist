// Copyright 2022 Forecasting Technologies LTD.
// Copyright 2022 Zeitgeist PM LLC.
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

//! Global Disputes pallet benchmarking.

#![allow(
    // Auto-generated code is a no man's land
    clippy::integer_arithmetic
)]
#![cfg(feature = "runtime-benchmarks")]

use crate::{
    global_disputes_pallet_api::GlobalDisputesPalletApi, types::*, BalanceOf, Call, Config,
    Pallet as GlobalDisputes, *,
};
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::{
    sp_runtime::traits::StaticLookup,
    traits::{Currency, Get},
    BoundedVec,
};
use frame_system::RawOrigin;
use num_traits::ops::checked::CheckedRem;
use sp_runtime::traits::{Bounded, SaturatedConversion, Saturating};
use sp_std::prelude::*;
use zeitgeist_primitives::types::OutcomeReport;

fn deposit<T>(caller: &T::AccountId)
where
    T: Config,
{
    let _ =
        T::Currency::deposit_creating(caller, BalanceOf::<T>::max_value() / 2u128.saturated_into());
}

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
    frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

benchmarks! {
    vote_on_outcome {
        // only Outcomes owners, but not Winners owners is present during vote_on_outcome
        let o in 1..T::MaxOwners::get();

        // ensure we have one vote left for the call
        let v in 0..(T::MaxGlobalDisputeVotes::get() - 1);

        let caller: T::AccountId = whitelisted_caller();
        // ensure that we get the worst case
        // to actually insert the new item at the end of the binary search
        let market_id: MarketIdOf<T> = v.into();
        let outcome = OutcomeReport::Scalar(0);
        let amount: BalanceOf<T> = T::MinOutcomeVoteAmount::get().saturated_into();
        deposit::<T>(&caller);
        for i in 1..=o {
            let owner = account("outcomes_owner", i, 0);
            GlobalDisputes::<T>::push_voting_outcome(
                &market_id,
                outcome.clone(),
                &owner,
                1_000_000_000u128.saturated_into(),
            )
            .unwrap();
        }

        let mut vote_locks: BoundedVec<(
            MarketIdOf<T>,
            BalanceOf<T>
        ), T::MaxGlobalDisputeVotes> = Default::default();
        for i in 0..v {
            let market_id: MarketIdOf<T> = i.saturated_into();
            let locked_balance: BalanceOf<T> = T::MinOutcomeVoteAmount::get().saturated_into();
            vote_locks.try_push((market_id, locked_balance)).unwrap();
        }
        <Locks<T>>::insert(caller.clone(), vote_locks);

        // minus one to ensure, that we use the worst case
        // for using a new winner info after the vote_on_outcome call
        let vote_sum = amount - 1u128.saturated_into();
        let outcome_info = OutcomeInfo { outcome_sum: vote_sum, owners: Default::default() };
        let winner_info = WinnerInfo {outcome: outcome.clone(), is_finished: false, outcome_info};
        <Winners<T>>::insert(market_id, winner_info);
    }: _(RawOrigin::Signed(caller.clone()), market_id, outcome.clone(), amount)
    verify {
        assert_last_event::<T>(
            Event::VotedOnOutcome::<T> {
                market_id,
                voter: caller,
                outcome,
                vote_amount: amount,
            }
            .into(),
        );
    }

    unlock_vote_balance_set {
        let l in 0..T::MaxGlobalDisputeVotes::get();
        let o in 1..T::MaxOwners::get();

        let vote_sum = 42u128.saturated_into();
        let mut owners = Vec::new();
        for i in 1..=o {
            let owner = account("winners_owner", i, 0);
            owners.push(owner);
        }
        let owners = BoundedVec::try_from(owners).unwrap();
        let outcome = OutcomeReport::Scalar(0);
        let outcome_info = OutcomeInfo { outcome_sum: vote_sum, owners };
        // is_finished is false,
        // because we need `lock_needed` to be greater zero to set a lock.
        let winner_info = WinnerInfo {outcome, is_finished: false, outcome_info};

        let caller: T::AccountId = whitelisted_caller();
        let voter: T::AccountId = account("voter", 0, 0);
        let voter_lookup = T::Lookup::unlookup(voter.clone());
        let mut vote_locks: BoundedVec<(MarketIdOf<T>, BalanceOf<T>), T::MaxGlobalDisputeVotes> =
            Default::default();
        for i in 0..l {
            let market_id: MarketIdOf<T> = i.saturated_into();
            let locked_balance: BalanceOf<T> = i.saturated_into();
            vote_locks.try_push((market_id, locked_balance)).unwrap();
            <Winners<T>>::insert(market_id, winner_info.clone());
        }
        <Locks<T>>::insert(voter.clone(), vote_locks.clone());
    }: {
        <Pallet<T>>::unlock_vote_balance(
            RawOrigin::Signed(caller.clone()).into(),
            voter_lookup,
        )
        .unwrap();
    } verify {
        let lock_info = <Locks<T>>::get(&voter);
        assert_eq!(lock_info, vote_locks);
    }

    unlock_vote_balance_remove {
        let l in 0..T::MaxGlobalDisputeVotes::get();
        let o in 1..T::MaxOwners::get();

        let vote_sum = 42u128.saturated_into();
        let mut owners = Vec::new();
        for i in 1..=o {
            let owner = account("winners_owner", i, 0);
            owners.push(owner);
        }
        let owners = BoundedVec::try_from(owners).unwrap();
        let outcome = OutcomeReport::Scalar(0);
        let outcome_info = OutcomeInfo { outcome_sum: vote_sum, owners };
        // is_finished is true,
        // because we need `lock_needed` to be zero to remove all locks.
        let winner_info = WinnerInfo {outcome, is_finished: true, outcome_info};

        let caller: T::AccountId = whitelisted_caller();
        let voter: T::AccountId = account("voter", 0, 0);
        let voter_lookup = T::Lookup::unlookup(voter.clone());
        let mut vote_locks: BoundedVec<(
            MarketIdOf<T>,
            BalanceOf<T>
        ), T::MaxGlobalDisputeVotes> = Default::default();
        for i in 0..l {
            let market_id: MarketIdOf<T> = i.saturated_into();
            let locked_balance: BalanceOf<T> = 1u128.saturated_into();
            vote_locks.try_push((market_id, locked_balance)).unwrap();
            <Winners<T>>::insert(market_id, winner_info.clone());
        }
        <Locks<T>>::insert(voter.clone(), vote_locks);
    }: {
        <Pallet<T>>::unlock_vote_balance(
            RawOrigin::Signed(caller.clone()).into(),
            voter_lookup,
        )
        .unwrap();
    } verify {
        let lock_info = <Locks<T>>::get(&voter);
        assert!(lock_info.is_empty());
    }

    add_vote_outcome {
        // concious decision for using component 0..MaxOwners here
        // because although we check that is_finished is false,
        // Winners counts processing time for the decoding of the owners vector.
        // then if the owner information is not present on Winners,
        // the owner info is present on Outcomes
        // this happens in the case, that Outcomes is not none at the query time.
        let w in 1..T::MaxOwners::get();

        let mut owners = Vec::new();
        for i in 1..=w {
            let owner = account("winners_owner", i, 0);
            owners.push(owner);
        }
        let owners = BoundedVec::try_from(owners).unwrap();
        let outcome_info = OutcomeInfo { outcome_sum: 42u128.saturated_into(), owners };
        let winner_info = WinnerInfo {
            outcome: OutcomeReport::Scalar(0),
            is_finished: false,
            outcome_info,
        };

        let caller: T::AccountId = whitelisted_caller();
        let market_id: MarketIdOf<T> = 0u128.saturated_into();
        let outcome = OutcomeReport::Scalar(20);
        <Winners<T>>::insert(market_id, winner_info);
        deposit::<T>(&caller);
    }: _(RawOrigin::Signed(caller.clone()), market_id, outcome.clone())
    verify {
        assert_last_event::<T>(Event::AddedVotingOutcome::<T> {
            market_id,
            owner: caller,
            outcome: outcome.clone(),
        }.into());
        let winner_info = <Winners<T>>::get(market_id).unwrap();
        assert_eq!(winner_info.outcome_info.outcome_sum, T::VotingOutcomeFee::get());
        // zero owners as long as dispute not finished and reward_outcome_owner not happened
        assert_eq!(winner_info.outcome_info.owners.len(), 0usize);

        let outcomes_item = <Outcomes<T>>::get(market_id, outcome).unwrap();
        assert_eq!(outcomes_item.outcome_sum, T::VotingOutcomeFee::get());
        assert_eq!(outcomes_item.owners.len(), 1usize);
    }

    reward_outcome_owner_with_funds {
        let o in 1..T::MaxOwners::get();

        let market_id: MarketIdOf<T> = 0u128.saturated_into();

        let mut owners_vec = Vec::new();
        for i in 1..=o {
            let owner = account("winners_owner", i, 0);
            owners_vec.push(owner);
        }
        let owners = BoundedVec::try_from(owners_vec.clone()).unwrap();

        let winner_info = WinnerInfo {
            outcome: OutcomeReport::Scalar(0),
            is_finished: true,
            outcome_info: OutcomeInfo {
                outcome_sum: 42u128.saturated_into(),
                owners,
            },
        };
        <Winners<T>>::insert(market_id, winner_info.clone());

        let reward_account = GlobalDisputes::<T>::reward_account(&market_id);
        let _ = T::Currency::deposit_creating(
            &reward_account,
            T::VotingOutcomeFee::get().saturating_mul(10u128.saturated_into()),
        );
        let reward_before = T::Currency::free_balance(&reward_account);

        let caller: T::AccountId = whitelisted_caller();

        let outcome = OutcomeReport::Scalar(20);

        deposit::<T>(&caller);
    }: {
        <Pallet<T>>::reward_outcome_owner(
            RawOrigin::Signed(caller.clone()).into(),
            market_id
        )
        .unwrap();
    } verify {
        assert!(winner_info.outcome_info.owners.len() == o as usize);
        assert_last_event::<T>(
            Event::OutcomeOwnersRewarded::<T> {
                market_id,
                owners: owners_vec,
            }
            .into(),
        );
        let remainder = reward_before.checked_rem(&o.into()).unwrap();
        let expected = if remainder < T::Currency::minimum_balance() {
            0u8.into()
        } else {
            remainder
        };
        assert_eq!(T::Currency::free_balance(&reward_account), expected);
    }

    reward_outcome_owner_no_funds {
        let o in 1..T::MaxOwners::get();

        let market_id: MarketIdOf<T> = 0u128.saturated_into();

        let mut owners = Vec::new();
        for i in 1..=o {
            let owner = account("winners_owner", i, 0);
            owners.push(owner);
        }
        let owners = BoundedVec::try_from(owners.clone()).unwrap();

        let winner_info = WinnerInfo {
            outcome: OutcomeReport::Scalar(0),
            is_finished: true,
            outcome_info: OutcomeInfo {
                outcome_sum: 42u128.saturated_into(),
                owners,
            },
        };
        <Winners<T>>::insert(market_id, winner_info);

        let caller: T::AccountId = whitelisted_caller();

        let outcome = OutcomeReport::Scalar(20);

        let reward_account = GlobalDisputes::<T>::reward_account(&market_id);
        assert!(T::Currency::free_balance(&reward_account) == 0u128.saturated_into());

        deposit::<T>(&caller);
    }: {
        <Pallet<T>>::reward_outcome_owner(RawOrigin::Signed(caller.clone()).into(), market_id)
            .unwrap();
    } verify {
        assert_last_event::<T>(Event::OutcomeOwnersRewardedWithNoFunds::<T> { market_id }.into());
    }

    purge_outcomes {
        // RemoveKeysLimit - 2 to ensure that we actually fully clean and return at the end
        let k in 1..(T::RemoveKeysLimit::get() - 2);

        let o in 1..T::MaxOwners::get();

        let market_id: MarketIdOf<T> = 0u128.saturated_into();

        for i in 1..=k {
            let owner = account("outcomes_owner", i, 0);
            GlobalDisputes::<T>::push_voting_outcome(
                &market_id,
                OutcomeReport::Scalar(i.into()),
                &owner,
                1_000_000_000u128.saturated_into(),
            )
            .unwrap();
        }

        let mut owners = Vec::new();
        for i in 1..=o {
            let owner = account("winners_owner", i, 0);
            owners.push(owner);
        }
        let owners = BoundedVec::try_from(owners.clone()).unwrap();
        let winner_outcome = OutcomeReport::Scalar(0);

        let outcome_info = OutcomeInfo {
            outcome_sum: 42u128.saturated_into(),
            owners
        };
        <Outcomes<T>>::insert(market_id, winner_outcome.clone(), outcome_info);

        let outcome_info = OutcomeInfo {
            outcome_sum: 42u128.saturated_into(),
            owners: Default::default()
        };
        let winner_info = WinnerInfo {outcome: winner_outcome, is_finished: true, outcome_info};
        <Winners<T>>::insert(market_id, winner_info);

        let caller: T::AccountId = whitelisted_caller();

        let outcome = OutcomeReport::Scalar(20);

        deposit::<T>(&caller);
    }: _(RawOrigin::Signed(caller.clone()), market_id)
    verify {
        assert!(<Outcomes<T>>::iter_prefix(market_id).next().is_none());
        assert_last_event::<T>(Event::OutcomesFullyCleaned::<T> { market_id }.into());
    }
}

impl_benchmark_test_suite!(
    GlobalDisputes,
    crate::mock::ExtBuilder::default().build(),
    crate::mock::Runtime
);
