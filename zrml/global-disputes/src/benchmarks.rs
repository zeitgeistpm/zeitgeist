#![allow(
    // Auto-generated code is a no man's land
    clippy::integer_arithmetic
)]
#![cfg(feature = "runtime-benchmarks")]

#[cfg(test)]
use crate::Pallet as GlobalDisputes;
use crate::{
    global_disputes_pallet_api::GlobalDisputesPalletApi, BalanceOf, Call, Config, CurrencyOf,
    Pallet,
};
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_support::{dispatch::UnfilteredDispatchable, traits::Currency};
use frame_system::RawOrigin;
use sp_runtime::traits::Bounded;
use zeitgeist_primitives::constants::{BASE, MinLiquidity};

fn deposit<T>(caller: &T::AccountId)
where
    T: Config,
{
    let _ = CurrencyOf::<T>::deposit_creating(caller, BalanceOf::<T>::max_value());
}

fn deposit_and_vote<T>(caller: &T::AccountId)
where
    T: Config,
{
    deposit::<T>(caller);
    let market_id = Default::default();
    let dispute_index = Default::default();
    let amount: BalanceOf<T> = 1000u128.into();
    Pallet::<T>::init_dispute_vote(&market_id, dispute_index, 10u128.into());
    Pallet::<T>::init_dispute_vote(&market_id, dispute_index + 1, 20u128.into());
    Call::<T>::vote { market_id, dispute_index, amount }
        .dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())
        .unwrap();
}

benchmarks! {
    vote {
        let caller: T::AccountId = whitelisted_caller();
        let market_id = Default::default();
        let dispute_index = Default::default();
        let amount: BalanceOf<T> = 1000u128.into();
        Pallet::<T>::init_dispute_vote(&market_id, dispute_index, 10u128.into());
        Pallet::<T>::init_dispute_vote(&market_id, dispute_index + 1, 20u128.into());
        deposit::<T>(&caller);
    }: _(RawOrigin::Signed(caller), market_id, dispute_index, amount)

    unlock {
        let caller: T::AccountId = whitelisted_caller();
        deposit_and_vote::<T>(&caller);
    }: _(RawOrigin::Signed(caller))
}

impl_benchmark_test_suite!(
    GlobalDisputes,
    crate::mock::ExtBuilder::default().build(),
    crate::mock::Runtime
);
