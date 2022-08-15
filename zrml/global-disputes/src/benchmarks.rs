#![allow(
    // Auto-generated code is a no man's land
    clippy::integer_arithmetic
)]
#![cfg(feature = "runtime-benchmarks")]

#[cfg(test)]
use crate::Pallet as GlobalDisputes;
use crate::{global_disputes_pallet_api::GlobalDisputesPalletApi, BalanceOf, Call, Config, Pallet};
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::{dispatch::UnfilteredDispatchable, traits::Currency};
use frame_system::RawOrigin;
use sp_runtime::traits::{Bounded, SaturatedConversion};
use zeitgeist_primitives::{constants::mock::MinOutcomeVoteAmount, types::OutcomeReport};

// ./target/release/zeitgeist benchmark pallet --chain=dev --steps=10 --repeat=1000 --pallet=zrml_global_disputes --extrinsic='*' --execution=wasm --wasm-execution=compiled --heap-pages=4096 --template=./misc/weight_template.hbs --output=./zrml/global-disputes/src/weights.rs

fn deposit<T>(caller: &T::AccountId)
where
    T: Config,
{
    let _ = T::Currency::deposit_creating(caller, BalanceOf::<T>::max_value());
}

fn deposit_and_vote<T>(caller: &T::AccountId)
where
    T: Config,
{
    deposit::<T>(caller);
    let market_id = 0u128.saturated_into();
    let outcome = OutcomeReport::Scalar(0);
    let amount: BalanceOf<T> = MinOutcomeVoteAmount::get().saturated_into();
    Pallet::<T>::push_voting_outcome(
        &market_id,
        OutcomeReport::Scalar(0),
        caller,
        10u128.saturated_into(),
    );
    Pallet::<T>::push_voting_outcome(
        &market_id,
        OutcomeReport::Scalar(20),
        caller,
        20u128.saturated_into(),
    );
    Call::<T>::vote_on_outcome { market_id, outcome, amount }
        .dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())
        .unwrap();
}

benchmarks! {
    vote_on_outcome {
        let caller: T::AccountId = whitelisted_caller();
        let market_id = 0u128.saturated_into();
        let outcome = OutcomeReport::Scalar(0);
        let amount: BalanceOf<T> = MinOutcomeVoteAmount::get().saturated_into();
        deposit::<T>(&caller);

        Pallet::<T>::push_voting_outcome(&market_id, OutcomeReport::Scalar(0), &caller, 10u128.saturated_into());
        Pallet::<T>::push_voting_outcome(&market_id, OutcomeReport::Scalar(20), &caller, 20u128.saturated_into());
    }: _(RawOrigin::Signed(caller), market_id, outcome, amount)

    unlock_vote_balance {
        let caller: T::AccountId = whitelisted_caller();
        deposit_and_vote::<T>(&caller);
    }: _(RawOrigin::Signed(caller.clone()), caller.clone())
}

impl_benchmark_test_suite!(
    GlobalDisputes,
    crate::mock::ExtBuilder::default().build(),
    crate::mock::Runtime
);
