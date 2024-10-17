use super::*;

#[test]
fn submit_proposal_fails_on_bad_origin() {
    ExtBuilder::build().execute_with(|| {
        let alice = Account::new(0);

        let duration = <Runtime as Config>::MinDuration::get();

        let call = RuntimeCall::System(SystemCall::remark {
            msg: "hullo",
            weight: Weight::from_parts(1, 2),
        });
        let query = MockOracleQuery { weight: Default::default(), value: Default::default() };
        let proposal = Proposal { when: Default::default(), call, query };
        assert_noop!(
            Futarchy::submit_proposal(alice.signed(), duration, proposal),
            DispatchError::BadOrigin,
        );
    });
}

// #[test]
// fn submit_proposal_fails_if_duration_is_too_short() {
//     ExtBuilder::build().execute_with(|| {
//
//         let duration = <Runtime as Config>::MinDuration::get() - 1;
//         let query = MockQuery { weight: 1u128, value: false };
//         let proposal = Proposal {
//             when: Default(),
//             call: Default(),
//             query,
//         }
//         assert_noop!(
//             Futarchy::submit_proposal(duration, proposal),
//             Er
//         )
//     });
// }
