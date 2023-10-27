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
use frame_support::{assert_noop, assert_ok};
use orml_traits::{MultiCurrency, MultiReservableCurrency};
use sp_runtime::{Perbill, Perquintill};
use test_case::test_case;
use zeitgeist_primitives::{
    constants::BASE,
    traits::DistributeFees,
    types::{Asset, ScoringRule},
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
                10 * BASE,
                25 * BASE,
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
            10 * BASE,
            25 * BASE,
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
        Markets::<Runtime>::insert(market_id, market.clone());

        let outcome_asset_0 = Asset::CategoricalOutcome(0, 2);

        let outcome_asset_amount = 10 * BASE;
        let base_asset_amount = 250 * BASE;

        assert_ok!(AssetManager::deposit(market.base_asset, &ALICE, base_asset_amount));

        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(ALICE),
            market_id,
            outcome_asset_0,
            OrderSide::Bid,
            outcome_asset_amount,
            base_asset_amount,
        ));

        let reserved_funds = AssetManager::reserved_balance(market.base_asset, &ALICE);
        let base_asset_fees =
            ExternalFees::<Runtime, FeeAccount>::get_fee(market_id, base_asset_amount);
        let base_asset_minus_fees = base_asset_amount - base_asset_fees;
        assert_eq!(reserved_funds, base_asset_minus_fees);

        let outcome_asset_1 = Asset::CategoricalOutcome(0, 1);

        let outcome_asset_amount = 10 * BASE;
        let base_asset_amount = 5 * BASE;
        assert_ok!(AssetManager::deposit(outcome_asset_1, &BOB, outcome_asset_amount));

        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(BOB),
            market_id,
            outcome_asset_1,
            OrderSide::Ask,
            outcome_asset_amount,
            base_asset_amount,
        ));

        let shares_reserved = AssetManager::reserved_balance(outcome_asset_1, &BOB);
        let outcome_asset_fees =
            ExternalFees::<Runtime, FeeAccount>::get_fee(market_id, outcome_asset_amount);
        let outcome_asset_minus_fees = outcome_asset_amount - outcome_asset_fees;
        assert_eq!(shares_reserved, outcome_asset_minus_fees);
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
        let outcome_asset_fees =
            ExternalFees::<Runtime, FeeAccount>::get_fee(market_id, outcome_asset_amount);
        assert_eq!(reserved_bob, outcome_asset_amount - outcome_asset_fees);

        let market_creator_balance_before =
            AssetManager::free_balance(market.base_asset, &MARKET_CREATOR);

        let order_id = 0u128;
        assert_ok!(AssetManager::deposit(market.base_asset, &ALICE, base_asset_amount));
        assert_ok!(Orderbook::fill_order(RuntimeOrigin::signed(ALICE), order_id, None));

        let market_creator_balance_after =
            AssetManager::free_balance(market.base_asset, &MARKET_CREATOR);
        let base_asset_fees =
            ExternalFees::<Runtime, FeeAccount>::get_fee(market_id, base_asset_amount);
        assert_eq!(market_creator_balance_after - market_creator_balance_before, base_asset_fees);

        let reserved_bob = AssetManager::reserved_balance(outcome_asset, &BOB);
        assert_eq!(reserved_bob, 0);

        System::assert_last_event(
            Event::<Runtime>::OrderFilled {
                order_id,
                maker: BOB,
                taker: ALICE,
                filled: base_asset_amount,
                unfilled_outcome_asset_amount: 0,
                unfilled_base_asset_amount: 0,
            }
            .into(),
        );

        let alice_bal = AssetManager::free_balance(market.base_asset, &ALICE);
        let alice_shares = AssetManager::free_balance(outcome_asset, &ALICE);
        assert_eq!(alice_bal, INITIAL_BALANCE);

        let filled_minus_fees = outcome_asset_amount - outcome_asset_fees;
        assert_eq!(alice_shares, filled_minus_fees);

        let bob_bal = AssetManager::free_balance(market.base_asset, &BOB);
        let bob_shares = AssetManager::free_balance(outcome_asset, &BOB);
        assert_eq!(bob_bal, INITIAL_BALANCE + base_asset_amount - base_asset_fees);
        assert_eq!(bob_shares, outcome_asset_fees);
    });
}

#[test]
fn it_fills_bid_orders_fully() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0u128;
        let market = market_mock::<Runtime>();
        Markets::<Runtime>::insert(market_id, market.clone());

        let outcome_asset = Asset::CategoricalOutcome(0, 1);

        let outcome_asset_amount = 10 * BASE;
        let base_asset_amount = 50 * BASE;

        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(BOB),
            market_id,
            outcome_asset,
            OrderSide::Bid,
            outcome_asset_amount,
            base_asset_amount,
        ));

        let reserved_bob = AssetManager::reserved_balance(market.base_asset, &BOB);
        let base_asset_fees =
            ExternalFees::<Runtime, FeeAccount>::get_fee(market_id, base_asset_amount);
        let base_asset_amount_minus_fees = base_asset_amount - base_asset_fees;
        assert_eq!(reserved_bob, base_asset_amount_minus_fees);

        let outcome_asset_fees =
            ExternalFees::<Runtime, FeeAccount>::get_fee(market_id, outcome_asset_amount);
        let outcome_asset_amount_minus_fees = outcome_asset_amount - outcome_asset_fees;

        let order_id = 0u128;
        assert_ok!(AssetManager::deposit(outcome_asset, &ALICE, outcome_asset_amount_minus_fees));
        assert_ok!(Orderbook::fill_order(RuntimeOrigin::signed(ALICE), order_id, None));

        let reserved_bob = AssetManager::reserved_balance(outcome_asset, &BOB);
        assert_eq!(reserved_bob, 0);

        System::assert_last_event(
            Event::<Runtime>::OrderFilled {
                order_id,
                maker: BOB,
                taker: ALICE,
                filled: outcome_asset_amount_minus_fees,
                unfilled_outcome_asset_amount: 0,
                unfilled_base_asset_amount: 0,
            }
            .into(),
        );

        let alice_bal = AssetManager::free_balance(market.base_asset, &ALICE);
        let alice_shares = AssetManager::free_balance(outcome_asset, &ALICE);
        assert_eq!(alice_bal, INITIAL_BALANCE + base_asset_amount_minus_fees);
        assert_eq!(alice_shares, 0);

        let bob_bal = AssetManager::free_balance(market.base_asset, &BOB);
        let bob_shares = AssetManager::free_balance(outcome_asset, &BOB);
        assert_eq!(bob_bal, INITIAL_BALANCE - base_asset_amount);
        assert_eq!(bob_shares, outcome_asset_amount_minus_fees);
    });
}

#[test]
fn it_fills_bid_orders_partially() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0u128;
        let market = market_mock::<Runtime>();
        Markets::<Runtime>::insert(market_id, market.clone());

        let outcome_asset = Asset::CategoricalOutcome(0, 1);

        let outcome_asset_amount = 100 * BASE;
        let base_asset_amount = 500 * BASE;

        assert_ok!(AssetManager::deposit(market.base_asset, &BOB, base_asset_amount));

        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(BOB),
            market_id,
            outcome_asset,
            OrderSide::Bid,
            outcome_asset_amount,
            base_asset_amount,
        ));

        let reserved_bob = AssetManager::reserved_balance(market.base_asset, &BOB);
        let base_asset_fees = ExternalFees::<Runtime, FeeAccount>::get_fee(market_id, 500 * BASE);
        assert_eq!(reserved_bob, base_asset_amount - base_asset_fees);

        let order_id = 0u128;
        assert_ok!(AssetManager::deposit(outcome_asset, &ALICE, outcome_asset_amount));

        // instead of selling 100 shares, Alice sells 70 shares
        let alice_portion = 70 * BASE;
        assert!(alice_portion < outcome_asset_amount);
        let alice_shares_left = outcome_asset_amount - alice_portion;
        let portion = Some(alice_portion);
        assert_ok!(Orderbook::fill_order(RuntimeOrigin::signed(ALICE), order_id, portion,));

        let order = <Orders<Runtime>>::get(order_id).unwrap();
        let outcome_asset_amount_minus_fees = outcome_asset_amount
            - ExternalFees::<Runtime, FeeAccount>::get_fee(market_id, outcome_asset_amount);
        let unfilled_outcome_asset_amount_minus_fees =
            outcome_asset_amount_minus_fees - alice_portion;

        let base_asset_fees =
            ExternalFees::<Runtime, FeeAccount>::get_fee(market_id, base_asset_amount);
        let base_asset_amount_minus_fees = base_asset_amount - base_asset_fees;
        let unfilled_base_asset_amount_minus_fees = base_asset_amount_minus_fees
            - Perquintill::from_rational(alice_portion, outcome_asset_amount_minus_fees)
                .mul_floor(base_asset_amount_minus_fees);
        assert_eq!(
            order,
            Order {
                market_id,
                side: OrderSide::Bid,
                maker: BOB,
                outcome_asset,
                base_asset: market.base_asset,
                outcome_asset_amount: unfilled_outcome_asset_amount_minus_fees,
                base_asset_amount: unfilled_base_asset_amount_minus_fees,
            }
        );

        let reserved_bob = AssetManager::reserved_balance(market.base_asset, &BOB);
        assert_eq!(reserved_bob, unfilled_base_asset_amount_minus_fees);

        System::assert_last_event(
            Event::<Runtime>::OrderFilled {
                order_id,
                maker: BOB,
                taker: ALICE,
                filled: alice_portion,
                unfilled_outcome_asset_amount: unfilled_outcome_asset_amount_minus_fees,
                unfilled_base_asset_amount: unfilled_base_asset_amount_minus_fees,
            }
            .into(),
        );

        let alice_bal = AssetManager::free_balance(market.base_asset, &ALICE);
        let alice_shares = AssetManager::free_balance(outcome_asset, &ALICE);
        let filled_base_asset_amount =
            Perquintill::from_rational(alice_portion, outcome_asset_amount)
                .mul_floor(base_asset_amount);
        assert_eq!(alice_bal, INITIAL_BALANCE + filled_base_asset_amount - 1);
        assert_eq!(alice_shares, alice_shares_left);

        let bob_bal = AssetManager::free_balance(market.base_asset, &BOB);
        let bob_shares = AssetManager::free_balance(outcome_asset, &BOB);
        assert_eq!(bob_bal, INITIAL_BALANCE);
        assert_eq!(bob_shares, alice_portion);

        let reserved_bob = AssetManager::reserved_balance(market.base_asset, &BOB);
        assert_eq!(reserved_bob, unfilled_base_asset_amount_minus_fees);
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

        assert_ok!(AssetManager::deposit(outcome_asset, &BOB, outcome_asset_amount));

        // Make an order from Bob to sell outcome tokens.
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

        let reserved_bob = AssetManager::reserved_balance(outcome_asset, &BOB);
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
        Markets::<Runtime>::insert(market_id, market.clone());

        let outcome_asset = Asset::CategoricalOutcome(0, 2);

        let outcome_asset_amount = 25 * BASE;
        let base_asset_amount = 10 * BASE;

        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(ALICE),
            market_id,
            outcome_asset,
            OrderSide::Bid,
            outcome_asset_amount,
            base_asset_amount,
        ));

        let outcome_asset_fees =
            ExternalFees::<Runtime, FeeAccount>::get_fee(market_id, outcome_asset_amount);
        let outcome_asset_amount_minus_fees = outcome_asset_amount - outcome_asset_fees;

        let base_asset_fees =
            ExternalFees::<Runtime, FeeAccount>::get_fee(market_id, base_asset_amount);
        let base_asset_amount_minus_fees = base_asset_amount - base_asset_fees;

        let order_id = 0u128;
        System::assert_last_event(
            Event::<Runtime>::OrderPlaced {
                order_id,
                order: Order {
                    market_id,
                    side: OrderSide::Bid,
                    maker: ALICE,
                    outcome_asset,
                    base_asset: Asset::Ztg,
                    outcome_asset_amount: outcome_asset_amount_minus_fees,
                    base_asset_amount: base_asset_amount_minus_fees,
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
                outcome_asset,
                base_asset: Asset::Ztg,
                outcome_asset_amount: outcome_asset_amount_minus_fees,
                base_asset_amount: base_asset_amount_minus_fees,
            }
        );

        assert_noop!(
            Orderbook::remove_order(RuntimeOrigin::signed(BOB), order_id),
            Error::<Runtime>::NotOrderCreator,
        );

        let reserved_funds = AssetManager::reserved_balance(market.base_asset, &ALICE);
        assert_eq!(reserved_funds, base_asset_amount_minus_fees);

        assert_ok!(Orderbook::remove_order(RuntimeOrigin::signed(ALICE), order_id));

        let reserved_funds = AssetManager::reserved_balance(market.base_asset, &ALICE);
        assert_eq!(reserved_funds, 0);

        assert!(<Orders<Runtime>>::get(order_id).is_none());

        System::assert_last_event(Event::<Runtime>::OrderRemoved { order_id, maker: ALICE }.into());
    });
}
