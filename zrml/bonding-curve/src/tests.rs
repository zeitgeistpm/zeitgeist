use crate::{Error, mock::*};
use frame_support::{
    assert_noop, assert_ok, traits::OnInitialize,
};
use orml_traits::MultiCurrency;

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

        let bc = BondingCurve::curves(0).unwrap();
        assert_eq!(bc.creator, ALICE);
        assert_eq!(bc.currency_id, 1);
        assert_eq!(bc.exponent, 1);
        assert_eq!(bc.slope, 100);

        // TODO: Ensure funds are reserved.
    });
}

#[test]
fn it_can_buy_from_a_bonding_curve() {
    ExtBuilder::default().build().execute_with(|| {
        // Alice creates a bonding curve.
        assert_ok!(
            BondingCurve::create(
                Origin::signed(ALICE),
                1,
                1,
                100,
            )
        );

        // Bob buys 2 tokens.
        assert_ok!(
            BondingCurve::buy(
                Origin::signed(BOB),
                0,
                2,
            )
        );

        let native_balance = Tokens::free_balance(0, &BOB);
        let token_balance = Tokens::free_balance(1, &BOB);
        let diff = 1_000 - native_balance;
        assert_eq!(diff, 200);
        assert_eq!(token_balance, 2);
    });
}

#[test]
fn it_can_sell_to_a_bonding_curve() {
    ExtBuilder::default().build().execute_with(|| {
        // Alice creates a bonding curve.
        assert_ok!(
            BondingCurve::create(
                Origin::signed(ALICE),
                1,
                1,
                100,
            )
        );

        // Bob buys 2 tokens.
        assert_ok!(
            BondingCurve::buy(
                Origin::signed(BOB),
                0,
                2,
            )
        );

        // First Bob tries to sell too many tokens.
        assert_noop!(
            BondingCurve::sell(
                Origin::signed(BOB),
                0,
                3,
            ),
            orml_tokens::Error::<Test>::BalanceTooLow,
        );
        // Bob sells 1 token back.
        assert_ok!(
            BondingCurve::sell(
                Origin::signed(BOB),
                0,
                1,
            )
        );
    });
}
