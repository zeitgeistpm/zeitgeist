#![allow(
    // Auto-generated code is a no man's land
    clippy::integer_arithmetic
)]
#![cfg(feature = "runtime-benchmarks")]

#[cfg(test)]
extern crate alloc;
use crate::{
    global_disputes_pallet_api::GlobalDisputesPalletApi, BalanceOf, Call, Config,
    Pallet as GlobalDisputes, Pallet, *,
};
use alloc::vec;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::{
    dispatch::UnfilteredDispatchable,
    sp_runtime::traits::StaticLookup,
    traits::{Currency, Get},
    BoundedVec,
};
use frame_system::RawOrigin;
use sp_runtime::traits::{Bounded, SaturatedConversion};
use zeitgeist_primitives::{constants::mock::MinOutcomeVoteAmount, types::OutcomeReport};

// ./target/release/zeitgeist benchmark pallet --chain=dev --steps=20 --repeat=50 --pallet=zrml_global_disputes --extrinsic='*' --execution=wasm --wasm-execution=compiled --heap-pages=4096 --template=./misc/weight_template.hbs --output=./zrml/global-disputes/src/weights.rs

fn deposit<T>(caller: &T::AccountId)
where
    T: Config,
{
    let _ = T::Currency::deposit_creating(caller, BalanceOf::<T>::max_value());
}

benchmarks! {
    vote_on_outcome {
        let o in 0..(T::MaxOwners::get().into());

        let w in 0..(T::MaxOwners::get().into());

        let caller: T::AccountId = whitelisted_caller();
        let market_id = 0u128.saturated_into();
        let outcome = OutcomeReport::Scalar(0);
        let amount: BalanceOf<T> = MinOutcomeVoteAmount::get().saturated_into();
        deposit::<T>(&caller);
        for i in 0..=o {
            let owner = account("outcomes_owner", i, 0);
            GlobalDisputes::<T>::push_voting_outcome(&market_id, OutcomeReport::Scalar(0), &owner, 10u128.saturated_into());
        }

        let locked_balance: BalanceOf<T> = amount - 1u128.saturated_into();
        <LockInfoOf<T>>::insert(caller.clone(), market_id, locked_balance);
        let vote_sum = amount - 1u128.saturated_into();
        let mut owners = vec![];
        for i in 0..w {
            let owner = account("winners_owner", i, 0);
            owners.push(owner);
        }
        let owners = BoundedVec::try_from(owners).unwrap();
        let winner_info = WinnerInfo {outcome: outcome.clone(), vote_sum, is_finished: false, owners};
        <Winners<T>>::insert(market_id, winner_info);
    }: _(RawOrigin::Signed(caller), market_id, outcome, amount)

    unlock_vote_balance {
        let l in 0..(T::MaxLocks::get().into());
        let o in 0..(T::MaxOwners::get().into());

        let vote_sum = 42u128.saturated_into();
        let mut owners = vec![];
        for i in 0..o {
            let owner = account("winners_owner", i, 0);
            owners.push(owner);
        }
        let owners = BoundedVec::try_from(owners).unwrap();
        let outcome = OutcomeReport::Scalar(0);
        let winner_info = WinnerInfo {outcome: outcome.clone(), vote_sum, is_finished: true, owners};

        let caller: T::AccountId = whitelisted_caller();
        let voter: T::AccountId = account("voter", 0, 0);
        let voter_lookup = T::Lookup::unlookup(voter.clone());
        for i in 0..=l {
            let market_id: MarketIdOf<T> = i.saturated_into();
            let locked_balance: BalanceOf<T> = 1u128.saturated_into();
            <LockInfoOf<T>>::insert(voter.clone(), market_id, locked_balance);
            <Winners<T>>::insert(market_id, winner_info.clone());
        }

        deposit::<T>(&caller);
    }: _(RawOrigin::Signed(caller.clone()), voter_lookup.clone())
}

impl_benchmark_test_suite!(
    GlobalDisputes,
    crate::mock::ExtBuilder::default().build(),
    crate::mock::Runtime
);
