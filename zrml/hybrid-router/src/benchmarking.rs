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

#![allow(
    // Auto-generated code is a no man's land
    clippy::arithmetic_side_effects
)]
#![cfg(feature = "runtime-benchmarks")]

#[cfg(test)]
use crate::Pallet as HybridRouter;

use crate::*;
use alloc::{vec, vec::Vec};
use frame_benchmarking::v2::*;
use frame_support::{
    assert_ok,
    storage::{with_transaction, TransactionOutcome::*},
};
use frame_system::RawOrigin;
use orml_traits::MultiCurrency;
use sp_runtime::{Perbill, SaturatedConversion};
use types::Strategy;
use zeitgeist_primitives::{
    constants::{base_multiples::*, CENT},
    math::fixed::{BaseProvider, FixedDiv, ZeitgeistBase},
    traits::{CompleteSetOperationsApi, DeployPoolApi, HybridRouterOrderbookApi},
    types::{Asset, Market, MarketCreation, MarketPeriod, MarketStatus, MarketType, ScoringRule},
};
use zrml_market_commons::MarketCommonsPalletApi;

// Same behavior as `assert_ok!`, except that it wraps the call inside a transaction layer. Required
// when calling into functions marked `require_transactional` to avoid a `Transactional(NoLayer)`
// error.
macro_rules! assert_ok_with_transaction {
    ($expr:expr) => {{
        assert_ok!(with_transaction(|| match $expr {
            Ok(val) => Commit(Ok(val)),
            Err(err) => Rollback(Err(err)),
        }));
    }};
}

fn create_spot_prices<T: Config>(asset_count: u16) -> Vec<BalanceOf<T>> {
    let base = ZeitgeistBase::<u128>::get().unwrap();
    let amount = base / asset_count as u128;
    let remainder = (base % asset_count as u128).saturated_into::<BalanceOf<T>>();

    let mut amounts = vec![amount.saturated_into::<BalanceOf<T>>(); asset_count as usize];
    amounts[0] += remainder;

    amounts
}

fn create_market<T>(caller: T::AccountId, base_asset: AssetOf<T>, asset_count: u16) -> MarketIdOf<T>
where
    T: Config,
{
    let market = Market {
        market_id: 0u8.into(),
        base_asset,
        creation: MarketCreation::Permissionless,
        creator_fee: Perbill::zero(),
        creator: caller.clone(),
        oracle: caller,
        metadata: vec![0, 50],
        market_type: MarketType::Categorical(asset_count),
        period: MarketPeriod::Block(0u32.into()..1u32.into()),
        deadlines: Default::default(),
        scoring_rule: ScoringRule::AmmCdaHybrid,
        status: MarketStatus::Active,
        report: None,
        resolved_outcome: None,
        dispute_mechanism: None,
        bonds: Default::default(),
        early_close: None,
    };
    let maybe_market_id = T::MarketCommons::push_market(market);
    maybe_market_id.unwrap()
}

fn create_market_and_deploy_pool<T: Config>(
    caller: T::AccountId,
    base_asset: AssetOf<T>,
    asset_count: u16,
    amount: BalanceOf<T>,
) -> MarketIdOf<T> {
    let market_id = create_market::<T>(caller.clone(), base_asset, asset_count);
    let total_cost = amount + T::AssetManager::minimum_balance(base_asset);
    assert_ok!(T::AssetManager::deposit(base_asset, &caller, total_cost));
    assert_ok_with_transaction!(T::CompleteSetOperations::buy_complete_set(
        caller.clone(),
        market_id,
        amount
    ));
    assert_ok_with_transaction!(T::AmmPoolDeployer::deploy_pool(
        caller,
        market_id,
        amount,
        create_spot_prices::<T>(asset_count),
        CENT.saturated_into(),
    ));
    market_id
}

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn buy(n: Linear<2, 16>, o: Linear<0, 10>) {
        let buyer: T::AccountId = whitelisted_caller();
        let base_asset = Asset::Ztg;
        let asset_count = n.try_into().unwrap();
        let market_id = create_market_and_deploy_pool::<T>(
            buyer.clone(),
            base_asset,
            asset_count,
            _100.saturated_into(),
        );

        let asset = Asset::CategoricalOutcome(market_id, 0u16);
        let amount_in = _1000.saturated_into();
        assert_ok!(T::AssetManager::deposit(base_asset, &buyer, amount_in));

        let spot_prices = create_spot_prices::<T>(asset_count);
        let first_spot_price = spot_prices[0];

        let max_price = _9_10.saturated_into();
        let orders = (0u128..o as u128).collect::<Vec<_>>();
        let maker_asset = asset;
        let maker_amount = _20.saturated_into();
        let taker_asset = base_asset;
        let taker_amount: BalanceOf<T> = _11.saturated_into();
        assert!(taker_amount.bdiv_floor(maker_amount).unwrap() > first_spot_price);
        for (i, order_id) in orders.iter().enumerate() {
            let order_creator: T::AccountId = account("order_creator", *order_id as u32, 0);
            let surplus = ((i + 1) as u128) * _1_2;
            let taker_amount = taker_amount + surplus.saturated_into::<BalanceOf<T>>();
            assert_ok!(T::AssetManager::deposit(maker_asset, &order_creator, maker_amount));
            assert_ok!(T::Orderbook::place_order(
                order_creator,
                market_id,
                maker_asset,
                maker_amount,
                taker_asset,
                taker_amount,
            ));
        }
        let strategy = Strategy::LimitOrder;

        #[extrinsic_call]
        buy(
            RawOrigin::Signed(buyer.clone()),
            market_id,
            asset_count,
            asset,
            amount_in,
            max_price,
            orders,
            strategy,
        );

        let buyer_limit_order = T::Orderbook::order(o as u128).unwrap();
        assert_eq!(buyer_limit_order.market_id, market_id);
        assert_eq!(buyer_limit_order.maker, buyer);
        assert_eq!(buyer_limit_order.maker_asset, base_asset);
        assert_eq!(buyer_limit_order.taker_asset, asset);
    }

    #[benchmark]
    fn sell(n: Linear<2, 10>, o: Linear<0, 10>) {
        let seller: T::AccountId = whitelisted_caller();
        let base_asset = Asset::Ztg;
        let asset_count = n.try_into().unwrap();
        let market_id = create_market_and_deploy_pool::<T>(
            seller.clone(),
            base_asset,
            asset_count,
            _100.saturated_into(),
        );

        let asset = Asset::CategoricalOutcome(market_id, 0u16);
        let amount_in = (_1000 * 100).saturated_into();
        assert_ok!(T::AssetManager::deposit(asset, &seller, amount_in));
        // seller base asset amount needs to exist,
        // otherwise repatriate_reserved_named from order book fails
        // with DeadAccount for base asset repatriate to seller beneficiary
        let min_balance = T::AssetManager::minimum_balance(base_asset);
        assert_ok!(T::AssetManager::deposit(base_asset, &seller, min_balance));

        let spot_prices = create_spot_prices::<T>(asset_count);
        let first_spot_price = spot_prices[0];

        let min_price = _1_100.saturated_into();
        let orders = (0u128..o as u128).collect::<Vec<_>>();
        let maker_asset: AssetOf<T> = base_asset;
        let maker_amount: BalanceOf<T> = _9.saturated_into();
        let taker_asset = asset;
        let taker_amount = _100.saturated_into();
        assert!(maker_amount.bdiv_floor(taker_amount).unwrap() < first_spot_price);
        for (i, order_id) in orders.iter().enumerate() {
            let order_creator: T::AccountId = account("order_creator", *order_id as u32, 0);
            let surplus = ((i + 1) as u128) * _1_2;
            let taker_amount = taker_amount + surplus.saturated_into::<BalanceOf<T>>();
            assert_ok!(T::AssetManager::deposit(
                maker_asset,
                &order_creator,
                maker_amount + _100.saturated_into()
            ));
            T::Orderbook::place_order(
                order_creator,
                market_id,
                maker_asset,
                maker_amount,
                taker_asset,
                taker_amount,
            )
            .unwrap();
        }
        let strategy = Strategy::LimitOrder;

        #[extrinsic_call]
        sell(
            RawOrigin::Signed(seller.clone()),
            market_id,
            asset_count,
            asset,
            amount_in,
            min_price,
            orders,
            strategy,
        );

        let seller_limit_order = T::Orderbook::order(o as u128).unwrap();
        assert_eq!(seller_limit_order.market_id, market_id);
        assert_eq!(seller_limit_order.maker, seller);
        assert_eq!(seller_limit_order.maker_asset, asset);
        assert_eq!(seller_limit_order.taker_asset, base_asset);
    }

    impl_benchmark_test_suite!(
        HybridRouter,
        crate::mock::ExtBuilder::default().build(),
        crate::mock::Runtime
    );
}
