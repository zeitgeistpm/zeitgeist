use crate::{mock::*, Error, OrderSide};
use frame_support::{
    assert_noop, assert_ok,
    traits::{Currency, ReservableCurrency},
};
use sp_core::H256;
use zrml_traits::shares::{ReservableShares, Shares as SharesTrait};

#[test]
fn it_makes_orders() {
    ExtBuilder::default().build().execute_with(|| {
        // Give some shares for Bob.
        Shares::set_balance(H256::repeat_byte(1), &BOB, 100);

        // Make an order from Alice to buy shares.
        assert_ok!(Orderbook::make_order(
            Origin::signed(ALICE),
            H256::repeat_byte(2),
            OrderSide::Bid,
            25,
            10,
        ));

        let reserved_funds = <Balances as ReservableCurrency<AccountId>>::reserved_balance(&ALICE);
        assert_eq!(reserved_funds, 250);

        // Make an order from Bob to sell shares.
        assert_ok!(Orderbook::make_order(
            Origin::signed(BOB),
            H256::repeat_byte(1),
            OrderSide::Ask,
            10,
            5,
        ));

        let shares_reserved =
            <Shares as ReservableShares<AccountId, H256>>::reserved_balance(
                H256::repeat_byte(1),
                &BOB,
            );
        assert_eq!(shares_reserved, 10);
    });
}

#[test]
fn it_takes_orders() {
    ExtBuilder::default().build().execute_with(|| {
        // Give some shares for Bob.
        let shares_id = H256::repeat_byte(1);
        Shares::set_balance(shares_id, &BOB, 100);

        // Make an order from Bob to sell shares.
        assert_ok!(Orderbook::make_order(
            Origin::signed(BOB),
            shares_id,
            OrderSide::Ask,
            10,
            5,
        ));

        let order_hash = Orderbook::order_hash(&BOB, shares_id, 0);
        assert_ok!(Orderbook::fill_order(Origin::signed(ALICE), order_hash));

        let alice_bal = <Balances as Currency<AccountId>>::free_balance(&ALICE);
        let alice_shares =
            <Shares as SharesTrait<AccountId, H256>>::free_balance(shares_id, &ALICE);
        assert_eq!(alice_bal, 950);
        assert_eq!(alice_shares, 10);

        let bob_bal = <Balances as Currency<AccountId>>::free_balance(&BOB);
        let bob_shares =
            <Shares as SharesTrait<AccountId, H256>>::free_balance(shares_id, &BOB);
        assert_eq!(bob_bal, 1_050);
        assert_eq!(bob_shares, 90);
    });
}

#[test]
fn it_cancels_orders() {
    ExtBuilder::default().build().execute_with(|| {
        // Make an order from Alice to buy shares.
        let share_id = H256::repeat_byte(2);
        assert_ok!(Orderbook::make_order(
            Origin::signed(ALICE),
            share_id,
            OrderSide::Bid,
            25,
            10,
        ));

        let order_hash = Orderbook::order_hash(&ALICE, share_id, 0);

        assert_noop!(
            Orderbook::cancel_order(Origin::signed(BOB), share_id, order_hash),
            Error::<Test>::NotOrderCreator,
        );

        assert_ok!(Orderbook::cancel_order(
            Origin::signed(ALICE),
            share_id,
            order_hash
        ));
    });
}
