// Copyright 2024-2025 Forecasting Technologies LTD.
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

use crate::{traits::ProposalStorage, types::Proposal, Call, Config, Event, Pallet, Proposals};
use alloc::vec;
use frame_benchmarking::v2::*;
use frame_support::{
    dispatch::RawOrigin,
    traits::{Bounded, Get},
};
use frame_system::Pallet as System;
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

        let now = System::<T>::block_number();
        let to_be_scheduled_at = now + duration;
        let mut proposals = Proposals::<T>::get(to_be_scheduled_at);
        for _ in 0..(T::MaxProposals::get() - 1) {
            proposals.try_push(proposal.clone()).unwrap();
        }
        Proposals::<T>::insert(to_be_scheduled_at, proposals);

        #[extrinsic_call]
        _(RawOrigin::Root, duration, proposal.clone());

        let expected_event =
            <T as Config>::RuntimeEvent::from(Event::<T>::Submitted { duration, proposal });
        System::<T>::assert_last_event(expected_event.into());
    }

    #[benchmark]
    fn maybe_schedule_proposal() {
        let when = u32::MAX.into();
        let oracle = T::BenchmarkHelper::create_oracle(true);
        let proposal =
            Proposal { when, call: Bounded::Inline(vec![7u8; 128].try_into().unwrap()), oracle };

        #[block]
        {
            Pallet::<T>::maybe_schedule_proposal(proposal.clone());
        }

        let expected_event = <T as Config>::RuntimeEvent::from(Event::<T>::Scheduled { proposal });
        System::<T>::assert_last_event(expected_event.into());
    }

    #[benchmark]
    fn take_proposals(n: Linear<1, 4>) {
        let when = u32::MAX.into();
        let oracle = T::BenchmarkHelper::create_oracle(true);
        let proposal =
            Proposal { when, call: Bounded::Inline(vec![7u8; 128].try_into().unwrap()), oracle };

        let now = System::<T>::block_number();
        let mut proposals = Proposals::<T>::get(now);
        for _ in 0..n {
            proposals.try_push(proposal.clone()).unwrap();
        }
        Proposals::<T>::insert(now, proposals);

        #[block]
        {
            let _ = <Pallet<T> as ProposalStorage<T>>::take(now);
        }
    }

    impl_benchmark_test_suite!(
        Pallet,
        crate::mock::ext_builder::ExtBuilder::build(),
        crate::mock::runtime::Runtime
    );
}
