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

#![allow(
    // Auto-generated code is a no man's land
    clippy::integer_arithmetic
)]
#![cfg(feature = "runtime-benchmarks")]

use crate::{
    global_disputes_pallet_api::GlobalDisputesPalletApi, BalanceOf, Call, Config,
    Pallet as GlobalDisputes, *,
};
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::{
    sp_runtime::traits::StaticLookup,
    traits::{Currency, Get},
    BoundedVec,
};
use frame_system::RawOrigin;
use sp_runtime::traits::{Bounded, SaturatedConversion};
use sp_std::prelude::*;
use zeitgeist_primitives::types::OutcomeReport;

// ./target/release/zeitgeist benchmark pallet --chain=dev --steps=20 --repeat=50 --pallet=zrml_global_disputes --extrinsic='*' --execution=wasm --wasm-execution=compiled --heap-pages=4096 --template=./misc/weight_template.hbs --output=./zrml/global-disputes/src/weights.rs

fn deposit<T>(caller: &T::AccountId)
where
    T: Config,
{
    let _ = T::Currency::deposit_creating(caller, BalanceOf::<T>::max_value());
}

benchmarks! {
    vote_on_outcome {
        // only Outcomes owners, but not Winners owners is present during vote_on_outcome
        let o in 1..(T::MaxOwners::get().into());

        let v in 0..(T::MaxGlobalDisputeVotes::get().into());

        let caller: T::AccountId = whitelisted_caller();
        // ensure that we get the worst case to actually insert the new item at the end of the binary search
        let market_id: MarketIdOf<T> = (v + 1).into();
        let outcome = OutcomeReport::Scalar(0);
        let amount: BalanceOf<T> = T::MinOutcomeVoteAmount::get().saturated_into();
        deposit::<T>(&caller);
        for i in 0..o {
            let owner = account("outcomes_owner", i, 0);
            GlobalDisputes::<T>::push_voting_outcome(&market_id, OutcomeReport::Scalar(0), &owner, 10_000u128.saturated_into());
        }

        let mut vote_locks = Vec::new();
        for i in 0..v {
            let market_id: MarketIdOf<T> = i.saturated_into();
            let locked_balance: BalanceOf<T> = T::MinOutcomeVoteAmount::get().saturated_into();
            vote_locks.push((market_id, locked_balance));
        }
        <Locks<T>>::insert(caller.clone(), LockInfo(vote_locks));

        // minus one to ensure, that we use the worst case for using a new winner info after the vote_on_outcome call
        let vote_sum = amount - 1u128.saturated_into();
        let winner_info = WinnerInfo {outcome: outcome.clone(), vote_sum, is_finished: false, owners: Default::default()};
        <Winners<T>>::insert(market_id, winner_info);
    }: _(RawOrigin::Signed(caller), market_id, outcome, amount)

    unlock_vote_balance {
        let l in 0..(T::MaxGlobalDisputeVotes::get().into());
        let o in 0..(T::MaxOwners::get().into());

        let vote_sum = 42u128.saturated_into();
        let mut owners = Vec::new();
        for i in 0..o {
            let owner = account("winners_owner", i, 0);
            owners.push(owner);
        }
        let owners = BoundedVec::try_from(owners).unwrap();
        let outcome = OutcomeReport::Scalar(0);
        // is_finished is true, because we want the worst case to actually delete list items of the locks
        let winner_info = WinnerInfo {outcome: outcome.clone(), vote_sum, is_finished: true, owners};

        let caller: T::AccountId = whitelisted_caller();
        let voter: T::AccountId = account("voter", 0, 0);
        let voter_lookup = T::Lookup::unlookup(voter.clone());
        let mut vote_locks = Vec::new();
        for i in 0..l {
            let market_id: MarketIdOf<T> = i.saturated_into();
            let locked_balance: BalanceOf<T> = 1u128.saturated_into();
            vote_locks.push((market_id, locked_balance));
            <Winners<T>>::insert(market_id, winner_info.clone());
        }
        <Locks<T>>::insert(voter.clone(), LockInfo(vote_locks));
    }: _(RawOrigin::Signed(caller.clone()), voter_lookup.clone())

    add_vote_outcome {
        // concious decision for using component 0..MaxOwners here
        // because although we check that is_finished is false,
        // Winners counts processing time for the decoding of the owners vector.
        // then if the owner information is not present on Winners, the owner info is present on Outcomes
        // this happens in the case, that Outcomes is not none at the query time.
        let w in 0..(T::MaxOwners::get().into());

        let mut owners = Vec::new();
        for i in 0..w {
            let owner = account("winners_owner", i, 0);
            owners.push(owner);
        }
        let owners = BoundedVec::try_from(owners).unwrap();
        let winner_info = WinnerInfo {outcome: OutcomeReport::Scalar(0), vote_sum: 42u128.saturated_into(), is_finished: false, owners};

        let caller: T::AccountId = whitelisted_caller();
        let market_id: MarketIdOf<T> = 0u128.saturated_into();
        let outcome = OutcomeReport::Scalar(20);
        <Winners<T>>::insert(market_id, winner_info.clone());
        deposit::<T>(&caller);
    }: _(RawOrigin::Signed(caller.clone()), market_id, outcome)

    reward_outcome_owner {
        let o in 0..(T::MaxOwners::get().into());

        let k in 0..(T::RemoveKeysLimit::get().into());

        let market_id: MarketIdOf<T> = 0u128.saturated_into();

        for i in 0..=k {
            let owner = account("outcomes_owner", i, 0);
            GlobalDisputes::<T>::push_voting_outcome(&market_id, OutcomeReport::Scalar(i.into()), &owner, 1_000u128.saturated_into());
        }

        let mut owners = Vec::new();
        for i in 0..o {
            let owner = account("winners_owner", i, 0);
            owners.push(owner);
        }
        let owners = BoundedVec::try_from(owners).unwrap();
        let winner_outcome = OutcomeReport::Scalar(0);

        let outcome_info = OutcomeInfo {outcome_sum: 42u128.saturated_into(), owners};
        <Outcomes<T>>::insert(market_id, winner_outcome.clone(), outcome_info.clone());

        let winner_info = WinnerInfo {outcome: winner_outcome.clone(), vote_sum: 42u128.saturated_into(), is_finished: true, owners: Default::default()};
        <Winners<T>>::insert(market_id, winner_info.clone());

        let reward_account = GlobalDisputes::<T>::reward_account(&market_id);
        let _ = T::Currency::deposit_creating(&reward_account, 100_000u128.saturated_into());

        let caller: T::AccountId = whitelisted_caller();

        let outcome = OutcomeReport::Scalar(20);

        deposit::<T>(&caller);
    }: _(RawOrigin::Signed(caller.clone()), market_id)
}

impl_benchmark_test_suite!(
    GlobalDisputes,
    crate::mock::ExtBuilder::default().build(),
    crate::mock::Runtime
);
