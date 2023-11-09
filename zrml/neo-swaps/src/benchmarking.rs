// Copyright 2023 Forecasting Technologies LTD.
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

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::{
    consts::*,
    traits::{liquidity_shares_manager::LiquiditySharesManager, pool_operations::PoolOperations},
    AssetOf, BalanceOf, MarketIdOf, Pallet as NeoSwaps, Pools,
};
use frame_benchmarking::v2::*;
use frame_support::{
    assert_ok,
    storage::{with_transaction, TransactionOutcome::*},
    traits::Get,
};
use frame_system::RawOrigin;
use orml_traits::MultiCurrency;
use sp_runtime::{Perbill, SaturatedConversion};
use zeitgeist_primitives::{
    constants::CENT,
    traits::CompleteSetOperationsApi,
    types::{Asset, Market, MarketCreation, MarketPeriod, MarketStatus, MarketType, ScoringRule},
};
use zrml_market_commons::MarketCommonsPalletApi;

macro_rules! assert_ok_with_transaction {
    ($expr:expr) => {{
        assert_ok!(with_transaction(|| match $expr {
            Ok(val) => Commit(Ok(val)),
            Err(err) => Rollback(Err(err)),
        }));
    }};
}

fn create_market<T>(
    caller: T::AccountId,
    base_asset: AssetOf<T>,
    asset_count: AssetIndexType,
) -> MarketIdOf<T> where T: Config, {
    let market = Market {
        base_asset,
        creation: MarketCreation::Permissionless,
        creator_fee: Perbill::zero(),
        creator: caller.clone(),
        oracle: caller,
        metadata: vec![0, 50],
        market_type: MarketType::Categorical(asset_count),
        period: MarketPeriod::Block(0u32.into()..1u32.into()),
        deadlines: Default::default(),
        scoring_rule: ScoringRule::Lmsr,
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

fn create_market_and_deploy_pool<T>(
    caller: T::AccountId,
    base_asset: AssetOf<T>,
    asset_count: AssetIndexType,
    amount: BalanceOf<T>,
) -> MarketIdOf<T> where T: Config {
    let market_id = create_market::<T>(caller.clone(), base_asset, asset_count);
    let total_cost = amount + T::MultiCurrency::minimum_balance(base_asset);
    assert_ok!(T::MultiCurrency::deposit(base_asset, &caller, total_cost));
    assert_ok_with_transaction!(T::CompleteSetOperations::buy_complete_set(
        caller.clone(),
        market_id,
        amount
    ));
    assert_ok!(NeoSwaps::<T>::deploy_pool(
        RawOrigin::Signed(caller).into(),
        market_id,
        amount,
        vec![_1_2.saturated_into(), _1_2.saturated_into()],
        CENT.saturated_into(),
    ));
    market_id
}

fn deposit_fees<T>(market_id: MarketIdOf<T>, amount: BalanceOf<T>) where T: Config {
    let mut pool = Pools::<T>::get(market_id).unwrap();
    assert_ok!(T::MultiCurrency::deposit(pool.collateral, &pool.account_id, amount));
    assert_ok!(pool.liquidity_shares_manager.deposit_fees(amount));
    Pools::<T>::insert(market_id, pool);
}

/// Populates the market's liquidity tree until almost full with one free leaf remaining and
/// deposits fees to ensure that lazy propagation is triggered.
#[allow(unused)]
fn populate_liquidity_tree_with_free_leaf<T>(market_id: MarketIdOf<T>)
where
    T: Config,
{
    let pool = Pools::<T>::get(market_id).unwrap();
    // The tree has 2^d nodes; the root is already taken, so we need to populate 2^d - 2 nodes.
    let nodes_to_populate = 2u32.pow(T::MaxLiquidityTreeDepth::get()) - 1;
    for i in 1..nodes_to_populate {
        let caller = account("", i, 0);
        assert_ok!(T::MultiCurrency::deposit(pool.collateral, &caller, _100.saturated_into()));
        assert_ok_with_transaction!(T::CompleteSetOperations::buy_complete_set(
            caller.clone(),
            market_id,
            _100.saturated_into(),
        ));
        assert_ok!(NeoSwaps::<T>::join(
            RawOrigin::Signed(caller).into(),
            market_id,
            _1.saturated_into(),
            vec![u128::MAX.saturated_into(); pool.assets().len()]
        ));
    }
    // Verify that we've got the right number of nodes.
    let pool = Pools::<T>::get(market_id).unwrap();
    let last = 2u32.pow(T::MaxLiquidityTreeDepth::get()) - 1;
    assert_eq!(pool.liquidity_shares_manager.nodes.len(), last as usize);
    deposit_fees::<T>(market_id, _1.saturated_into());
}

/// Populates the market's liquidity tree until almost full with one abandoned node remaining and
/// deposits some fees to ensure that lazy propagation is triggered.
#[allow(unused)]
fn populate_liquidity_tree_with_abandoned_node<T>(market_id: MarketIdOf<T>)
where
    T: Config,
{
    // Start by populating the entire tree. `caller` will own one of the leaves, withdraw their
    // stake, leaving an abandoned node at a leaf.
    populate_liquidity_tree_with_free_leaf::<T>(market_id);
    let pool = Pools::<T>::get(market_id).unwrap();
    let caller = whitelisted_caller();
    assert_ok!(T::MultiCurrency::deposit(pool.collateral, &caller, _100.saturated_into()));
    assert_ok_with_transaction!(T::CompleteSetOperations::buy_complete_set(
        caller.clone(),
        market_id,
        _100.saturated_into(),
    ));
    assert_ok!(NeoSwaps::<T>::join(
        RawOrigin::Signed(caller.clone()).into(),
        market_id,
        _1.saturated_into(),
        vec![u128::MAX.saturated_into(); pool.assets().len()]
    ));
    assert_ok!(NeoSwaps::<T>::exit(
        RawOrigin::Signed(caller).into(),
        market_id,
        _1.saturated_into(),
        vec![Zero::zero(); pool.assets().len()]
    ));
    // Verify that we've got the right number of nodes.
    let pool = Pools::<T>::get(market_id).unwrap();
    let last = 2u32.pow(T::MaxLiquidityTreeDepth::get()) - 1;
    assert_eq!(pool.liquidity_shares_manager.nodes.len(), last as usize);
    assert_eq!(pool.liquidity_shares_manager.abandoned_nodes, vec![last]);
    deposit_fees::<T>(market_id, _1.saturated_into());
}

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn buy() {
        let alice = whitelisted_caller();
        let base_asset = Asset::Ztg;
        let asset_count = 2u16;
        let market_id = create_market_and_deploy_pool::<T>(
            alice,
            base_asset,
            asset_count,
            _10.saturated_into(),
        );
        let asset_out = Asset::CategoricalOutcome(market_id, 0);
        let amount_in = _1.saturated_into();
        let min_amount_out = 0u8.saturated_into();

        let bob: T::AccountId = whitelisted_caller();
        assert_ok!(T::MultiCurrency::deposit(base_asset, &bob, amount_in));

        #[extrinsic_call]
        _(RawOrigin::Signed(bob), market_id, asset_count, asset_out, amount_in, min_amount_out);
    }

    #[benchmark]
    fn sell() {
        let alice = whitelisted_caller();
        let base_asset = Asset::Ztg;
        let market_id =
            create_market_and_deploy_pool::<T>(alice, base_asset, 2u16, _10.saturated_into());
        let asset_in = Asset::CategoricalOutcome(market_id, 0);
        let amount_in = _1.saturated_into();
        let min_amount_out = 0u8.saturated_into();

        let bob: T::AccountId = whitelisted_caller();
        assert_ok!(T::MultiCurrency::deposit(asset_in, &bob, amount_in));

        #[extrinsic_call]
        _(RawOrigin::Signed(bob), market_id, 2, asset_in, amount_in, min_amount_out);
    }

    #[benchmark]
    fn join() {
        let alice: T::AccountId = whitelisted_caller();
        let base_asset = Asset::Ztg;
        let market_id = create_market_and_deploy_pool::<T>(
            alice.clone(),
            base_asset,
            2u16,
            _10.saturated_into(),
        );
        let pool_shares_amount = _1.saturated_into();
        // Due to rounding, we need to buy a little more than the pool share amount.
        let complete_set_amount = _2.saturated_into();
        let max_amounts_in = vec![u128::MAX.saturated_into(), u128::MAX.saturated_into()];

        populate_liquidity_tree_with_free_leaf::<T>(market_id);
        assert_ok!(T::MultiCurrency::deposit(base_asset, &alice, complete_set_amount));
        assert_ok_with_transaction!(T::CompleteSetOperations::buy_complete_set(
            alice.clone(),
            market_id,
            complete_set_amount,
        ));

        #[extrinsic_call]
        _(RawOrigin::Signed(alice), market_id, pool_shares_amount, max_amounts_in);
    }

    // There are two execution paths in `exit`: 1) Keep pool alive or 2) destroy it. Clearly 1) is
    // heavier.
    #[benchmark]
    fn exit() {
        let alice: T::AccountId = whitelisted_caller();
        let base_asset = Asset::Ztg;
        let market_id = create_market_and_deploy_pool::<T>(
            alice.clone(),
            base_asset,
            2u16,
            _10.saturated_into(),
        );
        let pool_shares_amount = _1.saturated_into();
        let min_amounts_out = vec![0u8.saturated_into(), 0u8.saturated_into()];

        #[extrinsic_call]
        _(RawOrigin::Signed(alice), market_id, pool_shares_amount, min_amounts_out);

        assert!(Pools::<T>::contains_key(market_id)); // Ensure we took the right turn.
    }

    #[benchmark]
    fn withdraw_fees() {
        let alice: T::AccountId = whitelisted_caller();
        let base_asset = Asset::Ztg;
        let market_id = create_market_and_deploy_pool::<T>(
            alice.clone(),
            base_asset,
            2u16,
            _10.saturated_into(),
        );

        // Mock up some fees.
        deposit_fees::<T>(market_id, _1.saturated_into());

        #[extrinsic_call]
        _(RawOrigin::Signed(alice), market_id);
    }

    #[benchmark]
    fn deploy_pool() {
        let alice: T::AccountId = whitelisted_caller();
        let base_asset = Asset::Ztg;
        let market_id = create_market::<T>(alice.clone(), base_asset, 2);
        let amount = _10.saturated_into();
        let total_cost = amount + T::MultiCurrency::minimum_balance(base_asset);

        assert_ok!(T::MultiCurrency::deposit(base_asset, &alice, total_cost));
        assert_ok_with_transaction!(T::CompleteSetOperations::buy_complete_set(
            alice.clone(),
            market_id,
            amount
        ));

        #[extrinsic_call]
        _(
            RawOrigin::Signed(alice),
            market_id,
            amount,
            vec![_1_2.saturated_into(), _1_2.saturated_into()],
            CENT.saturated_into(),
        );
    }

    impl_benchmark_test_suite!(
        NeoSwaps,
        crate::mock::ExtBuilder::default().build(),
        crate::mock::Runtime
    );
}
