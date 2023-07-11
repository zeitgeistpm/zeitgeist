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

use crate::{mock::*, Error, OrderSide};
use frame_support::{
    assert_noop, assert_ok,
    traits::{Currency, ReservableCurrency},
};
use orml_traits::{MultiCurrency, MultiReservableCurrency};
use zeitgeist_primitives::{
    constants::BASE,
    types::{AccountIdTest, Asset},
};

#[test]
fn it_makes_orders() {
    ExtBuilder::default().build().execute_with(|| {
        // Give some shares for Bob.
        assert_ok!(Tokens::deposit(Asset::CategoricalOutcome(0, 1), &BOB, 100));

        // Make an order from Alice to buy shares.
        assert_ok!(Orderbook::make_order(
            RuntimeOrigin::signed(ALICE),
            Asset::CategoricalOutcome(0, 2),
            OrderSide::Bid,
            25,
            10,
        ));

        let reserved_funds =
            <Balances as ReservableCurrency<AccountIdTest>>::reserved_balance(&ALICE);
        assert_eq!(reserved_funds, 250);

        // Make an order from Bob to sell shares.
        assert_ok!(Orderbook::make_order(
            RuntimeOrigin::signed(BOB),
            Asset::CategoricalOutcome(0, 1),
            OrderSide::Ask,
            10,
            5,
        ));

        let shares_reserved = Tokens::reserved_balance(Asset::CategoricalOutcome(0, 1), &BOB);
        assert_eq!(shares_reserved, 10);
    });
}

#[test]
fn it_takes_orders() {
    ExtBuilder::default().build().execute_with(|| {
        // Give some shares for Bob.
        assert_ok!(Tokens::deposit(Asset::CategoricalOutcome(0, 1), &BOB, 100));

        // Make an order from Bob to sell shares.
        assert_ok!(Orderbook::make_order(
            RuntimeOrigin::signed(BOB),
            Asset::CategoricalOutcome(0, 1),
            OrderSide::Ask,
            10,
            5,
        ));

        let order_hash = Orderbook::order_hash(&BOB, Asset::CategoricalOutcome(0, 1), 0);
        assert_ok!(Orderbook::fill_order(RuntimeOrigin::signed(ALICE), order_hash));

        let alice_bal = <Balances as Currency<AccountIdTest>>::free_balance(&ALICE);
        let alice_shares = Tokens::free_balance(Asset::CategoricalOutcome(0, 1), &ALICE);
        assert_eq!(alice_bal, BASE - 50);
        assert_eq!(alice_shares, 10);

        let bob_bal = <Balances as Currency<AccountIdTest>>::free_balance(&BOB);
        let bob_shares = Tokens::free_balance(Asset::CategoricalOutcome(0, 1), &BOB);
        assert_eq!(bob_bal, BASE + 50);
        assert_eq!(bob_shares, 90);
    });
}

#[test]
fn it_cancels_orders() {
    ExtBuilder::default().build().execute_with(|| {
        // Make an order from Alice to buy shares.
        let share_id = Asset::CategoricalOutcome(0, 2);
        assert_ok!(Orderbook::make_order(
            RuntimeOrigin::signed(ALICE),
            share_id,
            OrderSide::Bid,
            25,
            10
        ));

        let order_hash = Orderbook::order_hash(&ALICE, share_id, 0);

        assert_noop!(
            Orderbook::cancel_order(RuntimeOrigin::signed(BOB), share_id, order_hash),
            Error::<Runtime>::NotOrderCreator,
        );

        assert_ok!(Orderbook::cancel_order(RuntimeOrigin::signed(ALICE), share_id, order_hash));
    });
}
