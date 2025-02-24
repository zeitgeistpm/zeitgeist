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

use super::*;

#[test]
fn submit_proposal_schedules_proposals() {
    ExtBuilder::build().execute_with(|| {
        let duration = <Runtime as Config>::MinDuration::get();

        let call = Bounded::Inline(vec![7u8; 128].try_into().unwrap());
        let oracle = MockOracle::new(Default::default(), true);
        let proposal = Proposal { when: Default::default(), call, oracle };

        // This ensures that if the scheduler is erroneously called, the test doesn't fail due to a
        // failure to configure the return value.
        MockScheduler::set_return_value(Ok(()));

        assert_ok!(Futarchy::submit_proposal(RawOrigin::Root.into(), duration, proposal.clone()));

        System::assert_last_event(
            Event::<Runtime>::Submitted { duration, proposal: proposal.clone() }.into(),
        );

        // Check that vector now contains proposal.
        let now = System::block_number();
        let to_be_scheduled_at = now + duration;
        assert_eq!(Proposals::get(to_be_scheduled_at).pop(), Some(proposal.clone()));

        utility::run_to_block(to_be_scheduled_at);

        // The proposal has now been removed and failed.
        assert!(Proposals::<Runtime>::get(to_be_scheduled_at).is_empty());
        assert!(MockScheduler::called_once_with(
            DispatchTime::At(proposal.when),
            proposal.call.clone()
        ));

        System::assert_last_event(Event::<Runtime>::Scheduled { proposal }.into());
    });
}

#[test]
fn submit_proposal_rejects_proposals() {
    ExtBuilder::build().execute_with(|| {
        let duration = <Runtime as Config>::MinDuration::get();

        let call = Bounded::Inline(vec![7u8; 128].try_into().unwrap());
        let oracle = MockOracle::new(Default::default(), false);
        let proposal = Proposal { when: Default::default(), call, oracle };

        // This ensures that if the scheduler is erroneously called, the test doesn't fail due to a
        // failure to configure the return value.
        MockScheduler::set_return_value(Ok(()));

        assert_ok!(Futarchy::submit_proposal(RawOrigin::Root.into(), duration, proposal.clone()));

        System::assert_last_event(
            Event::<Runtime>::Submitted { duration, proposal: proposal.clone() }.into(),
        );

        // Check that vector now contains proposal.
        let now = System::block_number();
        let to_be_scheduled_at = now + duration;
        assert_eq!(Proposals::get(to_be_scheduled_at).pop(), Some(proposal.clone()));

        utility::run_to_block(to_be_scheduled_at);

        // The proposal has now been removed and failed.
        assert!(Proposals::<Runtime>::get(to_be_scheduled_at).is_empty());
        assert!(MockScheduler::not_called());

        System::assert_last_event(Event::<Runtime>::Rejected { proposal }.into());
    });
}

#[test]
fn submit_proposal_fails_on_bad_origin() {
    ExtBuilder::build().execute_with(|| {
        let alice = Account::new(0);

        let duration = <Runtime as Config>::MinDuration::get();

        let call = Bounded::Inline(vec![7u8; 128].try_into().unwrap());
        let oracle = MockOracle::new(Default::default(), Default::default());
        let proposal = Proposal { when: Default::default(), call, oracle };

        assert_noop!(
            Futarchy::submit_proposal(alice.signed(), duration, proposal),
            DispatchError::BadOrigin,
        );
    });
}

#[test]
fn submit_proposal_fails_if_duration_is_too_short() {
    ExtBuilder::build().execute_with(|| {
        let duration = <Runtime as Config>::MinDuration::get() - 1;

        let call = Bounded::Inline(vec![7u8; 128].try_into().unwrap());
        let oracle = MockOracle::new(Default::default(), Default::default());
        let proposal = Proposal { when: Default::default(), call, oracle };

        assert_noop!(
            Futarchy::submit_proposal(RawOrigin::Root.into(), duration, proposal),
            Error::<Runtime>::DurationTooShort
        );
    });
}

#[test]
fn submit_proposal_fails_if_cache_is_full() {
    ExtBuilder::build().execute_with(|| {
        let duration = <Runtime as Config>::MinDuration::get();

        let call = Bounded::Inline(vec![7u8; 128].try_into().unwrap());
        let oracle = MockOracle::new(Default::default(), Default::default());
        let proposal = Proposal { when: Default::default(), call, oracle };

        // Mock up a full vector of proposals.
        let now = System::block_number();
        let to_be_scheduled_at = now + duration;
        let max_proposals: u32 = <Runtime as Config>::MaxProposals::get();
        let proposals_vec = vec![proposal.clone(); max_proposals as usize];
        let proposals: ProposalsOf<Runtime> = proposals_vec.try_into().unwrap();
        Proposals::<Runtime>::insert(to_be_scheduled_at, proposals);

        assert_noop!(
            Futarchy::submit_proposal(RawOrigin::Root.into(), duration, proposal),
            Error::<Runtime>::CacheFull
        );
    });
}
