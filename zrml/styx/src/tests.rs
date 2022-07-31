#![cfg(test)]

use crate::{mock::*, Event};
use frame_support::assert_ok;
use zeitgeist_primitives::types::Asset;

#[test]
fn should_emit_account_crossed_event_with_correct_value() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        assert_ok!(Styx::cross(Origin::signed(ALICE)));
        System::assert_last_event(
            Event::AccountCrossed(ALICE, Asset::Ztg, crate::BurnAmount::<Runtime>::get().into())
                .into(),
        );
    });
}

#[test]
fn should_set_burn_amount() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        assert_ok!(Styx::set_burn_amount(Origin::signed(SUDO), 144u128));
        System::assert_last_event(
            Event::CrossingFeeChanged(SUDO, Asset::Ztg, 144u128.into()).into(),
        );
        assert_eq!(crate::BurnAmount::<Runtime>::get(), 144u128.into());
    });
}
