#![cfg(test)]

use crate::{mock::*, Crossings, Error, Event};
use frame_support::{assert_noop, assert_ok, error::BadOrigin};

#[test]
fn cross_slashes_funds_and_stores_crossing() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        let burn_amount = crate::BurnAmount::<Runtime>::get();
        let original_balance = Balances::free_balance(&ALICE);
        assert_ok!(Styx::cross(Origin::signed(ALICE)));
        let balance_after_crossing = Balances::free_balance(&ALICE);
        let diff = original_balance - balance_after_crossing;
        let tx_fee_margin = 5_000_000_000u128;
        assert_eq!(Crossings::<Runtime>::contains_key(&ALICE), true);
        assert!(diff >= burn_amount && diff <= burn_amount + tx_fee_margin);
    });
}

#[test]
fn account_should_only_be_able_to_cross_once() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        assert_ok!(Styx::cross(Origin::signed(ALICE)));
        assert_noop!(Styx::cross(Origin::signed(ALICE)), Error::<Runtime>::HasAlreadyCrossed);
    });
}

#[test]
fn should_set_burn_amount() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        assert_ok!(Styx::set_burn_amount(Origin::signed(SUDO), 144u128));
        System::assert_last_event(Event::CrossingFeeChanged(144u128).into());
        assert_eq!(crate::BurnAmount::<Runtime>::get(), 144u128);
    });
}

#[test]
fn set_burn_amount_should_fail_with_unathorized_caller() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        assert_noop!(Styx::set_burn_amount(Origin::signed(9999), 144u128), BadOrigin);
    });
}

#[test]
fn account_should_not_cross_without_sufficient_funds() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        assert_ok!(Balances::set_balance(Origin::root(), ALICE, 0, 0));
        assert_noop!(
            Styx::cross(Origin::signed(ALICE)),
            Error::<Runtime>::FundDoesNotHaveEnoughFreeBalance
        );
    });
}

#[test]
fn should_emit_account_crossed_event_with_correct_value() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        assert_ok!(Styx::cross(Origin::signed(ALICE)));
        System::assert_last_event(
            Event::AccountCrossed(ALICE, crate::BurnAmount::<Runtime>::get()).into(),
        );
    });
}
