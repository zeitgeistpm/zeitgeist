#![cfg(test)]

use crate::{
    mock::{
        Balances, Court, Event, ExtBuilder, Origin, RandomnessCollectiveFlip, Runtime, System,
        Treasury, ALICE, BOB, CHARLIE, INITIAL_BALANCE,
    },
    Config, Error, Juror, JurorStatus, Jurors, RequestedJurors, Votes, RESERVE_ID,
};
use frame_support::{
    assert_noop, assert_ok,
    traits::{Hooks, NamedReservableCurrency},
};
use zeitgeist_primitives::{
    constants::BASE,
    traits::DisputeApi,
    types::{
        Market, MarketCreation, MarketDisputeMechanism, MarketPeriod, MarketStatus, MarketType,
        OutcomeReport, ScoringRule,
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
    scoring_rule: ScoringRule::CPMM,
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
fn exit_court_emits_correct_event() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        let who = ALICE;
        assert_ok!(Court::join_court(Origin::signed(who)));
        assert_ok!(Court::exit_court(Origin::signed(who)));
        System::assert_last_event(Event::Court(crate::Event::ExitedJuror(
            who,
            Juror { status: JurorStatus::Ok },
        )));
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
fn exit_court_correctly_removes_requests_for_jurors() {
    ExtBuilder::default().build().execute_with(|| {
        setup_blocks(2);
        assert_ok!(Court::join_court(Origin::signed(ALICE)));
        assert_ok!(Court::on_dispute(&[], &0, &DEFAULT_MARKET));
        assert_ok!(Court::exit_court(Origin::signed(ALICE)));
        // Alice was requested, but can no longer vote.
        assert_noop!(
            Court::vote(Origin::signed(ALICE), 0, OutcomeReport::Scalar(456)),
            Error::<Runtime>::JurorNotRequested
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
        let who = ALICE;
        assert_ok!(Court::join_court(Origin::signed(who)));
        let juror = Juror { status: JurorStatus::Ok };
        assert_eq!(Jurors::<Runtime>::iter().next().unwrap(), (who, juror.clone()));
    });
}

#[test]
fn join_court_emits_correct_signal() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        let who = ALICE;
        assert_ok!(Court::join_court(Origin::signed(who)));
        let juror = Juror { status: JurorStatus::Ok };
        System::assert_last_event(Event::Court(crate::Event::JoinedJuror(who, juror)));
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
        setup_blocks(2);
        assert_ok!(Court::join_court(Origin::signed(ALICE)));
        assert_ok!(Court::join_court(Origin::signed(BOB)));
        assert_ok!(Court::on_dispute(&[], &0, &DEFAULT_MARKET));
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
        setup_blocks(2);
        assert_ok!(Court::join_court(Origin::signed(ALICE)));
        assert_ok!(Court::join_court(Origin::signed(BOB)));
        assert_ok!(Court::join_court(Origin::signed(CHARLIE)));
        assert_ok!(Court::on_dispute(&[], &0, &DEFAULT_MARKET));
        assert_ok!(Court::vote(Origin::signed(ALICE), 0, OutcomeReport::Scalar(1)));
        assert_ok!(Court::vote(Origin::signed(BOB), 0, OutcomeReport::Scalar(2)));
        assert_ok!(Court::vote(Origin::signed(CHARLIE), 0, OutcomeReport::Scalar(3)));
        assert_ok!(Court::on_resolution(&[], &0, &DEFAULT_MARKET));
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
        setup_blocks(2);
        assert_ok!(Court::join_court(Origin::signed(ALICE)));
        assert_ok!(Court::join_court(Origin::signed(BOB)));
        assert_ok!(Court::join_court(Origin::signed(CHARLIE)));
        assert_ok!(Court::on_dispute(&[], &0, &DEFAULT_MARKET));
        assert_ok!(Court::vote(Origin::signed(ALICE), 0, OutcomeReport::Scalar(1)));
        assert_ok!(Court::vote(Origin::signed(BOB), 0, OutcomeReport::Scalar(1)));
        assert_ok!(Court::vote(Origin::signed(CHARLIE), 0, OutcomeReport::Scalar(2)));
        let outcome = Court::on_resolution(&[], &0, &DEFAULT_MARKET).unwrap();
        assert_eq!(outcome, Some(OutcomeReport::Scalar(1)));
    });
}

#[test]
fn on_resolution_sets_late_jurors_as_tardy() {
    ExtBuilder::default().build().execute_with(|| {
        setup_blocks(2);
        assert_ok!(Court::join_court(Origin::signed(ALICE)));
        assert_ok!(Court::join_court(Origin::signed(BOB)));
        assert_ok!(Court::on_dispute(&[], &0, &DEFAULT_MARKET));
        assert_ok!(Court::vote(Origin::signed(ALICE), 0, OutcomeReport::Scalar(1)));
        assert_ok!(Court::on_resolution(&[], &0, &DEFAULT_MARKET));
        assert_eq!(Jurors::<Runtime>::get(ALICE).unwrap().status, JurorStatus::Ok);
        assert_eq!(Jurors::<Runtime>::get(BOB).unwrap().status, JurorStatus::Tardy);
    });
}

#[test]
fn on_resolution_sets_jurors_that_voted_on_the_second_most_voted_outcome_as_tardy() {
    ExtBuilder::default().build().execute_with(|| {
        setup_blocks(2);
        assert_ok!(Court::join_court(Origin::signed(ALICE)));
        assert_ok!(Court::join_court(Origin::signed(BOB)));
        assert_ok!(Court::join_court(Origin::signed(CHARLIE)));
        assert_ok!(Court::on_dispute(&[], &0, &DEFAULT_MARKET));
        assert_ok!(Court::vote(Origin::signed(ALICE), 0, OutcomeReport::Scalar(1)));
        assert_ok!(Court::vote(Origin::signed(BOB), 0, OutcomeReport::Scalar(1)));
        assert_ok!(Court::vote(Origin::signed(CHARLIE), 0, OutcomeReport::Scalar(2)));
        assert_ok!(Court::on_resolution(&[], &0, &DEFAULT_MARKET));
        assert_eq!(Jurors::<Runtime>::get(CHARLIE).unwrap().status, JurorStatus::Tardy);
    });
}

#[test]
fn on_resolution_punishes_tardy_jurors_that_failed_to_vote_a_second_time() {
    ExtBuilder::default().build().execute_with(|| {
        setup_blocks(2);
        assert_ok!(Court::join_court(Origin::signed(ALICE)));
        assert_ok!(Court::join_court(Origin::signed(BOB)));
        assert_ok!(Court::set_stored_juror_as_tardy(&BOB));
        assert_ok!(Court::on_dispute(&[], &0, &DEFAULT_MARKET));
        assert_ok!(Court::vote(Origin::signed(ALICE), 0, OutcomeReport::Scalar(1)));
        assert_ok!(Court::on_resolution(&[], &0, &DEFAULT_MARKET));
        let join_court_stake = 40000000000;
        let slash = join_court_stake / 5;
        assert_eq!(Balances::free_balance(Treasury::account_id()), INITIAL_BALANCE + slash);
        assert_eq!(Balances::free_balance(BOB), INITIAL_BALANCE - slash);
        assert_eq!(Balances::reserved_balance_named(&RESERVE_ID, &BOB), 0);
    });
}

#[test]
fn on_resolution_removes_requested_jurors_and_votes() {
    ExtBuilder::default().build().execute_with(|| {
        setup_blocks(2);
        assert_ok!(Court::join_court(Origin::signed(ALICE)));
        assert_ok!(Court::join_court(Origin::signed(BOB)));
        assert_ok!(Court::join_court(Origin::signed(CHARLIE)));
        assert_ok!(Court::on_dispute(&[], &0, &DEFAULT_MARKET));
        assert_ok!(Court::vote(Origin::signed(ALICE), 0, OutcomeReport::Scalar(1)));
        assert_ok!(Court::vote(Origin::signed(BOB), 0, OutcomeReport::Scalar(1)));
        assert_ok!(Court::vote(Origin::signed(CHARLIE), 0, OutcomeReport::Scalar(2)));
        assert_ok!(Court::on_resolution(&[], &0, &DEFAULT_MARKET));
        assert_eq!(RequestedJurors::<Runtime>::iter().count(), 0);
        assert_eq!(Votes::<Runtime>::iter().count(), 0);
    });
}

#[test]
fn on_resolution_returns_none_if_no_votes_were_cast() {
    ExtBuilder::default().build().execute_with(|| {
        setup_blocks(2);
        assert_ok!(Court::join_court(Origin::signed(ALICE)));
        assert_ok!(Court::join_court(Origin::signed(BOB)));
        assert_ok!(Court::join_court(Origin::signed(CHARLIE)));
        assert_ok!(Court::on_dispute(&[], &0, &DEFAULT_MARKET));
        let outcome = Court::on_resolution(&[], &0, &DEFAULT_MARKET).unwrap();
        assert!(outcome.is_none());
    });
}

#[test]
fn random_jurors_returns_an_unique_different_subset_of_jurors() {
    ExtBuilder::default().build().execute_with(|| {
        setup_blocks(2);

        let mut rng = Court::rng();
        let random_jurors = Court::random_jurors(&DEFAULT_SET_OF_JURORS, 2, &mut rng);
        let mut at_least_one_set_is_different = false;

        for _ in 0..100 {
            setup_blocks(1);

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
        setup_blocks(2);
        let mut rng = Court::rng();
        let random_jurors = Court::random_jurors(&DEFAULT_SET_OF_JURORS, 2, &mut rng);
        for (_, juror) in random_jurors {
            assert!(DEFAULT_SET_OF_JURORS.iter().any(|el| &el.1 == juror));
        }
    });
}

#[test]
fn vote_fails_if_juror_is_not_requested_for_this_market() {
    ExtBuilder::default().build().execute_with(|| {
        setup_blocks(2);
        assert_ok!(Court::join_court(Origin::signed(ALICE)));
        assert_ok!(Court::on_dispute(&[], &0, &DEFAULT_MARKET));
        // Bob joins too late, so he's guaranteed to be requested for the second market, _not_ the
        // first.
        assert_ok!(Court::join_court(Origin::signed(BOB)));
        assert_ok!(Court::on_dispute(&[], &1, &DEFAULT_MARKET));
        assert_noop!(
            Court::vote(Origin::signed(BOB), 0, OutcomeReport::Scalar(456)),
            Error::<Runtime>::JurorNotRequested
        );
    });
}

#[test]
fn vote_fails_if_juror_is_late() {
    ExtBuilder::default().build().execute_with(|| {
        setup_blocks(2);
        assert_ok!(Court::join_court(Origin::signed(ALICE)));
        assert_ok!(Court::on_dispute(&[], &0, &DEFAULT_MARKET));
        let block_count = 1u64 + <Runtime as Config>::CourtCaseDuration::get();
        setup_blocks(block_count.try_into().unwrap());
        assert_noop!(
            Court::vote(Origin::signed(ALICE), 0, OutcomeReport::Scalar(456)),
            Error::<Runtime>::BlockLimitExceeded,
        );
    });
}

#[test]
fn vote_will_store_outcome_from_a_juror() {
    ExtBuilder::default().build().execute_with(|| {
        setup_blocks(2);
        assert_ok!(Court::join_court(Origin::signed(ALICE)));
        assert_ok!(Court::on_dispute(&[], &0, &DEFAULT_MARKET));
        assert_ok!(Court::vote(Origin::signed(ALICE), 0, OutcomeReport::Scalar(345)));
        assert_eq!(Votes::<Runtime>::get(ALICE, 0).unwrap(), (2, OutcomeReport::Scalar(345)));
    });
}

fn setup_blocks(num_blocks: u32) {
    for _ in 0..num_blocks {
        let current_block_number = System::block_number() + 1;
        let parent_block_hash = System::parent_hash();
        let current_digest = System::digest();

        System::initialize(&current_block_number, &parent_block_hash, &current_digest);
        RandomnessCollectiveFlip::on_initialize(current_block_number);
        System::finalize();
        System::set_block_number(current_block_number);
    }
}
