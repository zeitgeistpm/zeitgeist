// Copyright 2023 Forecasting Technologies LTD.
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

use crate::{mock::*, utils::market_mock, Error, Event, Order, OrderSide, Orders};
use frame_support::{
    assert_noop, assert_ok,
    traits::{Currency, ReservableCurrency},
};
use orml_traits::{MultiCurrency, MultiReservableCurrency};
use sp_runtime::Perbill;
use test_case::test_case;
use zeitgeist_primitives::{
    constants::BASE,
    traits::DistributeFees,
    types::{AccountIdTest, Asset, ScoringRule},
};
use zrml_market_commons::{MarketCommonsPalletApi, Markets};

#[test_case(ScoringRule::CPMM; "CPMM")]
#[test_case(ScoringRule::RikiddoSigmoidFeeMarketEma; "Rikiddo")]
fn place_order_fails_with_wrong_scoring_rule(scoring_rule: ScoringRule) {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0u128;
        let market = market_mock::<Runtime>();
        Markets::<Runtime>::insert(market_id, market);

        assert_ok!(MarketCommons::mutate_market(&market_id, |market| {
            market.scoring_rule = scoring_rule;
            Ok(())
        }));
        assert_noop!(
            Orderbook::place_order(
                RuntimeOrigin::signed(ALICE),
                market_id,
                Asset::CategoricalOutcome(0, 2),
                OrderSide::Bid,
                100,
                250,
            ),
            Error::<Runtime>::InvalidScoringRule
        );
    });
}

#[test_case(ScoringRule::CPMM; "CPMM")]
#[test_case(ScoringRule::RikiddoSigmoidFeeMarketEma; "Rikiddo")]
fn fill_order_fails_with_wrong_scoring_rule(scoring_rule: ScoringRule) {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0u128;
        let market = market_mock::<Runtime>();
        Markets::<Runtime>::insert(market_id, market);

        let order_id = 0u128;

        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(ALICE),
            market_id,
            Asset::CategoricalOutcome(0, 2),
            OrderSide::Bid,
            10,
            250,
        ));

        assert_ok!(MarketCommons::mutate_market(&market_id, |market| {
            market.scoring_rule = scoring_rule;
            Ok(())
        }));

        assert_noop!(
            Orderbook::fill_order(RuntimeOrigin::signed(ALICE), order_id, None),
            Error::<Runtime>::InvalidScoringRule
        );
    });
}

#[test]
fn it_fails_order_does_not_exist() {
    ExtBuilder::default().build().execute_with(|| {
        let order_id = 0u128;
        assert_noop!(
            Orderbook::fill_order(RuntimeOrigin::signed(ALICE), order_id, None),
            Error::<Runtime>::OrderDoesNotExist,
        );

        assert_noop!(
            Orderbook::remove_order(RuntimeOrigin::signed(ALICE), order_id),
            Error::<Runtime>::OrderDoesNotExist,
        );
    });
}

#[test]
fn it_places_orders() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0u128;
        let market = market_mock::<Runtime>();
        Markets::<Runtime>::insert(market_id, market);

        // Give some shares for Bob.
        assert_ok!(AssetManager::deposit(Asset::CategoricalOutcome(0, 1), &BOB, 10));

        // Make an order from Alice to buy shares.
        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(ALICE),
            market_id,
            Asset::CategoricalOutcome(0, 2),
            OrderSide::Bid,
            10,
            250,
        ));

        let reserved_funds =
            <Balances as ReservableCurrency<AccountIdTest>>::reserved_balance(&ALICE);
        assert_eq!(reserved_funds, 250);

        // Make an order from Bob to sell shares.
        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(BOB),
            market_id,
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
fn it_fills_ask_orders_fully() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0u128;
        let market = market_mock::<Runtime>();
        Markets::<Runtime>::insert(market_id, market.clone());

        let outcome_asset = Asset::CategoricalOutcome(0, 1);

        let outcome_asset_amount = 100 * BASE;
        let base_asset_amount = 500 * BASE;
        // Give some shares for Bob.
        assert_ok!(AssetManager::deposit(outcome_asset, &BOB, outcome_asset_amount));

        // Make an order from Bob to sell shares.
        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(BOB),
            market_id,
            outcome_asset,
            OrderSide::Ask,
            outcome_asset_amount,
            base_asset_amount,
        ));

        let reserved_bob = AssetManager::reserved_balance(outcome_asset, &BOB);
        assert_eq!(
            reserved_bob,
            outcome_asset_amount
                - ExternalFees::<Runtime, FeeAccount>::get_fee(market_id, outcome_asset_amount)
        );

        let market_creator_balance_before =
            AssetManager::free_balance(market.base_asset, &MARKET_CREATOR);

        let order_id = 0u128;
        assert_ok!(AssetManager::deposit(market.base_asset, &ALICE, base_asset_amount));
        assert_ok!(Orderbook::fill_order(RuntimeOrigin::signed(ALICE), order_id, None));

        let reserved_bob = AssetManager::reserved_balance(outcome_asset, &BOB);
        assert_eq!(reserved_bob, 0);

        System::assert_last_event(
            Event::<Runtime>::OrderFilled {
                order_id,
                maker: BOB,
                taker: ALICE,
                filled: 500 * BASE,
                unfilled_outcome_asset_amount: 0,
                unfilled_base_asset_amount: 0,
            }
            .into(),
        );

        let alice_bal = AssetManager::free_balance(market.base_asset, &ALICE);
        let alice_shares = AssetManager::free_balance(outcome_asset, &ALICE);
        assert_eq!(alice_bal, INITIAL_BALANCE);
        assert_eq!(alice_shares, 100 * BASE);

        let bob_bal = AssetManager::free_balance(market.base_asset, &BOB);
        let bob_shares = AssetManager::free_balance(outcome_asset, &BOB);
        assert_eq!(bob_bal, INITIAL_BALANCE + 500 * BASE);
        assert_eq!(bob_shares, 90 * BASE);
    });
}

#[test]
fn it_fills_bid_orders_fully() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0u128;
        let market = market_mock::<Runtime>();
        Markets::<Runtime>::insert(market_id, market);

        let outcome_asset = Asset::CategoricalOutcome(0, 1);

        // Make an order from Bob to sell shares.
        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(BOB),
            market_id,
            outcome_asset,
            OrderSide::Bid,
            10,
            50,
        ));

        let reserved_bob = Balances::reserved_balance(BOB);
        assert_eq!(reserved_bob, 50);

        let order_id = 0u128;
        assert_ok!(Tokens::deposit(outcome_asset, &ALICE, 10));
        assert_ok!(Orderbook::fill_order(RuntimeOrigin::signed(ALICE), order_id, None));

        let reserved_bob = Tokens::reserved_balance(outcome_asset, &BOB);
        assert_eq!(reserved_bob, 0);

        System::assert_last_event(
            Event::<Runtime>::OrderFilled {
                order_id,
                maker: BOB,
                taker: ALICE,
                filled: 10,
                unfilled_outcome_asset_amount: 0,
                unfilled_base_asset_amount: 0,
            }
            .into(),
        );

        let alice_bal = <Balances as Currency<AccountIdTest>>::free_balance(&ALICE);
        let alice_shares = Tokens::free_balance(outcome_asset, &ALICE);
        assert_eq!(alice_bal, BASE + 50);
        assert_eq!(alice_shares, 0);

        let bob_bal = <Balances as Currency<AccountIdTest>>::free_balance(&BOB);
        let bob_shares = Tokens::free_balance(outcome_asset, &BOB);
        assert_eq!(bob_bal, BASE - 50);
        assert_eq!(bob_shares, 10);
    });
}

#[test]
fn it_fills_bid_orders_partially() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0u128;
        let market = market_mock::<Runtime>();
        Markets::<Runtime>::insert(market_id, market);

        let outcome_asset = Asset::CategoricalOutcome(0, 1);

        // Make an order from Bob to buy outcome tokens.
        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(BOB),
            market_id,
            outcome_asset,
            OrderSide::Bid,
            1000,
            5000,
        ));

        let reserved_bob = Balances::reserved_balance(BOB);
        assert_eq!(reserved_bob, 5000);

        let order_id = 0u128;
        assert_ok!(Tokens::deposit(outcome_asset, &ALICE, 1000));

        // instead of selling 1000 shares, Alice sells 700 shares
        let portion = Some(700);
        assert_ok!(Orderbook::fill_order(RuntimeOrigin::signed(ALICE), order_id, portion,));

        let order = <Orders<Runtime>>::get(order_id).unwrap();
        assert_eq!(
            order,
            Order {
                market_id,
                side: OrderSide::Bid,
                maker: BOB,
                outcome_asset,
                base_asset: Asset::Ztg,
                // from 1000 to 300 changed (partially filled)
                outcome_asset_amount: 300,
                base_asset_amount: 1500,
            }
        );

        let reserved_bob = Balances::reserved_balance(BOB);
        // 5000 - (700 shares * 500 price) = 1500
        assert_eq!(reserved_bob, 1500);

        System::assert_last_event(
            Event::<Runtime>::OrderFilled {
                order_id,
                maker: BOB,
                taker: ALICE,
                filled: 700,
                unfilled_outcome_asset_amount: 300,
                unfilled_base_asset_amount: 1500,
            }
            .into(),
        );

        let alice_bal = <Balances as Currency<AccountIdTest>>::free_balance(&ALICE);
        let alice_shares = Tokens::free_balance(outcome_asset, &ALICE);
        assert_eq!(alice_bal, BASE + 3500);
        assert_eq!(alice_shares, 300);

        let bob_bal = <Balances as Currency<AccountIdTest>>::free_balance(&BOB);
        let bob_shares = Tokens::free_balance(outcome_asset, &BOB);
        // 3500 of base_asset lost, 1500 of base_asset reserved
        assert_eq!(bob_bal, BASE - 5000);
        assert_eq!(bob_shares, 700);

        let reserved_bob = Balances::reserved_balance(BOB);
        assert_eq!(reserved_bob, 1500);
    });
}

#[test]
fn it_fills_ask_orders_partially() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0u128;
        let market = market_mock::<Runtime>();
        Markets::<Runtime>::insert(market_id, market.clone());

        let outcome_asset = Asset::CategoricalOutcome(0, 1);

        let outcome_asset_amount = 100 * BASE;
        let base_asset_amount = 500 * BASE;

        assert_ok!(Tokens::deposit(outcome_asset, &BOB, outcome_asset_amount));

        // Make an order from Bob to sell outcome tokens.
        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(BOB),
            market_id,
            outcome_asset,
            OrderSide::Ask,
            outcome_asset_amount,
            base_asset_amount,
        ));

        let reserved_bob = Tokens::reserved_balance(outcome_asset, &BOB);
        assert_eq!(
            reserved_bob,
            outcome_asset_amount
                - ExternalFees::<Runtime, FeeAccount>::get_fee(market_id, outcome_asset_amount)
        );

        let order_id = 0u128;
        let market_creator_free_balance_before =
            AssetManager::free_balance(market.base_asset, &MARKET_CREATOR);

        // instead of buying 500 of the base asset, Alice buys 70 shares
        let alice_portion = 70 * BASE;
        let portion = Some(alice_portion);
        assert_ok!(Orderbook::fill_order(RuntimeOrigin::signed(ALICE), order_id, portion,));

        let market_creator_free_balance_after =
            AssetManager::free_balance(market.base_asset, &MARKET_CREATOR);
        assert_eq!(
            market_creator_free_balance_after - market_creator_free_balance_before,
            ExternalFees::<Runtime, FeeAccount>::get_fee(market_id, 70 * BASE)
        );

        let order = <Orders<Runtime>>::get(order_id).unwrap();
        let filled_outcome_asset_without_fees = 860_000_000_000;
        assert_eq!(
            filled_outcome_asset_without_fees,
            outcome_asset_amount
                - Perbill::from_rational(alice_portion, base_asset_amount)
                    .mul_floor(outcome_asset_amount)
        );
        let filled_outcome_asset_amount = filled_outcome_asset_without_fees
            - ExternalFees::<Runtime, FeeAccount>::get_fee(
                market_id,
                filled_outcome_asset_without_fees,
            );
        assert_eq!(
            order,
            Order {
                market_id,
                side: OrderSide::Ask,
                maker: BOB,
                outcome_asset,
                base_asset: Asset::Ztg,
                // from 100 to 86 changed (partially filled) minus fees
                outcome_asset_amount: filled_outcome_asset_amount,
                // from 500 to 430 changed (partially filled)
                base_asset_amount: base_asset_amount - alice_portion,
            }
        );

        let reserved_bob = Tokens::reserved_balance(outcome_asset, &BOB);
        assert_eq!(reserved_bob, filled_outcome_asset_amount);

        System::assert_last_event(
            Event::<Runtime>::OrderFilled {
                order_id,
                maker: BOB,
                taker: ALICE,
                filled: alice_portion,
                unfilled_outcome_asset_amount: filled_outcome_asset_amount,
                unfilled_base_asset_amount: base_asset_amount - alice_portion,
            }
            .into(),
        );

        let alice_bal = AssetManager::free_balance(Asset::Ztg, &ALICE);
        let alice_shares = AssetManager::free_balance(outcome_asset, &ALICE);
        assert_eq!(alice_bal, INITIAL_BALANCE - alice_portion);
        assert_eq!(
            alice_shares,
            Perbill::from_rational(alice_portion, base_asset_amount).mul_floor(
                outcome_asset_amount
                    - ExternalFees::<Runtime, FeeAccount>::get_fee(market_id, outcome_asset_amount)
            )
        );
        assert_eq!(alice_shares, 138_600_000_000);

        let bob_bal = AssetManager::free_balance(Asset::Ztg, &BOB);
        let bob_shares = AssetManager::free_balance(outcome_asset, &BOB);
        let filled_minus_fees =
            alice_portion - ExternalFees::<Runtime, FeeAccount>::get_fee(market_id, alice_portion);
        assert_eq!(bob_bal, INITIAL_BALANCE + filled_minus_fees);
        assert_eq!(
            bob_shares,
            ExternalFees::<Runtime, FeeAccount>::get_fee(market_id, outcome_asset_amount)
        );

        let reserved_bob = AssetManager::reserved_balance(outcome_asset, &BOB);
        assert_eq!(reserved_bob, filled_outcome_asset_amount);
    });
}

#[test]
fn it_removes_orders() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0u128;
        let market = market_mock::<Runtime>();
        Markets::<Runtime>::insert(market_id, market);

        // Make an order from Alice to buy shares.
        let share_id = Asset::CategoricalOutcome(0, 2);
        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(ALICE),
            market_id,
            share_id,
            OrderSide::Bid,
            25,
            10
        ));

        let order_id = 0u128;
        System::assert_last_event(
            Event::<Runtime>::OrderPlaced {
                order_id: 0,
                order: Order {
                    market_id,
                    side: OrderSide::Bid,
                    maker: ALICE,
                    outcome_asset: share_id,
                    base_asset: Asset::Ztg,
                    outcome_asset_amount: 25,
                    base_asset_amount: 10,
                },
            }
            .into(),
        );

        let order = <Orders<Runtime>>::get(order_id).unwrap();
        assert_eq!(
            order,
            Order {
                market_id,
                side: OrderSide::Bid,
                maker: ALICE,
                outcome_asset: share_id,
                base_asset: Asset::Ztg,
                outcome_asset_amount: 25,
                base_asset_amount: 10,
            }
        );

        assert_noop!(
            Orderbook::remove_order(RuntimeOrigin::signed(BOB), order_id),
            Error::<Runtime>::NotOrderCreator,
        );

        let reserved_funds =
            <Balances as ReservableCurrency<AccountIdTest>>::reserved_balance(&ALICE);
        assert_eq!(reserved_funds, 10);

        assert_ok!(Orderbook::remove_order(RuntimeOrigin::signed(ALICE), order_id));

        let reserved_funds =
            <Balances as ReservableCurrency<AccountIdTest>>::reserved_balance(&ALICE);
        assert_eq!(reserved_funds, 0);

        assert!(<Orders<Runtime>>::get(order_id).is_none());

        System::assert_last_event(Event::<Runtime>::OrderRemoved { order_id, maker: ALICE }.into());
    });
}
