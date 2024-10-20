// Copyright 2024 Forecasting Technologies LTD.
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

#![cfg(feature = "runtime-benchmarks")]

use crate::{types::Proposal, Call, Config, Event, Pallet, Proposals};
use alloc::vec;
use frame_benchmarking::v2::*;
use frame_support::{
    assert_ok,
    dispatch::RawOrigin,
    traits::{Bounded, Get},
};
use frame_system::{pallet_prelude::BlockNumberFor, Pallet as System};
use zeitgeist_primitives::traits::FutarchyBenchmarkHelper;

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn submit_proposal() {
        let duration = T::MinDuration::get();

        let oracle = T::BenchmarkHelper::create_oracle(true);
        let proposal = Proposal {
            when: Default::default(),
            call: Bounded::Inline(vec![7u8; 128].try_into().unwrap()),
            oracle,
        };

        #[extrinsic_call]
        _(RawOrigin::Root, duration, proposal.clone());

        let expected_event =
            <T as Config>::RuntimeEvent::from(Event::<T>::Submitted { duration, proposal });
        System::<T>::assert_last_event(expected_event.into());
    }

    #[benchmark]
    fn maybe_schedule_proposal() {
        let oracle = T::BenchmarkHelper::create_oracle(true);
        let proposal = Proposal {
            when: Default::default(),
            call: Bounded::Inline(vec![7u8; 128].try_into().unwrap()),
            oracle,
        };

        let block_number: BlockNumberFor<T> = 1u32.into();
        assert_ok!(Proposals::<T>::try_mutate(block_number, |proposals| {
            proposals.try_push(proposal.clone())
        }));

        #[block]
        {
            Pallet::<T>::maybe_schedule_proposal(proposal.clone());
        }

        let expected_event = <T as Config>::RuntimeEvent::from(Event::<T>::Scheduled { proposal });
        System::<T>::assert_last_event(expected_event.into());
    }

    impl_benchmark_test_suite!(
        Pallet,
        crate::mock::ext_builder::ExtBuilder::build(),
        crate::mock::runtime::Runtime
    );
}
