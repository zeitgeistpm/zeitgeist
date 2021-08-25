#![cfg(test)]

use crate::{
    mock::{
        Balances, Court, ExtBuilder, Origin, RandomnessCollectiveFlip, Runtime, System, ALICE, BOB,
    },
    Error, Juror, JurorStatus, Jurors, RequestedJurors, Votes,
};
use core::ops::Range;
use frame_support::{assert_noop, assert_ok, traits::Hooks};
use sp_runtime::traits::Header;
use zeitgeist_primitives::{
    constants::BASE,
    traits::DisputeApi,
    types::{
        Market, MarketCreation, MarketDisputeMechanism, MarketEnd, MarketStatus, MarketType,
        OutcomeReport,
    },
};

const DEFAULT_SET_OF_JURORS: &[(u128, Juror<u128>)] = &[
    (7, Juror { staked: 1, status: JurorStatus::Ok }),
    (6, Juror { staked: 2, status: JurorStatus::Tardy }),
    (5, Juror { staked: 3, status: JurorStatus::Ok }),
    (4, Juror { staked: 4, status: JurorStatus::Tardy }),
    (3, Juror { staked: 5, status: JurorStatus::Ok }),
    (2, Juror { staked: 6, status: JurorStatus::Ok }),
    (1, Juror { staked: 7, status: JurorStatus::Ok }),
];

#[test]
fn exit_court_successfully_removes_a_juror() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Court::join_court(Origin::signed(ALICE)));
        assert_eq!(Jurors::<Runtime>::iter().count(), 1);
        assert_ok!(Court::exit_court(Origin::signed(ALICE)));
        assert_eq!(Jurors::<Runtime>::iter().count(), 0);
    });
}

#[test]
fn exit_court_will_not_remove_an_unknown_juror() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            Court::exit_court(Origin::signed(ALICE)),
            Error::<Runtime>::JurorDoesNotExists
        );
    });
}

#[test]
fn join_court_reserves_balance_according_to_the_number_of_jurors() {
    ExtBuilder::default().build().execute_with(|| {
        assert_eq!(Balances::free_balance(ALICE), 1000 * BASE);
        assert_ok!(Court::join_court(Origin::signed(ALICE)));
        assert_eq!(Balances::free_balance(ALICE), 998 * BASE);
        assert_eq!(Balances::reserved_balance(ALICE), 2 * BASE);

        assert_eq!(Balances::free_balance(BOB), 1000 * BASE);
        assert_ok!(Court::join_court(Origin::signed(BOB)));
        assert_eq!(Balances::free_balance(BOB), 996 * BASE);
        assert_eq!(Balances::reserved_balance(BOB), 4 * BASE);
    });
}

#[test]
fn join_court_successfully_stores_a_juror() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Court::join_court(Origin::signed(ALICE)));
        assert_eq!(
            Jurors::<Runtime>::iter().next().unwrap(),
            (ALICE, Juror { staked: 2 * BASE, status: JurorStatus::Ok })
        );
    });
}

#[test]
fn join_court_will_not_insert_an_already_stored_juror() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Court::join_court(Origin::signed(ALICE)));
        assert_noop!(
            Court::join_court(Origin::signed(ALICE)),
            Error::<Runtime>::JurorAlreadyExists
        );
    });
}

#[test]
fn on_dispute_stores_jurors_that_should_vote() {
    ExtBuilder::default().build().execute_with(|| {
        setup_blocks(1..123);
        let _ = Court::join_court(Origin::signed(ALICE));
        let _ = Court::join_court(Origin::signed(BOB));
        let _ = Court::on_dispute(&[], 0);
        assert_noop!(
            Court::join_court(Origin::signed(ALICE)),
            Error::<Runtime>::JurorAlreadyExists
        );
        assert_eq!(RequestedJurors::<Runtime>::iter().count(), 2);
    });
}

#[test]
fn random_jurors_returns_an_unique_different_subset_of_jurors() {
    ExtBuilder::default().build().execute_with(|| {
        let mut block: u64 = 123;
        setup_blocks(1..block);

        let mut rng = Court::rng();
        let random_jurors = Court::random_jurors(&DEFAULT_SET_OF_JURORS, 2, &mut rng);
        let mut at_least_one_set_is_different = false;

        for _ in 0..100 {
            let next_block = block + 1;
            setup_blocks(block..next_block);
            block = next_block;

            let another_set_of_random_jurors =
                Court::random_jurors(&DEFAULT_SET_OF_JURORS, 2, &mut rng);
            let mut iter = another_set_of_random_jurors.iter();

            if let Some(juror) = iter.next() {
                at_least_one_set_is_different = random_jurors.iter().all(|el| el != juror);
            } else {
                at_least_one_set_is_different = false;
                continue;
            }
            for juror in iter {
                at_least_one_set_is_different &= random_jurors.iter().all(|el| el != juror);
            }

            if at_least_one_set_is_different {
                break;
            }
        }
        assert_eq!(at_least_one_set_is_different, true);
    });
}

#[test]
fn random_jurors_returns_a_subset_of_jurors() {
    ExtBuilder::default().build().execute_with(|| {
        setup_blocks(1..123);
        let mut rng = Court::rng();
        let random_jurors = Court::random_jurors(&DEFAULT_SET_OF_JURORS, 2, &mut rng);
        for (_, juror) in random_jurors {
            assert!(DEFAULT_SET_OF_JURORS.iter().any(|el| &el.1 == juror));
        }
    });
}

#[test]
fn vote_will_not_accept_unknown_accounts() {
    ExtBuilder::default().build().execute_with(|| {
        assert_noop!(
            Court::vote(Origin::signed(ALICE), 0, OutcomeReport::Scalar(0)),
            Error::<Runtime>::OnlyJurorsCanVote
        );
    });
}

#[test]
fn vote_will_stored_outcome_from_a_juror() {
    ExtBuilder::default().build().execute_with(|| {
        let _ = Court::join_court(Origin::signed(ALICE));
        let _ = Court::vote(Origin::signed(ALICE), 0, OutcomeReport::Scalar(0));
        assert_eq!(Votes::<Runtime>::get(ALICE, 0).unwrap(), (0, OutcomeReport::Scalar(0)));
    });
}

fn setup_blocks(range: Range<u64>) {
    let mut parent_hash = System::parent_hash();

    for i in range {
        System::initialize(&i, &parent_hash, &Default::default(), frame_system::InitKind::Full);
        RandomnessCollectiveFlip::on_initialize(i);

        let header = System::finalize();
        parent_hash = header.hash();
        System::set_block_number(*header.number());
    }
}
