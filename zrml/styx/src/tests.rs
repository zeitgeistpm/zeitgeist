// Copyright 2021-2022 Zeitgeist PM LLC.
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

#![cfg(test)]

use crate::{mock::*, Crossings, Error, Event};
use frame_support::{assert_noop, assert_ok, error::BadOrigin};

#[test]
fn cross_slashes_funds_and_stores_crossing() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        let burn_amount = crate::BurnAmount::<Runtime>::get();
        let original_balance = Balances::free_balance(&ALICE);
        assert_ok!(Styx::cross(RuntimeOrigin::signed(ALICE)));
        let balance_after_crossing = Balances::free_balance(&ALICE);
        let diff = original_balance - balance_after_crossing;
        assert!(Crossings::<Runtime>::contains_key(ALICE));
        assert_eq!(diff, burn_amount);
    });
}

#[test]
fn account_should_only_be_able_to_cross_once() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        assert_ok!(Styx::cross(RuntimeOrigin::signed(ALICE)));
        assert_noop!(
            Styx::cross(RuntimeOrigin::signed(ALICE)),
            Error::<Runtime>::HasAlreadyCrossed
        );
    });
}

#[test]
fn should_set_burn_amount() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        assert_ok!(Styx::set_burn_amount(RuntimeOrigin::signed(SUDO), 144u128));
        System::assert_last_event(Event::CrossingFeeChanged(144u128).into());
        assert_eq!(crate::BurnAmount::<Runtime>::get(), 144u128);
    });
}

#[test]
fn set_burn_amount_should_fail_with_unathorized_caller() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        assert_noop!(Styx::set_burn_amount(RuntimeOrigin::signed(9999), 144u128), BadOrigin);
    });
}

#[test]
fn account_should_not_cross_without_sufficient_funds() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        assert_ok!(Balances::set_balance(RuntimeOrigin::root(), ALICE, 0, 0));
        assert_noop!(
            Styx::cross(RuntimeOrigin::signed(ALICE)),
            Error::<Runtime>::FundDoesNotHaveEnoughFreeBalance
        );
    });
}

#[test]
fn should_emit_account_crossed_event_with_correct_value() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        assert_ok!(Styx::cross(RuntimeOrigin::signed(ALICE)));
        System::assert_last_event(
            Event::AccountCrossed(ALICE, crate::BurnAmount::<Runtime>::get()).into(),
        );
    });
}
