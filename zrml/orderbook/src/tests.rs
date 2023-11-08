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

use crate::{mock::*, utils::market_mock, Error, Event, Order, Orders};
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
        Markets::<Runtime>::insert(market_id, market.clone());

        assert_ok!(MarketCommons::mutate_market(&market_id, |market| {
            market.scoring_rule = scoring_rule;
            Ok(())
        }));
        assert_noop!(
            Orderbook::place_order(
                RuntimeOrigin::signed(ALICE),
                market_id,
                market.base_asset,
                10 * BASE,
                Asset::CategoricalOutcome(0, 2),
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
        Markets::<Runtime>::insert(market_id, market.clone());

        let maker_asset = market.base_asset;
        let taker_asset = Asset::CategoricalOutcome(0, 2);

        let order_id = 0u128;

        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(ALICE),
            market_id,
            maker_asset,
            10 * BASE,
            taker_asset,
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

        let taker_asset_0 = Asset::CategoricalOutcome(0, 2);

        let taker_amount = 10 * BASE;
        let maker_amount = 250 * BASE;

        assert_ok!(AssetManager::deposit(market.base_asset, &ALICE, maker_amount));

        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(ALICE),
            market_id,
            market.base_asset,
            maker_amount,
            taker_asset_0,
            taker_amount,
        ));

        let reserved_funds = AssetManager::reserved_balance(market.base_asset, &ALICE);
        assert_eq!(reserved_funds, maker_amount);

        let maker_asset = Asset::CategoricalOutcome(0, 1);

        let maker_amount = 10 * BASE;
        let taker_amount = 5 * BASE;
        assert_ok!(AssetManager::deposit(maker_asset, &BOB, maker_amount));

        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(BOB),
            market_id,
            maker_asset,
            maker_amount,
            market.base_asset,
            taker_amount,
        ));

        let shares_reserved = AssetManager::reserved_balance(maker_asset, &BOB);
        assert_eq!(shares_reserved, maker_amount);
    });
}

#[test]
fn it_fills_order_fully_maker_outcome_asset() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0u128;
        let market = market_mock::<Runtime>();
        Markets::<Runtime>::insert(market_id, market.clone());

        let maker_asset = Asset::CategoricalOutcome(0, 1);
        let taker_asset = market.base_asset;

        let maker_amount = 100 * BASE;
        let taker_amount = 500 * BASE;
        assert_ok!(AssetManager::deposit(maker_asset, &BOB, maker_amount));

        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(BOB),
            market_id,
            maker_asset,
            maker_amount,
            taker_asset,
            taker_amount,
        ));

        let reserved_bob = AssetManager::reserved_balance(maker_asset, &BOB);
        assert_eq!(reserved_bob, maker_amount);

        let market_creator_balance_before =
            AssetManager::free_balance(taker_asset, &MARKET_CREATOR);

        let order_id = 0u128;
        assert_ok!(AssetManager::deposit(taker_asset, &ALICE, taker_amount));
        assert_ok!(Orderbook::fill_order(RuntimeOrigin::signed(ALICE), order_id, None));

        let market_creator_balance_after = AssetManager::free_balance(taker_asset, &MARKET_CREATOR);
        let taker_fees = ExternalFees::<Runtime, FeeAccount>::get_fee(market_id, taker_amount);
        assert_eq!(market_creator_balance_after - market_creator_balance_before, taker_fees);

        let reserved_bob = AssetManager::reserved_balance(maker_asset, &BOB);
        assert_eq!(reserved_bob, 0);

        System::assert_last_event(
            Event::<Runtime>::OrderFilled {
                order_id,
                maker: BOB,
                taker: ALICE,
                filled_taker_amount: taker_amount,
                unfilled_maker_amount: 0,
                unfilled_taker_amount: 0,
            }
            .into(),
        );

        let alice_maker_asset_free = AssetManager::free_balance(taker_asset, &ALICE);
        let alice_taker_asset_free = AssetManager::free_balance(maker_asset, &ALICE);
        assert_eq!(alice_maker_asset_free, INITIAL_BALANCE);

        assert_eq!(alice_taker_asset_free, maker_amount);

        let bob_taker_asset_free = AssetManager::free_balance(market.base_asset, &BOB);
        let bob_maker_asset_free = AssetManager::free_balance(maker_asset, &BOB);
        assert_eq!(bob_taker_asset_free, INITIAL_BALANCE + taker_amount - taker_fees);
        assert_eq!(bob_maker_asset_free, 0);
    });
}

#[test]
fn it_fills_order_fully_maker_base_asset() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0u128;
        let market = market_mock::<Runtime>();
        Markets::<Runtime>::insert(market_id, market.clone());

        let maker_asset = market.base_asset;
        let taker_asset = Asset::CategoricalOutcome(0, 1);

        let taker_amount = 10 * BASE;
        let maker_amount = 50 * BASE;

        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(BOB),
            market_id,
            maker_asset,
            maker_amount,
            taker_asset,
            taker_amount,
        ));

        let reserved_bob = AssetManager::reserved_balance(maker_asset, &BOB);
        assert_eq!(reserved_bob, maker_amount);

        let order_id = 0u128;
        assert_ok!(AssetManager::deposit(taker_asset, &ALICE, taker_amount));
        assert_ok!(Orderbook::fill_order(RuntimeOrigin::signed(ALICE), order_id, None));

        let reserved_bob = AssetManager::reserved_balance(taker_asset, &BOB);
        assert_eq!(reserved_bob, 0);

        System::assert_last_event(
            Event::<Runtime>::OrderFilled {
                order_id,
                maker: BOB,
                taker: ALICE,
                filled_taker_amount: taker_amount,
                unfilled_taker_amount: 0,
                unfilled_maker_amount: 0,
            }
            .into(),
        );

        let alice_maker_asset_free = AssetManager::free_balance(maker_asset, &ALICE);
        let alice_taker_asset_free = AssetManager::free_balance(taker_asset, &ALICE);
        let maker_fees = ExternalFees::<Runtime, FeeAccount>::get_fee(market_id, maker_amount);
        let maker_amount_minus_fees = maker_amount - maker_fees;
        assert_eq!(alice_maker_asset_free, INITIAL_BALANCE + maker_amount_minus_fees);
        assert_eq!(alice_taker_asset_free, 0);

        let bob_bal = AssetManager::free_balance(maker_asset, &BOB);
        let bob_shares = AssetManager::free_balance(taker_asset, &BOB);
        assert_eq!(bob_bal, INITIAL_BALANCE - maker_amount);
        assert_eq!(bob_shares, taker_amount);
    });
}

#[test]
fn it_fills_order_partially_maker_base_asset() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0u128;
        let market = market_mock::<Runtime>();
        Markets::<Runtime>::insert(market_id, market.clone());

        let maker_asset = market.base_asset;
        let taker_asset = Asset::CategoricalOutcome(0, 1);

        let maker_amount = 500 * BASE;
        let taker_amount = 100 * BASE;

        assert_ok!(AssetManager::deposit(maker_asset, &BOB, maker_amount));

        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(BOB),
            market_id,
            maker_asset,
            maker_amount,
            taker_asset,
            taker_amount,
        ));

        let reserved_bob = AssetManager::reserved_balance(maker_asset, &BOB);
        assert_eq!(reserved_bob, maker_amount);

        let order_id = 0u128;
        assert_ok!(AssetManager::deposit(taker_asset, &ALICE, taker_amount));

        let market_creator_free_before = AssetManager::free_balance(maker_asset, &MARKET_CREATOR);

        // instead of selling 100 shares, Alice sells 70 shares
        let alice_portion = 70 * BASE;
        assert!(alice_portion < taker_amount);
        let alice_taker_asset_free_left = taker_amount - alice_portion;
        let portion = Some(alice_portion);
        assert_ok!(Orderbook::fill_order(RuntimeOrigin::signed(ALICE), order_id, portion,));

        let order = <Orders<Runtime>>::get(order_id).unwrap();
        let unfilled_taker_amount = taker_amount - alice_portion;

        let filled_maker_amount =
            Perquintill::from_rational(alice_portion, taker_amount).mul_floor(maker_amount);
        let unfilled_maker_amount = maker_amount - filled_maker_amount;

        assert_eq!(
            order,
            Order {
                market_id,
                maker: BOB,
                maker_asset,
                maker_amount: unfilled_maker_amount,
                taker_asset,
                taker_amount: unfilled_taker_amount,
            }
        );

        System::assert_last_event(
            Event::<Runtime>::OrderFilled {
                order_id,
                maker: BOB,
                taker: ALICE,
                filled_taker_amount: alice_portion,
                unfilled_maker_amount,
                unfilled_taker_amount,
            }
            .into(),
        );

        let market_creator_free_after = AssetManager::free_balance(maker_asset, &MARKET_CREATOR);
        let maker_fees =
            ExternalFees::<Runtime, FeeAccount>::get_fee(market_id, filled_maker_amount);
        assert_eq!(market_creator_free_after - market_creator_free_before, maker_fees);

        let alice_maker_asset_free = AssetManager::free_balance(maker_asset, &ALICE);
        let alice_taker_asset_free = AssetManager::free_balance(taker_asset, &ALICE);
        let filled_maker_amount =
            Perquintill::from_rational(alice_portion, taker_amount).mul_floor(maker_amount);
        let filled_maker_amount_minus_fees = filled_maker_amount
            - ExternalFees::<Runtime, FeeAccount>::get_fee(market_id, filled_maker_amount);
        assert_eq!(alice_maker_asset_free, INITIAL_BALANCE + filled_maker_amount_minus_fees);
        assert_eq!(alice_taker_asset_free, alice_taker_asset_free_left);

        let bob_maker_asset_free = AssetManager::free_balance(maker_asset, &BOB);
        let bob_taker_asset_free = AssetManager::free_balance(taker_asset, &BOB);
        assert_eq!(bob_maker_asset_free, INITIAL_BALANCE);
        assert_eq!(bob_taker_asset_free, alice_portion);

        let reserved_bob = AssetManager::reserved_balance(maker_asset, &BOB);
        assert_eq!(reserved_bob, unfilled_maker_amount);
    });
}

#[test]
fn it_fills_order_partially_maker_outcome_asset() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0u128;
        let market = market_mock::<Runtime>();
        Markets::<Runtime>::insert(market_id, market.clone());

        let maker_asset = Asset::CategoricalOutcome(0, 1);
        let taker_asset = market.base_asset;

        let maker_amount = 100 * BASE;
        let taker_amount = 500 * BASE;

        assert_ok!(AssetManager::deposit(maker_asset, &BOB, maker_amount));

        // Make an order from Bob to sell outcome tokens.
        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(BOB),
            market_id,
            maker_asset,
            maker_amount,
            taker_asset,
            taker_amount,
        ));

        let reserved_bob = AssetManager::reserved_balance(maker_asset, &BOB);
        assert_eq!(reserved_bob, maker_amount);

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
        let filled_maker_amount = 860_000_000_000;
        assert_eq!(
            order,
            Order {
                market_id,
                maker: BOB,
                maker_asset,
                // from 100 to 86 changed (partially filled) minus fees
                maker_amount: filled_maker_amount,
                taker_asset,
                // from 500 to 430 changed (partially filled)
                taker_amount: taker_amount - alice_portion,
            }
        );

        let reserved_bob = AssetManager::reserved_balance(maker_asset, &BOB);
        assert_eq!(reserved_bob, filled_maker_amount);

        System::assert_last_event(
            Event::<Runtime>::OrderFilled {
                order_id,
                maker: BOB,
                taker: ALICE,
                filled_taker_amount: alice_portion,
                unfilled_maker_amount: filled_maker_amount,
                unfilled_taker_amount: taker_amount - alice_portion,
            }
            .into(),
        );

        let alice_taker_asset_free = AssetManager::free_balance(taker_asset, &ALICE);
        let alice_maker_asset_free = AssetManager::free_balance(maker_asset, &ALICE);
        assert_eq!(alice_taker_asset_free, INITIAL_BALANCE - alice_portion);
        assert_eq!(
            alice_maker_asset_free,
            Perbill::from_rational(alice_portion, taker_amount).mul_floor(maker_amount)
        );
        assert_eq!(alice_maker_asset_free, 140_000_000_000);

        let bob_taker_asset_free = AssetManager::free_balance(taker_asset, &BOB);
        let bob_maker_asset_free = AssetManager::free_balance(maker_asset, &BOB);
        let filled_minus_fees =
            alice_portion - ExternalFees::<Runtime, FeeAccount>::get_fee(market_id, alice_portion);
        assert_eq!(bob_taker_asset_free, INITIAL_BALANCE + filled_minus_fees);
        assert_eq!(bob_maker_asset_free, 0);

        let reserved_bob = AssetManager::reserved_balance(maker_asset, &BOB);
        assert_eq!(reserved_bob, filled_maker_amount);
    });
}

#[test]
fn it_removes_order() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0u128;
        let market = market_mock::<Runtime>();
        Markets::<Runtime>::insert(market_id, market.clone());

        let maker_asset = market.base_asset;
        let taker_asset = Asset::CategoricalOutcome(0, 2);

        let taker_amount = 25 * BASE;
        let maker_amount = 10 * BASE;

        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(ALICE),
            market_id,
            maker_asset,
            maker_amount,
            taker_asset,
            taker_amount,
        ));

        let reserved_funds = AssetManager::reserved_balance(market.base_asset, &ALICE);
        assert_eq!(reserved_funds, maker_amount);

        let order_id = 0u128;
        System::assert_last_event(
            Event::<Runtime>::OrderPlaced {
                order_id,
                order: Order {
                    market_id,
                    maker: ALICE,
                    maker_asset,
                    maker_amount,
                    taker_asset,
                    taker_amount,
                },
            }
            .into(),
        );

        let order = <Orders<Runtime>>::get(order_id).unwrap();
        assert_eq!(
            order,
            Order { market_id, maker: ALICE, maker_asset, maker_amount, taker_asset, taker_amount }
        );

        assert_noop!(
            Orderbook::remove_order(RuntimeOrigin::signed(BOB), order_id),
            Error::<Runtime>::NotOrderCreator,
        );

        let reserved_funds = AssetManager::reserved_balance(market.base_asset, &ALICE);
        assert_eq!(reserved_funds, maker_amount);

        assert_ok!(Orderbook::remove_order(RuntimeOrigin::signed(ALICE), order_id));

        let reserved_funds = AssetManager::reserved_balance(market.base_asset, &ALICE);
        assert_eq!(reserved_funds, 0);

        assert!(<Orders<Runtime>>::get(order_id).is_none());

        System::assert_last_event(Event::<Runtime>::OrderRemoved { order_id, maker: ALICE }.into());
    });
}
