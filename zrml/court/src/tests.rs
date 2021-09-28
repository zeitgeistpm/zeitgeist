#![cfg(test)]

use crate::{
    mock::{
        Balances, Court, ExtBuilder, Origin, RandomnessCollectiveFlip, Runtime, System, ALICE, BOB,
        CHARLIE, INITIAL_BALANCE,
    },
    Error, Juror, JurorStatus, Jurors, RequestedJurors, Votes, RESERVE_ID,
};
use core::ops::Range;
use frame_support::{
    assert_noop, assert_ok,
    traits::{Hooks, NamedReservableCurrency},
};
use sp_runtime::traits::Header;
use zeitgeist_primitives::{
    constants::BASE,
    traits::DisputeApi,
    types::{
        Market, MarketCreation, MarketDisputeMechanism, MarketPeriod, MarketStatus, MarketType,
        OutcomeReport,
    },
};

const DEFAULT_MARKET: Market<u128, u64, u64> = Market {
    creation: MarketCreation::Permissionless,
    creator_fee: 0,
    creator: 0,
    market_type: MarketType::Scalar(0..=100),
    mdm: MarketDisputeMechanism::Court,
    metadata: vec![],
    oracle: 0,
    period: MarketPeriod::Block(0..100),
    report: None,
    resolved_outcome: None,
    status: MarketStatus::Closed,
};
const DEFAULT_SET_OF_JURORS: &[(u128, Juror)] = &[
    (7, Juror { status: JurorStatus::Ok }),
    (6, Juror { status: JurorStatus::Tardy }),
    (5, Juror { status: JurorStatus::Ok }),
    (4, Juror { status: JurorStatus::Tardy }),
    (3, Juror { status: JurorStatus::Ok }),
    (2, Juror { status: JurorStatus::Ok }),
    (1, Juror { status: JurorStatus::Ok }),
];

#[test]
fn exit_court_successfully_removes_a_juror_and_frees_balances() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Court::join_court(Origin::signed(ALICE)));
        assert_eq!(Jurors::<Runtime>::iter().count(), 1);
        assert_eq!(Balances::free_balance(ALICE), 998 * BASE);
        assert_eq!(Balances::reserved_balance_named(&RESERVE_ID, &ALICE), 2 * BASE);
        assert_ok!(Court::exit_court(Origin::signed(ALICE)));
        assert_eq!(Jurors::<Runtime>::iter().count(), 0);
        assert_eq!(Balances::free_balance(ALICE), INITIAL_BALANCE);
        assert_eq!(Balances::reserved_balance_named(&RESERVE_ID, &ALICE), 0);
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
        assert_eq!(Balances::reserved_balance_named(&RESERVE_ID, &ALICE), 2 * BASE);

        assert_eq!(Balances::free_balance(BOB), 1000 * BASE);
        assert_ok!(Court::join_court(Origin::signed(BOB)));
        assert_eq!(Balances::free_balance(BOB), 996 * BASE);
        assert_eq!(Balances::reserved_balance_named(&RESERVE_ID, &BOB), 4 * BASE);
    });
}

#[test]
fn join_court_successfully_stores_a_juror() {
    ExtBuilder::default().build().execute_with(|| {
        assert_ok!(Court::join_court(Origin::signed(ALICE)));
        assert_eq!(
            Jurors::<Runtime>::iter().next().unwrap(),
            (ALICE, Juror { status: JurorStatus::Ok })
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
fn on_dispute_denies_non_court_markets() {
    ExtBuilder::default().build().execute_with(|| {
        let mut market = DEFAULT_MARKET;
        market.mdm = MarketDisputeMechanism::SimpleDisputes;
        assert_noop!(
            Court::on_dispute(&[], &0, &market),
            Error::<Runtime>::MarketDoesNotHaveCourtMechanism
        );
    });
}

#[test]
fn on_resolution_denies_non_court_markets() {
    ExtBuilder::default().build().execute_with(|| {
        let mut market = DEFAULT_MARKET;
        market.mdm = MarketDisputeMechanism::SimpleDisputes;
        assert_noop!(
            Court::on_resolution(&[], &0, &market),
            Error::<Runtime>::MarketDoesNotHaveCourtMechanism
        );
    });
}

#[test]
fn on_dispute_stores_jurors_that_should_vote() {
    ExtBuilder::default().build().execute_with(|| {
        setup_blocks(1..123);
        let _ = Court::join_court(Origin::signed(ALICE));
        let _ = Court::join_court(Origin::signed(BOB));
        Court::on_dispute(&[], &0, &DEFAULT_MARKET).unwrap();
        assert_noop!(
            Court::join_court(Origin::signed(ALICE)),
            Error::<Runtime>::JurorAlreadyExists
        );
        assert_eq!(RequestedJurors::<Runtime>::iter().count(), 2);
    });
}

// Alice is the winner, Bob is tardy and Charlie is the loser
#[test]
fn on_resolution_awards_winners_and_slashes_losers() {
    ExtBuilder::default().build().execute_with(|| {
        setup_blocks(1..2);
        Court::join_court(Origin::signed(ALICE)).unwrap();
        Court::join_court(Origin::signed(BOB)).unwrap();
        Court::join_court(Origin::signed(CHARLIE)).unwrap();
        Court::on_dispute(&[], &0, &DEFAULT_MARKET).unwrap();
        Court::vote(Origin::signed(ALICE), 0, OutcomeReport::Scalar(1)).unwrap();
        Court::vote(Origin::signed(BOB), 0, OutcomeReport::Scalar(2)).unwrap();
        Court::vote(Origin::signed(CHARLIE), 0, OutcomeReport::Scalar(3)).unwrap();
        let _ = Court::on_resolution(&[], &0, &DEFAULT_MARKET).unwrap();
        assert_eq!(Balances::free_balance(ALICE), 998 * BASE + 3 * BASE);
        assert_eq!(Balances::reserved_balance_named(&RESERVE_ID, &ALICE), 2 * BASE);
        assert_eq!(Balances::free_balance(BOB), 996 * BASE);
        assert_eq!(Balances::reserved_balance_named(&RESERVE_ID, &BOB), 4 * BASE);
        assert_eq!(Balances::free_balance(CHARLIE), 994 * BASE);
        assert_eq!(Balances::reserved_balance_named(&RESERVE_ID, &CHARLIE), 3 * BASE);
    });
}

#[test]
fn on_resolution_decides_market_outcome_based_on_the_majority() {
    ExtBuilder::default().build().execute_with(|| {
        setup_blocks(1..2);
        Court::join_court(Origin::signed(ALICE)).unwrap();
        Court::join_court(Origin::signed(BOB)).unwrap();
        Court::join_court(Origin::signed(CHARLIE)).unwrap();
        Court::on_dispute(&[], &0, &DEFAULT_MARKET).unwrap();
        Court::vote(Origin::signed(ALICE), 0, OutcomeReport::Scalar(1)).unwrap();
        Court::vote(Origin::signed(BOB), 0, OutcomeReport::Scalar(1)).unwrap();
        Court::vote(Origin::signed(CHARLIE), 0, OutcomeReport::Scalar(2)).unwrap();
        let outcome = Court::on_resolution(&[], &0, &DEFAULT_MARKET).unwrap();
        assert_eq!(outcome, OutcomeReport::Scalar(1))
    });
}

#[test]
fn on_resolution_sets_late_jurors_as_tardy() {
    ExtBuilder::default().build().execute_with(|| {
        setup_blocks(1..2);
        Court::join_court(Origin::signed(ALICE)).unwrap();
        Court::join_court(Origin::signed(BOB)).unwrap();
        Court::vote(Origin::signed(ALICE), 0, OutcomeReport::Scalar(1)).unwrap();
        Court::on_dispute(&[], &0, &DEFAULT_MARKET).unwrap();
        let _ = Court::on_resolution(&[], &0, &DEFAULT_MARKET).unwrap();
        assert_eq!(Jurors::<Runtime>::get(ALICE).unwrap().status, JurorStatus::Ok);
        assert_eq!(Jurors::<Runtime>::get(BOB).unwrap().status, JurorStatus::Tardy);
    });
}

#[test]
fn on_resolution_sets_jurors_that_voted_on_the_second_most_voted_outcome_as_tardy() {
    ExtBuilder::default().build().execute_with(|| {
        setup_blocks(1..2);
        Court::join_court(Origin::signed(ALICE)).unwrap();
        Court::join_court(Origin::signed(BOB)).unwrap();
        Court::join_court(Origin::signed(CHARLIE)).unwrap();
        Court::on_dispute(&[], &0, &DEFAULT_MARKET).unwrap();
        Court::vote(Origin::signed(ALICE), 0, OutcomeReport::Scalar(1)).unwrap();
        Court::vote(Origin::signed(BOB), 0, OutcomeReport::Scalar(1)).unwrap();
        Court::vote(Origin::signed(CHARLIE), 0, OutcomeReport::Scalar(2)).unwrap();
        let _ = Court::on_resolution(&[], &0, &DEFAULT_MARKET).unwrap();
        assert_eq!(Jurors::<Runtime>::get(CHARLIE).unwrap().status, JurorStatus::Tardy);
    });
}

#[test]
fn on_resolution_punishes_tardy_jurors_that_failed_to_vote_a_second_time() {
    ExtBuilder::default().build().execute_with(|| {
        setup_blocks(1..2);
        Court::join_court(Origin::signed(ALICE)).unwrap();
        Court::join_court(Origin::signed(BOB)).unwrap();
        Court::set_stored_juror_as_tardy(&BOB).unwrap();
        Court::vote(Origin::signed(ALICE), 0, OutcomeReport::Scalar(1)).unwrap();
        Court::on_dispute(&[], &0, &DEFAULT_MARKET).unwrap();
        let _ = Court::on_resolution(&[], &0, &DEFAULT_MARKET).unwrap();
        let join_court_stake = 40000000000;
        let slash = join_court_stake / 5;
        assert_eq!(Balances::free_balance(Court::treasury_account_id()), INITIAL_BALANCE + slash);
        assert_eq!(Balances::free_balance(BOB), INITIAL_BALANCE - slash);
        assert_eq!(Balances::reserved_balance_named(&RESERVE_ID, &BOB), 0);
    });
}

#[test]
fn on_resolution_removes_requested_jurors_and_votes() {
    ExtBuilder::default().build().execute_with(|| {
        setup_blocks(1..2);
        Court::join_court(Origin::signed(ALICE)).unwrap();
        Court::join_court(Origin::signed(BOB)).unwrap();
        Court::join_court(Origin::signed(CHARLIE)).unwrap();
        Court::on_dispute(&[], &0, &DEFAULT_MARKET).unwrap();
        Court::vote(Origin::signed(ALICE), 0, OutcomeReport::Scalar(1)).unwrap();
        Court::vote(Origin::signed(BOB), 0, OutcomeReport::Scalar(1)).unwrap();
        Court::vote(Origin::signed(CHARLIE), 0, OutcomeReport::Scalar(2)).unwrap();
        let _ = Court::on_resolution(&[], &0, &DEFAULT_MARKET).unwrap();
        assert_eq!(RequestedJurors::<Runtime>::iter().count(), 0);
        assert_eq!(Votes::<Runtime>::iter().count(), 0);
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
