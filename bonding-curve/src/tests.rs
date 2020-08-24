use crate::{Error, mock::*};
use frame_support::{
    assert_noop, assert_ok, traits::OnInitialize,
};

#[test]
fn it_creates_a_new_bonding_curve() {
    ExtBuilder::default().build().execute_with(|| {
        // Make sure we can't create a bonding curve that has already being used.
        assert_noop!(
            BondingCurve::create(
                Origin::signed(ALICE),
                0,
                1,
                100,
            ),
            Error::<Test>::CurrencyAlreadyExists,
        );

        assert_ok!(
            BondingCurve::create(
                Origin::signed(ALICE),
                1,
                1,
                100,
            )
        );
    });
}

