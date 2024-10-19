use super::*;

#[test]
fn submit_proposal_rejects_proposals() {
    ExtBuilder::build().execute_with(|| {
        let duration = <Runtime as Config>::MinDuration::get();

        let remark = SystemCall::remark { remark: "hullo".into() };
        let call = Preimage::bound(CallOf::<Runtime>::from(remark)).unwrap();
        let query = MockOracleQuery::new(Default::default(), false);
        let proposal = Proposal { when: Default::default(), call, query };

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
        assert_eq!(Proposals::get(to_be_scheduled_at).pop(), Some(proposal));

        utility::run_to_block(to_be_scheduled_at);

        // The proposal has now been removed and failed.
        assert!(Proposals::<Runtime>::get(to_be_scheduled_at).is_empty());
        assert!(MockScheduler::not_called());

        System::assert_last_event(Event::<Runtime>::Rejected.into());
    });
}

#[test]
fn submit_proposal_schedules_proposals() {
    ExtBuilder::build().execute_with(|| {
        let duration = <Runtime as Config>::MinDuration::get();

        let remark = SystemCall::remark { remark: "hullo".into() };
        let call = Preimage::bound(CallOf::<Runtime>::from(remark)).unwrap();
        let query = MockOracleQuery::new(Default::default(), true);
        let proposal = Proposal { when: Default::default(), call, query };

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
        assert!(MockScheduler::called_once_with(DispatchTime::At(proposal.when), proposal.call));

        System::assert_last_event(Event::<Runtime>::Scheduled.into());
    });
}

#[test]
fn submit_proposal_fails_on_bad_origin() {
    ExtBuilder::build().execute_with(|| {
        let alice = Account::new(0);

        let duration = <Runtime as Config>::MinDuration::get();

        let remark = SystemCall::remark { remark: "hullo".into() };
        let call = Preimage::bound(CallOf::<Runtime>::from(remark)).unwrap();
        let query = MockOracleQuery::new(Default::default(), Default::default());
        let proposal = Proposal { when: Default::default(), call, query };

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

        let remark = SystemCall::remark { remark: "hullo".into() };
        let call = Preimage::bound(CallOf::<Runtime>::from(remark)).unwrap();
        let query = MockOracleQuery::new(Default::default(), Default::default());
        let proposal = Proposal { when: Default::default(), call, query };

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

        let remark = SystemCall::remark { remark: "hullo".into() };
        let call = Preimage::bound(CallOf::<Runtime>::from(remark)).unwrap();
        let query = MockOracleQuery::new(Default::default(), Default::default());
        let proposal = Proposal { when: Default::default(), call, query };

        // Mock up a full vector of proposals.
        let now = System::block_number();
        let to_be_scheduled_at = now + duration;
        let cache_size: u32 = <CacheSize as Get<Option<u32>>>::get().unwrap();
        let proposals_vec = vec![proposal.clone(); cache_size as usize];
        let proposals: BoundedVec<_, CacheSize> = proposals_vec.try_into().unwrap();
        Proposals::<Runtime>::insert(to_be_scheduled_at, proposals);

        assert_noop!(
            Futarchy::submit_proposal(RawOrigin::Root.into(), duration, proposal),
            Error::<Runtime>::CacheFull
        );
    });
}
