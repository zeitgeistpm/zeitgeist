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

#![cfg(test)]

use super::*;

#[test]
fn buy_from_amm_and_then_fill_specified_order() {
    ExtBuilder::default().build().execute_with(|| {
        let liquidity = _10;
        let pivot = _1_100;
        let spot_prices = vec![_1_2 - pivot, _1_2 + pivot];
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

        let asset = Asset::CategoricalOutcome(market_id, 0);
        let amount_in = _2;

        let order_maker_amount = _12;
        let order_taker_amount = _6;
        assert_ok!(AssetManager::deposit(asset, &CHARLIE, order_maker_amount));
        assert_ok!(OrderBook::place_order(
            RuntimeOrigin::signed(CHARLIE),
            market_id,
            asset,
            order_maker_amount,
            BASE_ASSET,
            order_taker_amount,
        ));

        let order_ids = Orders::<Runtime>::iter().map(|(k, _)| k).collect::<Vec<_>>();

        let max_price = _3_4.saturated_into::<BalanceOf<Runtime>>();
        let strategy = Strategy::LimitOrder;
        assert_ok!(HybridRouter::buy(
            RuntimeOrigin::signed(ALICE),
            market_id,
            asset_count,
            asset,
            amount_in,
            max_price,
            order_ids,
            strategy,
        ));

        /*
        AMM does this before the buy trade:

        let swap_fees = pool.swap_fee.bmul(amount)?;
        pool.liquidity_shares_manager.deposit_fees(swap_fees)?; // Should only error unexpectedly!
        let external_fees =
            T::ExternalFees::distribute(market_id, pool.collateral, &pool.account_id, amount);
        let total_fees = external_fees.saturating_add(swap_fees);
        let remaining = amount.checked_sub(&total_fees).ok_or(Error::<T>::Unexpected)?;
        Ok(FeeDistribution { remaining, swap_fees, external_fees })
        */
        // This is what the amm has executed: 2832657984
        let amm_amount_in = 2776004824;
        System::assert_has_event(
            NeoSwapsEvent::<Runtime>::BuyExecuted {
                who: ALICE,
                market_id,
                asset_out: asset,
                amount_in: amm_amount_in,
                // AMM has executed 5664741768
                amount_out: 5552568736,
                // AMM has executed 28326580
                swap_fee_amount: 27760048,
                external_fee_amount: 0,
            }
            .into(),
        );

        let order_ids = Orders::<Runtime>::iter().map(|(k, _)| k).collect::<Vec<_>>();
        assert_eq!(order_ids.len(), 1);
        let order = Orders::<Runtime>::get(order_ids[0]).unwrap();
        let unfilled_base_asset_amount = 42776004824;
        assert_eq!(
            order,
            Order {
                market_id,
                maker: CHARLIE,
                maker_asset: Asset::CategoricalOutcome(market_id, 0),
                maker_amount: 85552009648,
                taker_asset: BASE_ASSET,
                taker_amount: unfilled_base_asset_amount,
            }
        );
        let filled_base_asset_amount = order_taker_amount - unfilled_base_asset_amount;
        assert_eq!(filled_base_asset_amount, 17223995176);
        assert_eq!(amm_amount_in + filled_base_asset_amount, amount_in);
    });
}

#[test]
fn buy_from_amm_if_specified_order_has_higher_prices_than_the_amm() {
    ExtBuilder::default().build().execute_with(|| {
        let liquidity = _10;
        let spot_prices = vec![_1_4, _3_4];
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

        let asset = Asset::CategoricalOutcome(market_id, 0);
        let amount = _2;

        let order_maker_amount = _4;
        let order_taker_amount = _2;
        assert_ok!(AssetManager::deposit(asset, &CHARLIE, order_maker_amount));
        assert_ok!(OrderBook::place_order(
            RuntimeOrigin::signed(CHARLIE),
            market_id,
            asset,
            order_maker_amount,
            BASE_ASSET,
            order_taker_amount,
        ));

        let order_ids = Orders::<Runtime>::iter().map(|(k, _)| k).collect::<Vec<_>>();

        let max_price = _3_4.saturated_into::<BalanceOf<Runtime>>();
        let strategy = Strategy::LimitOrder;
        assert_ok!(HybridRouter::buy(
            RuntimeOrigin::signed(ALICE),
            market_id,
            asset_count,
            asset,
            amount,
            max_price,
            order_ids,
            strategy,
        ));

        let order_ids = Orders::<Runtime>::iter().map(|(k, _)| k).collect::<Vec<_>>();
        assert_eq!(order_ids.len(), 1);
        let order = Orders::<Runtime>::get(order_ids[0]).unwrap();
        assert_eq!(
            order,
            Order {
                market_id,
                maker: CHARLIE,
                maker_asset: Asset::CategoricalOutcome(market_id, 0),
                maker_amount: _4,
                taker_asset: BASE_ASSET,
                taker_amount: _2,
            }
        );
    });
}

#[test]
fn buy_fill_multiple_orders_if_amm_spot_price_higher_than_order_prices() {
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

        let asset = Asset::CategoricalOutcome(market_id, 0);
        let amount_in = _2;

        let order_maker_amount = _1;
        let order_taker_amount = _1_2;
        assert_ok!(AssetManager::deposit(asset, &CHARLIE, 2 * order_maker_amount));
        assert_ok!(OrderBook::place_order(
            RuntimeOrigin::signed(CHARLIE),
            market_id,
            asset,
            order_maker_amount,
            BASE_ASSET,
            order_taker_amount,
        ));
        assert_ok!(OrderBook::place_order(
            RuntimeOrigin::signed(CHARLIE),
            market_id,
            asset,
            order_maker_amount,
            BASE_ASSET,
            order_taker_amount,
        ));

        let order_ids = Orders::<Runtime>::iter().map(|(k, _)| k).collect::<Vec<_>>();

        let max_price = _3_4.saturated_into::<BalanceOf<Runtime>>();
        let strategy = Strategy::LimitOrder;
        assert_ok!(HybridRouter::buy(
            RuntimeOrigin::signed(ALICE),
            market_id,
            asset_count,
            asset,
            amount_in,
            max_price,
            order_ids,
            strategy,
        ));

        let order_ids = Orders::<Runtime>::iter().map(|(k, _)| k).collect::<Vec<_>>();
        assert_eq!(order_ids.len(), 0);
    });
}

#[test]
fn buy_fill_specified_order_partially_if_amm_spot_price_higher() {
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

        let asset = Asset::CategoricalOutcome(market_id, 0);
        let amount = _2;

        let order_maker_amount = _8;
        let order_taker_amount = _4;
        assert_ok!(AssetManager::deposit(asset, &CHARLIE, order_maker_amount));
        assert_ok!(OrderBook::place_order(
            RuntimeOrigin::signed(CHARLIE),
            market_id,
            asset,
            order_maker_amount,
            BASE_ASSET,
            order_taker_amount,
        ));

        let order_ids = Orders::<Runtime>::iter().map(|(k, _)| k).collect::<Vec<_>>();
        assert_eq!(order_ids.len(), 1);
        let order_id = order_ids[0];

        let max_price = _3_4.saturated_into::<BalanceOf<Runtime>>();
        let orders = vec![order_id];
        let strategy = Strategy::LimitOrder;
        assert_ok!(HybridRouter::buy(
            RuntimeOrigin::signed(ALICE),
            market_id,
            asset_count,
            asset,
            amount,
            max_price,
            orders,
            strategy,
        ));

        let order = Orders::<Runtime>::get(order_id).unwrap();
        assert_eq!(
            order,
            Order {
                market_id,
                maker: CHARLIE,
                maker_asset: Asset::CategoricalOutcome(market_id, 0),
                maker_amount: _4,
                taker_asset: BASE_ASSET,
                taker_amount: _2,
            }
        );
    });
}

#[test]
fn buy_fails_if_asset_not_equal_to_order_book_maker_asset() {
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

        let asset = Asset::CategoricalOutcome(market_id, 0);
        let amount = _2;

        assert_ok!(OrderBook::place_order(
            RuntimeOrigin::signed(CHARLIE),
            market_id,
            BASE_ASSET,
            10000000000,
            Asset::CategoricalOutcome(market_id, 0),
            20000000000,
        ));

        let order_ids = Orders::<Runtime>::iter().map(|(k, _)| k).collect::<Vec<_>>();
        assert_eq!(order_ids.len(), 1);
        let order_id = order_ids[0];

        let max_price = _3_4.saturated_into::<BalanceOf<Runtime>>();
        let orders = vec![order_id];
        let strategy = Strategy::LimitOrder;
        assert_noop!(
            HybridRouter::buy(
                RuntimeOrigin::signed(ALICE),
                market_id,
                asset_count,
                asset,
                amount,
                max_price,
                orders,
                strategy,
            ),
            Error::<Runtime>::AssetNotEqualToOrderBookMakerAsset
        );
    });
}

#[test]
fn buy_fails_if_order_price_above_max_price() {
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

        let asset = Asset::CategoricalOutcome(market_id, 0);
        let amount = _2;

        let order_maker_amount = _1;
        assert_ok!(AssetManager::deposit(asset, &CHARLIE, order_maker_amount));
        assert_ok!(OrderBook::place_order(
            RuntimeOrigin::signed(CHARLIE),
            market_id,
            Asset::CategoricalOutcome(market_id, 0),
            order_maker_amount,
            BASE_ASSET,
            _2,
        ));

        let order_ids = Orders::<Runtime>::iter().map(|(k, _)| k).collect::<Vec<_>>();
        assert_eq!(order_ids.len(), 1);
        let order_id = order_ids[0];

        let max_price = _3_4.saturated_into::<BalanceOf<Runtime>>();
        let orders = vec![order_id];
        let strategy = Strategy::LimitOrder;
        assert_noop!(
            HybridRouter::buy(
                RuntimeOrigin::signed(ALICE),
                market_id,
                asset_count,
                asset,
                amount,
                max_price,
                orders,
                strategy,
            ),
            Error::<Runtime>::OrderPriceAboveMaxPrice
        );
    });
}

#[test]
fn buy_from_amm() {
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

        let asset = Asset::CategoricalOutcome(market_id, 0);
        let amount = _2;
        let max_price = _3_4.saturated_into::<BalanceOf<Runtime>>();
        let orders = vec![];
        let strategy = Strategy::LimitOrder;
        assert_ok!(HybridRouter::buy(
            RuntimeOrigin::signed(ALICE),
            market_id,
            asset_count,
            asset,
            amount,
            max_price,
            orders,
            strategy,
        ));

        System::assert_has_event(
            NeoSwapsEvent::<Runtime>::BuyExecuted {
                who: ALICE,
                market_id,
                asset_out: asset,
                amount_in: 20000000000,
                amount_out: 36852900215,
                swap_fee_amount: 200000000,
                external_fee_amount: 200000000,
            }
            .into(),
        );
    });
}

#[test]
fn buy_max_price_lower_than_amm_spot_price_results_in_place_order() {
    ExtBuilder::default().build().execute_with(|| {
        let liquidity = _10;
        let spot_prices = vec![_1_2 + 1u128, _1_2 - 1u128];
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

        let asset = Asset::CategoricalOutcome(market_id, 0);
        let amount = _2;
        //*  max_price is just 1 smaller than the spot price of the AMM
        //*  this results in no buy on the AMM, but places an order on the order book
        let max_price = (_1_2).saturated_into::<BalanceOf<Runtime>>();
        let orders = vec![];
        let strategy = Strategy::LimitOrder;
        assert_ok!(HybridRouter::buy(
            RuntimeOrigin::signed(ALICE),
            market_id,
            asset_count,
            asset,
            amount,
            max_price,
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
                maker_asset: base_asset,
                maker_amount: _2,
                taker_asset: asset,
                taker_amount: _4,
            }
        );
    });
}

#[test]
fn buy_from_amm_but_low_amount() {
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

        let asset = Asset::CategoricalOutcome(market_id, 0);
        let amount_in = _2;
        //*  max_price is just 1 larger than the spot price of the AMM
        //*  this results in a low buy amount_in on the AMM
        let max_price = (_1_2 + 1u128).saturated_into::<BalanceOf<Runtime>>();
        let orders = vec![];
        let strategy = Strategy::LimitOrder;
        assert_ok!(HybridRouter::buy(
            RuntimeOrigin::signed(ALICE),
            market_id,
            asset_count,
            asset,
            amount_in,
            max_price,
            orders,
            strategy,
        ));

        System::assert_has_event(
            NeoSwapsEvent::<Runtime>::BuyExecuted {
                who: ALICE,
                market_id,
                asset_out: asset,
                amount_in: 29,
                amount_out: 58,
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
                maker_asset: base_asset,
                maker_amount: 19999999971,
                taker_asset: asset,
                taker_amount: 39999999935,
            }
        );
    });
}

#[test]
fn buy_from_amm_only() {
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

        let asset = Asset::CategoricalOutcome(market_id, 0);
        let amount = _2;
        let max_price = _3_4.saturated_into::<BalanceOf<Runtime>>();
        let orders = vec![];
        let strategy = Strategy::ImmediateOrCancel;
        assert_ok!(HybridRouter::buy(
            RuntimeOrigin::signed(ALICE),
            market_id,
            asset_count,
            asset,
            amount,
            max_price,
            orders,
            strategy,
        ));

        System::assert_has_event(
            NeoSwapsEvent::<Runtime>::BuyExecuted {
                who: ALICE,
                market_id,
                asset_out: asset,
                amount_in: 20000000000,
                amount_out: 36852900215,
                swap_fee_amount: 200000000,
                external_fee_amount: 200000000,
            }
            .into(),
        );

        let order_keys = Orders::<Runtime>::iter().map(|(k, _)| k).collect::<Vec<_>>();
        assert_eq!(order_keys.len(), 0);
    });
}

#[test]
fn buy_places_limit_order_no_pool() {
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
        let asset = Asset::CategoricalOutcome(market_id, 0);
        let amount = 10 * BASE;
        let max_price = (BASE / 2).saturated_into::<BalanceOf<Runtime>>();
        let orders = vec![];
        let strategy = Strategy::LimitOrder;
        assert_ok!(HybridRouter::buy(
            RuntimeOrigin::signed(ALICE),
            market_id,
            asset_count,
            asset,
            amount,
            max_price,
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
                maker_asset: base_asset,
                maker_amount: 10 * BASE,
                taker_asset: asset,
                taker_amount: 20 * BASE,
            }
        );
    });
}

#[test]
fn buy_fails_if_balance_too_low() {
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
        let asset = Asset::CategoricalOutcome(market_id, 0);
        let amount = 10 * BASE;

        assert_ok!(Balances::set_balance(RuntimeOrigin::root(), ALICE, amount - 1, 0));
        let max_price = (BASE / 2).saturated_into::<BalanceOf<Runtime>>();
        let orders = vec![];
        let strategy = Strategy::LimitOrder;
        assert_noop!(
            HybridRouter::buy(
                RuntimeOrigin::signed(ALICE),
                market_id,
                asset_count,
                asset,
                amount,
                max_price,
                orders,
                strategy,
            ),
            CurrenciesError::<Runtime>::BalanceTooLow
        );
    });
}