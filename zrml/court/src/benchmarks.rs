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

#[cfg(test)]
use crate::Pallet as Court;
use crate::{BalanceOf, Call, Config, CurrencyOf, Pallet};
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_support::{dispatch::UnfilteredDispatchable, traits::Currency};
use frame_system::RawOrigin;
use sp_runtime::traits::Bounded;
use zeitgeist_primitives::types::OutcomeReport;

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
    Call::<T>::join_court {}
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
        let outcome = OutcomeReport::Scalar(u128::MAX);
        deposit_and_join_court::<T>(&caller);
    }: _(RawOrigin::Signed(caller), market_id, outcome)

    impl_benchmark_test_suite!(
        Court,
        crate::mock::ExtBuilder::default().build(),
        crate::mock::Runtime,
    );
}

