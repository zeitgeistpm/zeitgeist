#![allow(
    // Auto-generated code is a no man's land
    clippy::integer_arithmetic
)]
#![cfg(feature = "runtime-benchmarks")]

#[cfg(test)]
use crate::Pallet as Court;
use crate::{BalanceOf, Call, Config, CurrencyOf, Pallet};
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::{dispatch::UnfilteredDispatchable, traits::Currency};
use frame_system::RawOrigin;
use sp_runtime::traits::Bounded;
use zeitgeist_primitives::types::Outcome;

fn deposit<T>(caller: &T::AccountId)
where
    T: Config,
{
    let _ = CurrencyOf::<T>::deposit_creating(caller, BalanceOf::<T>::max_value());
}

fn deposit_and_join_court<T>(caller: &T::AccountId)
where
    T: Config,
{
    deposit::<T>(caller);
    Call::<T>::join_court()
        .dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())
        .unwrap();
}

benchmarks! {
    exit_court {
        let caller: T::AccountId = whitelisted_caller();
        deposit_and_join_court::<T>(&caller);
    }: _(RawOrigin::Signed(caller))

    join_court {
        let caller: T::AccountId = whitelisted_caller();
        deposit::<T>(&caller);
    }: _(RawOrigin::Signed(caller))

    vote {
        let caller: T::AccountId = whitelisted_caller();
        let market_id = Default::default();
        let outcome = Outcome::Scalar(u128::MAX);
        deposit_and_join_court::<T>(&caller);
    }: _(RawOrigin::Signed(caller), market_id, outcome)
}

impl_benchmark_test_suite!(Court, crate::mock::ExtBuilder::default().build(), crate::mock::Runtime);
