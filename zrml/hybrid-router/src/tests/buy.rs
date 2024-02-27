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
        let amount = _2;
        //*  max_price is just 1 larger than the spot price of the AMM
        //*  this results in a low buy amount on the AMM
        let max_price = (_1_2 + 1u128).saturated_into::<BalanceOf<Runtime>>();
        let orders = vec![];
        let strategy = Strategy::LimitOrder;
        assert_ok!(HybridRouter::buy(
            RuntimeOrigin::signed(ALICE),
            market_id,
            asset_count as u32,
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
                maker_amount: 9999999987,
                taker_asset: asset,
                taker_amount: 19999999971,
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
            asset_count as u32,
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
fn buy_places_limit_order() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        let mut market = market_mock::<Runtime>(MARKET_CREATOR);
        let base_asset = market.base_asset;
        let required_asset_count = match &market.market_type {
            MarketType::Scalar(_) => panic!("Categorical market type is expected!"),
            MarketType::Categorical(categories) => *categories as u32,
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
                maker_amount: 5 * BASE,
                taker_asset: asset,
                taker_amount: 10 * BASE,
            }
        );
    });
}

#[test]
fn buy_emits_event() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        let mut market = market_mock::<Runtime>(MARKET_CREATOR);
        let required_asset_count = match &market.market_type {
            MarketType::Scalar(_) => panic!("Categorical market type is expected!"),
            MarketType::Categorical(categories) => *categories as u32,
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

        System::assert_last_event(
            Event::<Runtime>::HybridRouterBuyExecuted {
                who: ALICE,
                market_id,
                asset,
                amount,
                max_price,
            }
            .into(),
        );
    });
}

#[test]
fn buy_fails_if_cancel_strategy_applied() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        let mut market = market_mock::<Runtime>(MARKET_CREATOR);
        let required_asset_count = match &market.market_type {
            MarketType::Scalar(_) => panic!("Categorical market type is expected!"),
            MarketType::Categorical(categories) => *categories as u32,
        };
        market.status = MarketStatus::Active;
        Markets::<Runtime>::insert(market_id, market);

        let asset_count = required_asset_count;
        let asset = Asset::CategoricalOutcome(market_id, 0);
        let amount = 10 * BASE;
        let max_price = (BASE / 2).saturated_into::<BalanceOf<Runtime>>();
        let orders = vec![];
        let strategy = Strategy::ImmediateOrCancel;
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
            Error::<Runtime>::CancelStrategyApplied
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
            MarketType::Categorical(categories) => *categories as u32,
        };
        market.status = MarketStatus::Active;
        Markets::<Runtime>::insert(market_id, market);

        assert_ok!(Balances::set_balance(RuntimeOrigin::root(), ALICE, 0, 0));

        let asset_count = required_asset_count;
        let asset = Asset::CategoricalOutcome(market_id, 0);
        let amount = 10 * BASE;
        let max_price = (BASE / 2).saturated_into::<BalanceOf<Runtime>>();
        let orders = vec![];
        let strategy = Strategy::ImmediateOrCancel;
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

#[test]
fn buy_fails_if_asset_count_mismatch() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        let mut market = market_mock::<Runtime>(MARKET_CREATOR);
        let required_asset_count = match &market.market_type {
            MarketType::Scalar(_) => panic!("Categorical market type is expected!"),
            MarketType::Categorical(categories) => *categories as u32,
        };
        market.status = MarketStatus::Active;
        Markets::<Runtime>::insert(market_id, market);

        let asset_count = 2;
        assert_ne!(required_asset_count, asset_count);
        let asset = Asset::CategoricalOutcome(market_id, 0);
        let amount = 10 * BASE;
        let max_price = (BASE / 2).saturated_into::<BalanceOf<Runtime>>();
        let orders = vec![];
        let strategy = Strategy::ImmediateOrCancel;
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
            Error::<Runtime>::AssetCountMismatch
        );
    });
}

#[test]
fn buy_fails_if_market_does_not_exist() {
    ExtBuilder::default().build().execute_with(|| {
        let market_id = 0;
        let asset_count = 2;
        let asset = Asset::CategoricalOutcome(market_id, 0);
        let amount = 10 * BASE;
        let max_price = (BASE / 2).saturated_into::<BalanceOf<Runtime>>();
        let orders = vec![];
        let strategy = Strategy::ImmediateOrCancel;
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
            MError::<Runtime>::MarketDoesNotExist
        );
    });
}
