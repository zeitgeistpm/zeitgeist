// Copyright 2022-2023 Forecasting Technologies LTD.
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

//! Crowdfunding pallet benchmarking.

#![allow(
    // Auto-generated code is a no man's land
    clippy::integer_arithmetic
)]
#![cfg(feature = "runtime-benchmarks")]

use crate::{
    types::*, BalanceOf, Call, Config,
    Pallet as Crowdfund, *,
};
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
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
    fund {
        let o in 1..T::MaxOwners::get();

        let v in 0..(T::MaxGlobalDisputeVotes::get() - 1);

        let caller: T::AccountId = whitelisted_caller();
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

    impl_benchmark_test_suite!(
        Crowdfund,
        crate::mock::ExtBuilder::default().build(),
        crate::mock::Runtime,
    );
}
