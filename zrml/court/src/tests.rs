#![cfg(test)]

use crate::{
    mock::{Balances, Court, ExtBuilder, Origin, Runtime, ALICE, BOB},
    Error, Juror, JurorStatus, Jurors,
};
use frame_support::{assert_noop, assert_ok};
use zeitgeist_primitives::constants::BASE;

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
