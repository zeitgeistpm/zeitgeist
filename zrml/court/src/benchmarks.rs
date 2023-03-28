// Copyright 2023 Forecasting Technologies LTD.
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
use crate::{BalanceOf, Call, Config, Pallet};
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_support::traits::{Currency, Get};
use frame_system::RawOrigin;
use sp_runtime::traits::Bounded;

fn deposit<T>(caller: &T::AccountId)
where
    T: Config,
{
    let _ = T::Currency::deposit_creating(caller, BalanceOf::<T>::max_value());
}

benchmarks! {
    join_court {
        let caller: T::AccountId = whitelisted_caller();
        deposit::<T>(&caller);
        let amount = T::MinJurorStake::get();
    }: _(RawOrigin::Signed(caller), amount)


    impl_benchmark_test_suite!(
        Court,
        crate::mock::ExtBuilder::default().build(),
        crate::mock::Runtime,
    );
}
