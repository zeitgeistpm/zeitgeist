// Copyright 2024 Forecasting Technologies LTD.
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

use super::*;
use frame_support::traits::fungibles::Unbalanced;

#[test]
fn sell_to_amm_and_then_fill_specified_order() {
    ExtBuilder::default().build().execute_with(|| {
        let liquidity = _10;
        let pivot = _1_100;
        let spot_prices = vec![_1_2 + pivot, _1_2 - pivot];
        let swap_fee = CENT;
        let asset_count = 2u16;
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Categorical(asset_count),
            liquidity,
            spot_prices.clone(),
            swap_fee,
        );

        let asset = Assets::CategoricalOutcome(market_id, 0);
        let amount_in = _2;

        let order_maker_amount = _6;
        let order_taker_amount = _12;
        assert_ok!(AssetManager::deposit(BASE_ASSET, &CHARLIE, order_maker_amount));
        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(CHARLIE),
            market_id,
            BASE_ASSET,
            order_maker_amount,
            asset,
            order_taker_amount,
        ));

        let order_ids = Orders::<Runtime>::iter().map(|(k, _)| k).collect::<Vec<_>>();

        assert_ok!(AssetRouter::increase_balance(asset, &ALICE, amount_in, Precision::Exact));

        let min_price = _1_4.saturated_into::<BalanceOf<Runtime>>();
        let strategy = Strategy::LimitOrder;
        assert_ok!(HybridRouter::sell(
            RuntimeOrigin::signed(ALICE),
            market_id,
            asset_count,
            asset,
            amount_in,
            min_price,
            order_ids,
            strategy,
        ));

        let amm_amount_in = 5608094330;
        System::assert_has_event(
            NeoSwapsEvent::<Runtime>::SellExecuted {
                who: ALICE,
                market_id,
                asset_in: asset,
                amount_in: amm_amount_in,
                amount_out: 2775447716,
                swap_fee_amount: 28320895,
                external_fee_amount: 28320895,
            }
            .into(),
        );

        let order_ids = Orders::<Runtime>::iter().map(|(k, _)| k).collect::<Vec<_>>();
        assert_eq!(order_ids.len(), 1);
        let order = Orders::<Runtime>::get(order_ids[0]).unwrap();
        let unfilled_base_asset_amount = 105608094330;
        assert_eq!(
            order,
            Order {
                market_id,
                maker: CHARLIE,
                maker_asset: BASE_ASSET,
                maker_amount: 52804047165,
                taker_asset: Assets::CategoricalOutcome(market_id, 0),
                taker_amount: unfilled_base_asset_amount,
            }
        );
        let filled_base_asset_amount = order_taker_amount - unfilled_base_asset_amount;
        assert_eq!(filled_base_asset_amount, 14391905670);
        assert_eq!(amm_amount_in + filled_base_asset_amount, amount_in);
    });
}

#[test]
fn sell_to_amm_if_specified_order_has_lower_prices_than_the_amm() {
    ExtBuilder::default().build().execute_with(|| {
        let liquidity = _10;
        let spot_prices = vec![_9_10, _1_10];
        let swap_fee = CENT;
        let asset_count = 2u16;
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Categorical(asset_count),
            liquidity,
            spot_prices.clone(),
            swap_fee,
        );

        let asset = Assets::CategoricalOutcome(market_id, 0);
        let amount = _1;

        let order_maker_amount = _1;
        let order_taker_amount = _2;
        assert_ok!(AssetManager::deposit(BASE_ASSET, &CHARLIE, order_maker_amount));
        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(CHARLIE),
            market_id,
            BASE_ASSET,
            order_maker_amount,
            asset,
            order_taker_amount,
        ));

        let order_ids = Orders::<Runtime>::iter().map(|(k, _)| k).collect::<Vec<_>>();

        assert_ok!(AssetRouter::increase_balance(asset, &ALICE, amount, Precision::Exact));

        let min_price = _1_4.saturated_into::<BalanceOf<Runtime>>();
        let strategy = Strategy::LimitOrder;
        assert_ok!(HybridRouter::sell(
            RuntimeOrigin::signed(ALICE),
            market_id,
            asset_count,
            asset,
            amount,
            min_price,
            order_ids,
            strategy,
        ));

        let order_ids = Orders::<Runtime>::iter().map(|(k, _)| k).collect::<Vec<_>>();
        assert_eq!(order_ids.len(), 1);
        let order = Orders::<Runtime>::get(order_ids[0]).unwrap();
        let order_price = order.price(BASE_ASSET).unwrap();
        assert_eq!(order_price, _1_2);
        assert_eq!(
            order,
            Order {
                market_id,
                maker: CHARLIE,
                maker_asset: BASE_ASSET,
                maker_amount: _1,
                taker_asset: Assets::CategoricalOutcome(market_id, 0),
                taker_amount: _2,
            }
        );
    });
}

#[test]
fn sell_fill_multiple_orders_if_amm_spot_price_lower_than_order_prices() {
    ExtBuilder::default().build().execute_with(|| {
        let liquidity = _10;
        let spot_prices = vec![_1_2 - 1, _1_2 + 1];
        let swap_fee = CENT;
        let asset_count = 2u16;
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Categorical(asset_count),
            liquidity,
            spot_prices.clone(),
            swap_fee,
        );

        let asset = Assets::CategoricalOutcome(market_id, 0);
        let amount_in = _2;

        let order_maker_amount = _1_2;
        let order_taker_amount = _1;
        assert_ok!(AssetManager::deposit(BASE_ASSET, &CHARLIE, 2 * order_maker_amount));
        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(CHARLIE),
            market_id,
            BASE_ASSET,
            order_maker_amount,
            asset,
            order_taker_amount,
        ));
        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(CHARLIE),
            market_id,
            BASE_ASSET,
            order_maker_amount,
            asset,
            order_taker_amount,
        ));

        let order_ids = Orders::<Runtime>::iter().map(|(k, _)| k).collect::<Vec<_>>();

        assert_ok!(AssetRouter::increase_balance(asset, &ALICE, amount_in, Precision::Exact));

        let min_price = _1_4.saturated_into::<BalanceOf<Runtime>>();
        let strategy = Strategy::LimitOrder;
        assert_ok!(HybridRouter::sell(
            RuntimeOrigin::signed(ALICE),
            market_id,
            asset_count,
            asset,
            amount_in,
            min_price,
            order_ids,
            strategy,
        ));

        let order_ids = Orders::<Runtime>::iter().map(|(k, _)| k).collect::<Vec<_>>();
        assert_eq!(order_ids.len(), 0);
    });
}

#[test]
fn sell_fill_specified_order_partially_if_amm_spot_price_lower() {
    ExtBuilder::default().build().execute_with(|| {
        let liquidity = _10;
        let spot_prices = vec![_1_2 - 1, _1_2 + 1];
        let swap_fee = CENT;
        let asset_count = 2u16;
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Categorical(asset_count),
            liquidity,
            spot_prices.clone(),
            swap_fee,
        );

        let asset = Assets::CategoricalOutcome(market_id, 0);
        let amount = _2;

        let order_maker_amount = _4;
        let order_taker_amount = _8;
        assert_ok!(AssetManager::deposit(BASE_ASSET, &CHARLIE, order_maker_amount));
        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(CHARLIE),
            market_id,
            BASE_ASSET,
            order_maker_amount,
            asset,
            order_taker_amount,
        ));

        let order_ids = Orders::<Runtime>::iter().map(|(k, _)| k).collect::<Vec<_>>();
        assert_eq!(order_ids.len(), 1);
        let order_id = order_ids[0];

        assert_ok!(AssetRouter::increase_balance(asset, &ALICE, amount, Precision::Exact));

        let min_price = _1_4.saturated_into::<BalanceOf<Runtime>>();
        let orders = vec![order_id];
        let strategy = Strategy::LimitOrder;
        assert_ok!(HybridRouter::sell(
            RuntimeOrigin::signed(ALICE),
            market_id,
            asset_count,
            asset,
            amount,
            min_price,
            orders,
            strategy,
        ));

        let order = Orders::<Runtime>::get(order_id).unwrap();
        assert_eq!(
            order,
            Order {
                market_id,
                maker: CHARLIE,
                maker_asset: BASE_ASSET,
                maker_amount: _3,
                taker_asset: Assets::CategoricalOutcome(market_id, 0),
                taker_amount: _6,
            }
        );
    });
}

#[test]
fn sell_fails_if_asset_not_equal_to_order_book_taker_asset() {
    ExtBuilder::default().build().execute_with(|| {
        let liquidity = _10;
        let spot_prices = vec![_1_2, _1_2];
        let swap_fee = CENT;
        let asset_count = 2u16;
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Categorical(asset_count),
            liquidity,
            spot_prices.clone(),
            swap_fee,
        );

        let asset = Assets::CategoricalOutcome(market_id, 0);
        let amount_in = _2;

        let maker_amount = _1;
        assert_ok!(AssetRouter::increase_balance(asset, &CHARLIE, maker_amount, Precision::Exact));

        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(CHARLIE),
            market_id,
            asset,
            maker_amount,
            BASE_ASSET,
            amount_in,
        ));

        let order_ids = Orders::<Runtime>::iter().map(|(k, _)| k).collect::<Vec<_>>();
        assert_eq!(order_ids.len(), 1);
        let order_id = order_ids[0];

        assert_ok!(AssetRouter::increase_balance(asset, &ALICE, amount_in, Precision::Exact));

        let min_price = _1_4.saturated_into::<BalanceOf<Runtime>>();
        let orders = vec![order_id];
        let strategy = Strategy::LimitOrder;
        assert_noop!(
            HybridRouter::sell(
                RuntimeOrigin::signed(ALICE),
                market_id,
                asset_count,
                asset,
                amount_in,
                min_price,
                orders,
                strategy,
            ),
            Error::<Runtime>::AssetNotEqualToOrderbookTakerAsset
        );
    });
}

#[test]
fn sell_fails_if_order_price_below_min_price() {
    ExtBuilder::default().build().execute_with(|| {
        let liquidity = _10;
        let spot_prices = vec![_1_2, _1_2];
        let swap_fee = CENT;
        let asset_count = 2u16;
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Categorical(asset_count),
            liquidity,
            spot_prices.clone(),
            swap_fee,
        );

        let asset = Assets::CategoricalOutcome(market_id, 0);
        let amount = _2;

        let order_maker_amount = _1;
        assert_ok!(AssetManager::deposit(BASE_ASSET, &CHARLIE, order_maker_amount));
        assert_ok!(Orderbook::place_order(
            RuntimeOrigin::signed(CHARLIE),
            market_id,
            BASE_ASSET,
            order_maker_amount,
            Assets::CategoricalOutcome(market_id, 0),
            amount,
        ));

        let order_ids = Orders::<Runtime>::iter().map(|(k, _)| k).collect::<Vec<_>>();
        assert_eq!(order_ids.len(), 1);
        let order_id = order_ids[0];

        assert_ok!(AssetRouter::increase_balance(asset, &ALICE, 5 * amount, Precision::Exact));

        let min_price = _3_4.saturated_into::<BalanceOf<Runtime>>();
        let orders = vec![order_id];
        let strategy = Strategy::LimitOrder;
        assert_noop!(
            HybridRouter::sell(
                RuntimeOrigin::signed(ALICE),
                market_id,
                asset_count,
                asset,
                amount,
                min_price,
                orders,
                strategy,
            ),
            Error::<Runtime>::OrderPriceBelowMinPrice
        );
    });
}

#[test]
fn sell_to_amm() {
    ExtBuilder::default().build().execute_with(|| {
        let liquidity = _10;
        let spot_prices = vec![_1_2, _1_2];
        let swap_fee = CENT;
        let asset_count = 2u16;
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Categorical(asset_count),
            liquidity,
            spot_prices.clone(),
            swap_fee,
        );

        let asset = Assets::CategoricalOutcome(market_id, 0);
        let amount = _2;

        assert_ok!(AssetRouter::increase_balance(asset, &ALICE, amount, Precision::Exact));

        let min_price = _1_4.saturated_into::<BalanceOf<Runtime>>();
        let orders = vec![];
        let strategy = Strategy::LimitOrder;
        assert_ok!(HybridRouter::sell(
            RuntimeOrigin::signed(ALICE),
            market_id,
            asset_count,
            asset,
            amount,
            min_price,
            orders,
            strategy,
        ));

        System::assert_has_event(
            NeoSwapsEvent::<Runtime>::SellExecuted {
                who: ALICE,
                market_id,
                asset_in: asset,
                amount_in: 20000000000,
                amount_out: 9460629504,
                swap_fee_amount: 96537036,
                external_fee_amount: 96537035,
            }
            .into(),
        );
    });
}

#[test]
fn sell_min_price_higher_than_amm_spot_price_results_in_place_order() {
    ExtBuilder::default().build().execute_with(|| {
        let liquidity = _10;
        let spot_prices = vec![_1_2 - 1u128, _1_2 + 1u128];
        let swap_fee = CENT;
        let asset_count = 2u16;
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Categorical(asset_count),
            liquidity,
            spot_prices.clone(),
            swap_fee,
        );
        let market = Markets::<Runtime>::get(market_id).unwrap();
        let base_asset = market.base_asset;

        let asset = Assets::CategoricalOutcome(market_id, 0);
        let amount = _2;

        assert_ok!(AssetRouter::increase_balance(asset, &ALICE, amount, Precision::Exact));

        //*  spot price of the AMM is 1 smaller than the min_price
        //*  this results in no sell on the AMM, but places an order on the order book
        let min_price = (_1_2).saturated_into::<BalanceOf<Runtime>>();
        let orders = vec![];
        let strategy = Strategy::LimitOrder;
        assert_ok!(HybridRouter::sell(
            RuntimeOrigin::signed(ALICE),
            market_id,
            asset_count,
            asset,
            amount,
            min_price,
            orders,
            strategy,
        ));

        let order_keys = Orders::<Runtime>::iter().map(|(k, _)| k).collect::<Vec<_>>();
        assert_eq!(order_keys.len(), 1);
        let order_id = order_keys[0];
        let order = Orders::<Runtime>::get(order_id).unwrap();
        assert_eq!(
            order,
            Order {
                market_id,
                maker: ALICE,
                maker_asset: asset,
                maker_amount: _2,
                taker_asset: base_asset.into(),
                taker_amount: _1,
            }
        );
    });
}

#[test]
fn sell_to_amm_but_low_amount() {
    ExtBuilder::default().build().execute_with(|| {
        let liquidity = _10;
        let spot_prices = vec![_1_2, _1_2];
        let swap_fee = CENT;
        let asset_count = 2u16;
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Categorical(asset_count),
            liquidity,
            spot_prices.clone(),
            swap_fee,
        );
        let market = Markets::<Runtime>::get(market_id).unwrap();
        let base_asset = market.base_asset;

        let asset = Assets::CategoricalOutcome(market_id, 0);
        let amount_in = _2;

        assert_ok!(AssetRouter::increase_balance(asset, &ALICE, amount_in, Precision::Exact));

        //*  min_price is just 1 smaller than the spot price of the AMM
        //*  this results in a low sell amount_in on the AMM
        let min_price = (_1_2 - 1u128).saturated_into::<BalanceOf<Runtime>>();
        let orders = vec![];
        let strategy = Strategy::LimitOrder;
        assert_ok!(HybridRouter::sell(
            RuntimeOrigin::signed(ALICE),
            market_id,
            asset_count,
            asset,
            amount_in,
            min_price,
            orders,
            strategy,
        ));

        System::assert_has_event(
            NeoSwapsEvent::<Runtime>::SellExecuted {
                who: ALICE,
                market_id,
                asset_in: asset,
                amount_in: 58,
                amount_out: 29,
                swap_fee_amount: 0,
                external_fee_amount: 0,
            }
            .into(),
        );

        let order_keys = Orders::<Runtime>::iter().map(|(k, _)| k).collect::<Vec<_>>();
        assert_eq!(order_keys.len(), 1);
        let order_id = order_keys[0];
        let order = Orders::<Runtime>::get(order_id).unwrap();
        assert_eq!(
            order,
            Order {
                market_id,
                maker: ALICE,
                maker_asset: asset,
                maker_amount: 19999999942,
                taker_asset: base_asset.into(),
                taker_amount: 9999999969,
            }
        );
    });
}

#[test]
fn sell_succeeds_for_numerical_soft_failure() {
    ExtBuilder::default().build().execute_with(|| {
        let liquidity = _10;
        let spot_prices = vec![_9_10, _1_10];
        let swap_fee = CENT;
        let asset_count = 2u16;
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Categorical(asset_count),
            liquidity,
            spot_prices.clone(),
            swap_fee,
        );

        let asset = Assets::CategoricalOutcome(market_id, 0);
        let amount_in = _1000 * 100;

        // increase_balance does not set total issuance
        AssetRouter::set_total_issuance(asset, amount_in);
        assert_ok!(AssetRouter::increase_balance(asset, &ALICE, amount_in, Precision::Exact));

        let min_price = (_1_100 / 1000).saturated_into::<BalanceOf<Runtime>>();
        let orders = vec![];
        let strategy = Strategy::LimitOrder;
        assert_ok!(HybridRouter::sell(
            RuntimeOrigin::signed(ALICE),
            market_id,
            asset_count,
            asset,
            amount_in,
            min_price,
            orders,
            strategy,
        ));

        let order = Orders::<Runtime>::get(0).unwrap();
        assert_eq!(
            order,
            Order {
                market_id,
                maker: ALICE,
                maker_asset: Assets::CategoricalOutcome(market_id, 0),
                maker_amount: _1000 * 100,
                taker_asset: BASE_ASSET,
                taker_amount: _1,
            }
        );
    });
}

#[test]
fn sell_to_amm_only() {
    ExtBuilder::default().build().execute_with(|| {
        let liquidity = _10;
        let spot_prices = vec![_1_2, _1_2];
        let swap_fee = CENT;
        let asset_count = 2u16;
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Categorical(asset_count),
            liquidity,
            spot_prices.clone(),
            swap_fee,
        );

        let asset = Assets::CategoricalOutcome(market_id, 0);
        let amount = _2;

        assert_ok!(AssetRouter::increase_balance(asset, &ALICE, amount, Precision::Exact));

        let min_price = _1_4.saturated_into::<BalanceOf<Runtime>>();
        let orders = vec![];
        let strategy = Strategy::ImmediateOrCancel;
        assert_ok!(HybridRouter::sell(
            RuntimeOrigin::signed(ALICE),
            market_id,
            asset_count,
            asset,
            amount,
            min_price,
            orders,
            strategy,
        ));

        System::assert_has_event(
            NeoSwapsEvent::<Runtime>::SellExecuted {
                who: ALICE,
                market_id,
                asset_in: asset,
                amount_in: 20000000000,
                amount_out: 9460629504,
                swap_fee_amount: 96537036,
                external_fee_amount: 96537035,
            }
            .into(),
        );

        let order_keys = Orders::<Runtime>::iter().map(|(k, _)| k).collect::<Vec<_>>();
        assert_eq!(order_keys.len(), 0);
    });
}

#[test]
fn sell_places_limit_order_no_pool() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        let mut market = market_mock::<Runtime>(MARKET_CREATOR);
        let base_asset = market.base_asset;
        let required_asset_count = match &market.market_type {
            MarketType::Scalar(_) => panic!("Categorical market type is expected!"),
            MarketType::Categorical(categories) => *categories,
        };
        market.status = MarketStatus::Active;
        Markets::<Runtime>::insert(market_id, market);

        let asset_count = required_asset_count;
        let asset = Assets::CategoricalOutcome(market_id, 0);
        let amount = 10 * BASE;

        assert_ok!(AssetRouter::increase_balance(asset, &ALICE, amount, Precision::Exact));

        let min_price = (BASE / 2).saturated_into::<BalanceOf<Runtime>>();
        let orders = vec![];
        let strategy = Strategy::LimitOrder;
        assert_ok!(HybridRouter::sell(
            RuntimeOrigin::signed(ALICE),
            market_id,
            asset_count,
            asset,
            amount,
            min_price,
            orders,
            strategy,
        ));

        let order_keys = Orders::<Runtime>::iter().map(|(k, _)| k).collect::<Vec<_>>();
        assert_eq!(order_keys.len(), 1);
        let order_id = order_keys[0];
        let order = Orders::<Runtime>::get(order_id).unwrap();
        assert_eq!(
            order,
            Order {
                market_id,
                maker: ALICE,
                maker_asset: asset,
                maker_amount: 10 * BASE,
                taker_asset: base_asset.into(),
                taker_amount: 5 * BASE,
            }
        );
    });
}

#[test]
fn sell_fails_if_balance_too_low() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        let mut market = market_mock::<Runtime>(MARKET_CREATOR);
        let required_asset_count = match &market.market_type {
            MarketType::Scalar(_) => panic!("Categorical market type is expected!"),
            MarketType::Categorical(categories) => *categories,
        };
        market.status = MarketStatus::Active;
        Markets::<Runtime>::insert(market_id, market);

        let asset_count = required_asset_count;
        let asset = Assets::CategoricalOutcome(market_id, 0);
        let amount = 10 * BASE;

        assert_ok!(AssetRouter::increase_balance(asset, &ALICE, amount - 1, Precision::Exact));

        let min_price = (BASE / 2).saturated_into::<BalanceOf<Runtime>>();
        let orders = vec![];
        let strategy = Strategy::ImmediateOrCancel;
        assert_noop!(
            HybridRouter::sell(
                RuntimeOrigin::signed(ALICE),
                market_id,
                asset_count,
                asset,
                amount,
                min_price,
                orders,
                strategy,
            ),
            TokensError::<Runtime>::BalanceTooLow
        );
    });
}

#[test]
fn sell_emits_event() {
    ExtBuilder::default().build().execute_with(|| {
        let liquidity = _10;
        let pivot = _1_100;
        let spot_prices = vec![_1_2 + pivot, _1_2 - pivot];
        let swap_fee = CENT;
        let asset_count = 2u16;
        let market_id = create_market_and_deploy_pool(
            ALICE,
            BASE_ASSET,
            MarketType::Categorical(asset_count),
            liquidity,
            spot_prices.clone(),
            swap_fee,
        );

        let asset = Assets::CategoricalOutcome(market_id, 0);
        let amount_in = _1000 * 100;

        let min_price = _1_100.saturated_into::<BalanceOf<Runtime>>();
        let orders = (0u128..50u128).collect::<Vec<_>>();
        let maker_asset = BASE_ASSET;
        let maker_amount: BalanceOf<Runtime> = _9.saturated_into();
        let taker_asset = asset;
        let taker_amount = _100.saturated_into::<BalanceOf<Runtime>>();
        for (i, _) in orders.iter().enumerate() {
            let order_creator = i as AccountIdTest;
            let surplus = ((i + 1) as u128) * _1_2;
            let taker_amount = taker_amount + surplus.saturated_into::<BalanceOf<Runtime>>();
            assert_ok!(AssetManager::deposit(maker_asset, &order_creator, maker_amount));
            assert_ok!(Orderbook::place_order(
                RuntimeOrigin::signed(order_creator),
                market_id,
                maker_asset,
                maker_amount,
                taker_asset,
                taker_amount,
            ));
        }

        let order_ids = Orders::<Runtime>::iter().map(|(k, _)| k).collect::<Vec<_>>();

        // increase_balance does not set total issuance
        AssetRouter::set_total_issuance(asset, amount_in);
        assert_ok!(AssetRouter::increase_balance(asset, &ALICE, amount_in, Precision::Exact));

        let strategy = Strategy::LimitOrder;
        assert_ok!(HybridRouter::sell(
            RuntimeOrigin::signed(ALICE),
            market_id,
            asset_count,
            asset,
            amount_in,
            min_price,
            order_ids,
            strategy,
        ));

        System::assert_last_event(
            Event::<Runtime>::HybridRouterExecuted {
                tx_type: TxType::Sell,
                who: ALICE,
                market_id,
                price_limit: min_price,
                asset_in: asset,
                amount_in,
                asset_out: BASE_ASSET,
                amount_out: 4551619284973,
                external_fee_amount: 45985911066,
                swap_fee_amount: 985911072,
            }
            .into(),
        );
    });
}

#[test]
fn sell_fails_if_asset_count_mismatch() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        let mut market = market_mock::<Runtime>(MARKET_CREATOR);
        let required_asset_count = match &market.market_type {
            MarketType::Scalar(_) => panic!("Categorical market type is expected!"),
            MarketType::Categorical(categories) => *categories,
        };
        market.status = MarketStatus::Active;
        Markets::<Runtime>::insert(market_id, market);

        let asset_count = 2;
        assert_ne!(required_asset_count, asset_count);
        let asset = Assets::CategoricalOutcome(market_id, 0);

        let amount_in = 2 * BASE;
        assert_ok!(AssetRouter::increase_balance(asset, &ALICE, amount_in, Precision::Exact));

        let max_price = (BASE / 2).saturated_into::<BalanceOf<Runtime>>();
        let orders = vec![];
        let strategy = Strategy::ImmediateOrCancel;
        assert_noop!(
            HybridRouter::sell(
                RuntimeOrigin::signed(ALICE),
                market_id,
                asset_count,
                asset,
                amount_in,
                max_price,
                orders,
                strategy,
            ),
            Error::<Runtime>::AssetCountMismatch
        );
    });
}

#[test]
fn sell_fails_if_amount_is_zero() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        let mut market = market_mock::<Runtime>(MARKET_CREATOR);
        let required_asset_count = match &market.market_type {
            MarketType::Scalar(_) => panic!("Categorical market type is expected!"),
            MarketType::Categorical(categories) => *categories,
        };
        market.status = MarketStatus::Active;
        Markets::<Runtime>::insert(market_id, market);

        let asset_count = required_asset_count;
        let asset = Assets::CategoricalOutcome(market_id, 0);
        let amount_in = 0;
        let max_price = (BASE / 2).saturated_into::<BalanceOf<Runtime>>();
        let orders = vec![];
        let strategy = Strategy::LimitOrder;
        assert_noop!(
            HybridRouter::sell(
                RuntimeOrigin::signed(ALICE),
                market_id,
                asset_count,
                asset,
                amount_in,
                max_price,
                orders,
                strategy,
            ),
            Error::<Runtime>::AmountIsZero
        );
    });
}

#[test]
fn sell_fails_if_cancel_strategy_applied() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        let mut market = market_mock::<Runtime>(MARKET_CREATOR);
        let required_asset_count = match &market.market_type {
            MarketType::Scalar(_) => panic!("Categorical market type is expected!"),
            MarketType::Categorical(categories) => *categories,
        };
        market.status = MarketStatus::Active;
        Markets::<Runtime>::insert(market_id, market);

        let asset_count = required_asset_count;
        let asset = Assets::CategoricalOutcome(market_id, 0);
        let amount_in = 10 * BASE;
        assert_ok!(AssetRouter::increase_balance(asset, &ALICE, amount_in, Precision::Exact));
        let max_price = (BASE / 2).saturated_into::<BalanceOf<Runtime>>();
        let orders = vec![];
        let strategy = Strategy::ImmediateOrCancel;
        assert_noop!(
            HybridRouter::sell(
                RuntimeOrigin::signed(ALICE),
                market_id,
                asset_count,
                asset,
                amount_in,
                max_price,
                orders,
                strategy,
            ),
            Error::<Runtime>::CancelStrategyApplied
        );
    });
}

#[test]
fn sell_fails_if_market_does_not_exist() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        let asset_count = 2;
        let asset = Assets::CategoricalOutcome(market_id, 0);
        let amount_in = 10 * BASE;
        assert_ok!(AssetRouter::increase_balance(asset, &ALICE, amount_in, Precision::Exact));
        let max_price = (BASE / 2).saturated_into::<BalanceOf<Runtime>>();
        let orders = vec![];
        let strategy = Strategy::ImmediateOrCancel;
        assert_noop!(
            HybridRouter::sell(
                RuntimeOrigin::signed(ALICE),
                market_id,
                asset_count,
                asset,
                amount_in,
                max_price,
                orders,
                strategy,
            ),
            MError::<Runtime>::MarketDoesNotExist
        );
    });
}
