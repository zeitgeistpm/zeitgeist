// Copyright 2023-2025 Forecasting Technologies LTD.
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
    liquidity_tree::{traits::LiquidityTreeHelper, types::LiquidityTree},
    traits::{LiquiditySharesManager, PoolOperations, PoolStorage},
    types::{DecisionMarketOracle, DecisionMarketOracleScoreboard},
    AssetOf, BalanceOf, MarketIdOf, Pallet as NeoSwaps, Pools, MIN_SPOT_PRICE,
};
use alloc::{vec, vec::Vec};
use core::{cell::Cell, iter, marker::PhantomData};
use frame_benchmarking::v2::*;
use frame_support::{
    assert_ok,
    storage::{with_transaction, TransactionOutcome::*},
};
use frame_system::RawOrigin;
use orml_traits::MultiCurrency;
use sp_runtime::{
    traits::{Get, Zero},
    Perbill, SaturatedConversion,
};
use zeitgeist_primitives::{
    constants::{base_multiples::*, CENT},
    math::fixed::{BaseProvider, FixedDiv, FixedMul, ZeitgeistBase},
    traits::{CombinatorialTokensFuel, CompleteSetOperationsApi, FutarchyOracle},
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

trait LiquidityTreeBenchmarkHelper<T: Config> {
    fn calculate_min_pool_shares_amount(&self) -> BalanceOf<T>;
}

impl<T, U> LiquidityTreeBenchmarkHelper<T> for LiquidityTree<T, U>
where
    T: Config,
    U: Get<u32>,
{
    /// Calculate the minimum amount required to join a liquidity tree without erroring.
    fn calculate_min_pool_shares_amount(&self) -> BalanceOf<T> {
        self.total_shares()
            .unwrap()
            .bmul_ceil(MIN_RELATIVE_LP_POSITION_VALUE.saturated_into())
            .unwrap()
    }
}

/// Utilities for setting up benchmarks.
struct BenchmarkHelper<T> {
    current_id: Cell<u32>,
    _marker: PhantomData<T>,
}

impl<T: Config> BenchmarkHelper<T> {
    fn new() -> Self {
        BenchmarkHelper { current_id: Cell::new(0), _marker: PhantomData }
    }

    /// Return an iterator which ranges over _unused_ accounts.
    fn accounts(&self) -> impl Iterator<Item = T::AccountId> + '_ {
        iter::from_fn(move || {
            let id = self.current_id.get();
            self.current_id.set(id + 1);
            Some(account("", id, 0))
        })
    }

    /// Populates the market's liquidity tree until almost full with one free leaf remaining.
    /// Ensures that the tree has the expected configuration of nodes.
    fn populate_liquidity_tree_with_free_leaf(&self, market_id: MarketIdOf<T>) {
        let max_node_count = LiquidityTreeOf::<T>::max_node_count();
        let last = (max_node_count - 1) as usize;
        for caller in self.accounts().take(last - 1) {
            add_liquidity_provider_to_market::<T>(market_id, caller);
        }
        // Verify that we've got the right number of nodes.
        let pool = Pools::<T>::get(market_id).unwrap();
        assert_eq!(pool.liquidity_shares_manager.nodes.len(), last);
    }

    /// Populates the market's liquidity tree until full. The `caller` is the owner of the last
    /// leaf. Ensures that the tree has the expected configuration of nodes.
    fn populate_liquidity_tree_until_full(&self, market_id: MarketIdOf<T>, caller: T::AccountId) {
        // Start by populating the entire tree except for one node. `caller` will then join and
        // occupy the last node.
        self.populate_liquidity_tree_with_free_leaf(market_id);
        add_liquidity_provider_to_market::<T>(market_id, caller);
        // Verify that we've got the right number of nodes.
        let pool = Pools::<T>::get(market_id).unwrap();
        let max_node_count = LiquidityTreeOf::<T>::max_node_count();
        assert_eq!(pool.liquidity_shares_manager.nodes.len(), max_node_count as usize);
    }

    /// Populates the market's liquidity tree until almost full with one abandoned node remaining.
    fn populate_liquidity_tree_with_abandoned_node(&self, market_id: MarketIdOf<T>) {
        // Start by populating the entire tree. `caller` will own one of the leaves, withdraw their
        // stake, leaving an abandoned node at a leaf.
        let caller = self.accounts().next().unwrap();
        self.populate_liquidity_tree_until_full(market_id, caller.clone());
        let pool = Pools::<T>::get(market_id).unwrap();
        let pool_shares_amount = pool.liquidity_shares_manager.shares_of(&caller).unwrap();
        assert_ok!(NeoSwaps::<T>::exit(
            RawOrigin::Signed(caller).into(),
            market_id,
            pool_shares_amount,
            vec![Zero::zero(); pool.assets().len()]
        ));
        // Verify that we've got the right number of nodes.
        let pool = Pools::<T>::get(market_id).unwrap();
        let max_node_count = LiquidityTreeOf::<T>::max_node_count();
        assert_eq!(pool.liquidity_shares_manager.nodes.len(), max_node_count as usize);
        let last = max_node_count - 1;
        assert_eq!(pool.liquidity_shares_manager.abandoned_nodes, vec![last]);
    }

    /// Run the common setup of `join` benchmarks and return the target market's ID and Bob's
    /// address (who will execute the call).
    ///
    /// Parameters:
    ///
    /// - `market_id`: The ID to set the benchmark up for.
    /// - `complete_set_amount`: The amount of complete sets to buy for Bob.
    fn set_up_liquidity_benchmark(
        &self,
        market_id: MarketIdOf<T>,
        account: AccountIdOf<T>,
        complete_set_amount: Option<BalanceOf<T>>,
    ) {
        let pool = Pools::<T>::get(market_id).unwrap();
        let multiplier = MIN_RELATIVE_LP_POSITION_VALUE + 1_000;
        let complete_set_amount = complete_set_amount.unwrap_or_else(|| {
            pool.reserves.values().max().unwrap().bmul_ceil(multiplier.saturated_into()).unwrap()
        });
        assert_ok!(T::MultiCurrency::deposit(pool.collateral, &account, complete_set_amount));
        assert_ok_with_transaction!(T::CompleteSetOperations::buy_complete_set(
            account,
            market_id,
            complete_set_amount,
        ));
    }
}

fn create_market<T: Config>(
    caller: T::AccountId,
    base_asset: AssetOf<T>,
    asset_count: AssetIndexType,
) -> MarketIdOf<T> {
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
    T::MarketCommons::push_market(market).unwrap()
}

fn create_spot_prices<T: Config>(asset_count: u16) -> Vec<BalanceOf<T>> {
    let mut result = vec![MIN_SPOT_PRICE.saturated_into(); (asset_count - 1) as usize];
    // Price distribution has no bearing on the benchmarks.
    let remaining_u128 =
        ZeitgeistBase::<u128>::get().unwrap() - (asset_count - 1) as u128 * MIN_SPOT_PRICE;
    result.push(remaining_u128.saturated_into());
    result
}

fn create_market_and_deploy_pool<T: Config>(
    caller: T::AccountId,
    base_asset: AssetOf<T>,
    asset_count: AssetIndexType,
    amount: BalanceOf<T>,
) -> MarketIdOf<T> {
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
        create_spot_prices::<T>(asset_count),
        CENT.saturated_into(),
    ));
    market_id
}

fn deposit_fees<T: Config>(market_id: MarketIdOf<T>, amount: BalanceOf<T>) {
    let mut pool = Pools::<T>::get(market_id).unwrap();
    assert_ok!(T::MultiCurrency::deposit(pool.collateral, &pool.account_id, amount));
    assert_ok!(pool.liquidity_shares_manager.deposit_fees(amount));
    Pools::<T>::insert(market_id, pool);
}

// Let `caller` join the pool of `market_id` after adding the  required funds to their account.
fn add_liquidity_provider_to_market<T: Config>(market_id: MarketIdOf<T>, caller: AccountIdOf<T>) {
    let pool = Pools::<T>::get(market_id).unwrap();
    // Buy a little more to account for rounding.
    let pool_shares_amount =
        pool.liquidity_shares_manager.calculate_min_pool_shares_amount() + _1.saturated_into();
    let ratio =
        pool_shares_amount.bdiv(pool.liquidity_shares_manager.total_shares().unwrap()).unwrap();
    let complete_set_amount =
        pool.reserves.values().max().unwrap().bmul_ceil(ratio).unwrap() * 2u8.into();
    assert_ok!(T::MultiCurrency::deposit(pool.collateral, &caller, complete_set_amount));
    assert_ok_with_transaction!(T::CompleteSetOperations::buy_complete_set(
        caller.clone(),
        market_id,
        complete_set_amount,
    ));
    assert_ok!(NeoSwaps::<T>::join(
        RawOrigin::Signed(caller.clone()).into(),
        market_id,
        pool_shares_amount,
        vec![u128::MAX.saturated_into(); pool.assets().len()]
    ));
}

#[benchmarks]
mod benchmarks {
    use super::*;

    /// TODO(#1221): Replace hardcoded variant with `{ MAX_ASSETS as u32 }` as soon as possible.
    #[benchmark]
    fn buy(n: Linear<2, 4>) {
        let alice = whitelisted_caller();
        let base_asset = Asset::Ztg;
        let asset_count = n.try_into().unwrap();
        let market_id = create_market_and_deploy_pool::<T>(
            alice,
            base_asset,
            asset_count,
            (100 * _100).saturated_into(),
        );
        let asset_out = Asset::CategoricalOutcome(market_id, 0);
        let amount_in = _1000.saturated_into();
        let min_amount_out = 0u8.saturated_into();

        let helper = BenchmarkHelper::<T>::new();
        let bob = helper.accounts().next().unwrap();
        assert_ok!(T::MultiCurrency::deposit(base_asset, &bob, amount_in));

        #[extrinsic_call]
        _(RawOrigin::Signed(bob), market_id, asset_count, asset_out, amount_in, min_amount_out);
    }

    #[benchmark]
    fn sell(n: Linear<2, 128>) {
        let alice = whitelisted_caller();
        let base_asset = Asset::Ztg;
        let asset_count = n.try_into().unwrap();
        let market_id = create_market_and_deploy_pool::<T>(
            alice,
            base_asset,
            asset_count,
            (100 * _100).saturated_into(),
        );
        let asset_in = Asset::CategoricalOutcome(market_id, asset_count - 1);
        let amount_in = _100.saturated_into();
        let min_amount_out = 0u8.saturated_into();

        let helper = BenchmarkHelper::<T>::new();
        let bob = helper.accounts().next().unwrap();
        assert_ok!(T::MultiCurrency::deposit(asset_in, &bob, amount_in));

        #[extrinsic_call]
        _(RawOrigin::Signed(bob), market_id, asset_count, asset_in, amount_in, min_amount_out);
    }

    // Bob already owns a leaf at maximum depth in the tree but decides to increase his stake.
    // Maximum propagation steps thanks to maximum depth.
    #[benchmark]
    fn join_in_place(n: Linear<2, 128>) {
        let alice: T::AccountId = whitelisted_caller();
        let base_asset = Asset::Ztg;
        let asset_count = n.try_into().unwrap();
        let market_id = create_market_and_deploy_pool::<T>(
            alice.clone(),
            base_asset,
            asset_count,
            (100 * _100).saturated_into(),
        );
        let helper = BenchmarkHelper::<T>::new();
        let bob = helper.accounts().next().unwrap();
        helper.populate_liquidity_tree_until_full(market_id, bob.clone());
        let pool_shares_amount = _1.saturated_into();
        // Due to rounding, we need to buy a little more than the pool share amount.
        let complete_set_amount = _1000.saturated_into();
        helper.set_up_liquidity_benchmark(market_id, bob.clone(), Some(complete_set_amount));
        let max_amounts_in = vec![u128::MAX.saturated_into(); asset_count as usize];

        // Double check that there's no abandoned node or free leaf.
        let pool = Pools::<T>::get(market_id).unwrap();
        assert_eq!(pool.liquidity_shares_manager.abandoned_nodes.len(), 0);
        let max_node_count = LiquidityTreeOf::<T>::max_node_count();
        assert_eq!(pool.liquidity_shares_manager.node_count(), max_node_count);

        #[extrinsic_call]
        join(RawOrigin::Signed(bob), market_id, pool_shares_amount, max_amounts_in);
    }

    // Bob joins the pool and is assigned an abandoned node at maximum depth in the tree. Maximum
    // propagation steps thanks to maximum depth.
    #[benchmark]
    fn join_reassigned(n: Linear<2, 128>) {
        let alice: T::AccountId = whitelisted_caller();
        let base_asset = Asset::Ztg;
        let asset_count = n.try_into().unwrap();
        let market_id = create_market_and_deploy_pool::<T>(
            alice.clone(),
            base_asset,
            asset_count,
            (100 * _100).saturated_into(),
        );
        let helper = BenchmarkHelper::<T>::new();
        helper.populate_liquidity_tree_with_abandoned_node(market_id);
        let pool = Pools::<T>::get(market_id).unwrap();
        let pool_shares_amount = pool.liquidity_shares_manager.calculate_min_pool_shares_amount();
        // Due to rounding, we need to buy a little more than the pool share amount.
        let bob = helper.accounts().next().unwrap();
        helper.set_up_liquidity_benchmark(market_id, bob.clone(), None);
        let max_amounts_in = vec![u128::MAX.saturated_into(); asset_count as usize];

        // Double check that there's an abandoned node.
        assert_eq!(pool.liquidity_shares_manager.abandoned_nodes.len(), 1);

        #[extrinsic_call]
        join(RawOrigin::Signed(bob), market_id, pool_shares_amount, max_amounts_in);

        let pool = Pools::<T>::get(market_id).unwrap();
        assert_eq!(pool.liquidity_shares_manager.abandoned_nodes.len(), 0);
    }

    // Bob joins the pool and is assigned a leaf at maximum depth in the tree. Maximum propagation
    // steps thanks to maximum depth.
    #[benchmark]
    fn join_leaf(n: Linear<2, 128>) {
        let alice: T::AccountId = whitelisted_caller();
        let base_asset = Asset::Ztg;
        let asset_count = n.try_into().unwrap();
        let market_id = create_market_and_deploy_pool::<T>(
            alice.clone(),
            base_asset,
            asset_count,
            (100 * _100).saturated_into(),
        );
        let helper = BenchmarkHelper::<T>::new();
        helper.populate_liquidity_tree_with_free_leaf(market_id);
        let pool = Pools::<T>::get(market_id).unwrap();
        let pool_shares_amount = pool.liquidity_shares_manager.calculate_min_pool_shares_amount();
        // Due to rounding, we need to buy a little more than the pool share amount.
        let bob = helper.accounts().next().unwrap();
        helper.set_up_liquidity_benchmark(market_id, bob.clone(), None);
        let max_amounts_in = vec![u128::MAX.saturated_into(); asset_count as usize];

        // Double-check that there's a free leaf.
        let max_node_count = LiquidityTreeOf::<T>::max_node_count();
        assert_eq!(pool.liquidity_shares_manager.node_count(), max_node_count - 1);

        #[extrinsic_call]
        join(RawOrigin::Signed(bob), market_id, pool_shares_amount, max_amounts_in);

        // Ensure that the leaf is taken.
        let pool = Pools::<T>::get(market_id).unwrap();
        assert_eq!(pool.liquidity_shares_manager.node_count(), max_node_count);
    }

    // Worst-case benchmark of `exit`. A couple of conditions must be met to get the worst-case:
    //
    // - Caller withdraws their total share (the node is then abandoned, resulting in extra writes).
    // - The pool is kept alive (changing the pool struct instead of destroying it is heavier).
    // - The caller owns a leaf of maximum depth (equivalent to the second condition unless the tree
    //   has max depth zero).
    #[benchmark]
    fn exit(n: Linear<2, 128>) {
        let alice: T::AccountId = whitelisted_caller();
        let base_asset = Asset::Ztg;
        let asset_count = n.try_into().unwrap();
        let market_id = create_market_and_deploy_pool::<T>(
            alice.clone(),
            base_asset,
            asset_count,
            (100 * _100).saturated_into(),
        );
        let min_amounts_out = vec![0u8.into(); asset_count as usize];

        let helper = BenchmarkHelper::<T>::new();
        let bob = helper.accounts().next().unwrap();
        helper.populate_liquidity_tree_until_full(market_id, bob.clone());
        let pool = Pools::<T>::get(market_id).unwrap();
        let pool_shares_amount = pool.liquidity_shares_manager.shares_of(&bob).unwrap();

        #[extrinsic_call]
        _(RawOrigin::Signed(bob), market_id, pool_shares_amount, min_amounts_out);

        assert!(Pools::<T>::contains_key(market_id)); // Ensure we took the right turn.
    }

    // Worst-case benchmark of `withdraw_fees`: Bob, who owns a leaf of maximum depth, withdraws his
    // stake.
    #[benchmark]
    fn withdraw_fees() {
        let alice: T::AccountId = whitelisted_caller();
        let market_id = create_market_and_deploy_pool::<T>(
            alice.clone(),
            Asset::Ztg,
            2u16,
            (100 * _100).saturated_into(),
        );
        let helper = BenchmarkHelper::<T>::new();
        let bob = helper.accounts().next().unwrap();
        helper.populate_liquidity_tree_until_full(market_id, bob.clone());
        helper.set_up_liquidity_benchmark(market_id, bob.clone(), None);

        // Mock up some fees. Needs to be large enough to ensure that Bob's share is not smaller
        // than the existential deposit.
        let max_node_count = LiquidityTreeOf::<T>::max_node_count() as u128;
        let fee_amount = (max_node_count * _1000).saturated_into();
        deposit_fees::<T>(market_id, fee_amount);

        #[extrinsic_call]
        _(RawOrigin::Signed(bob), market_id);
    }

    #[benchmark]
    fn deploy_pool(n: Linear<2, 128>) {
        let alice: T::AccountId = whitelisted_caller();
        let base_asset = Asset::Ztg;
        let asset_count = n.try_into().unwrap();
        let market_id = create_market::<T>(alice.clone(), base_asset, asset_count);
        let amount = (100 * _100).saturated_into();
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
            create_spot_prices::<T>(asset_count),
            CENT.saturated_into(),
        );
    }

    // Remark on benchmarks for combinatorial pools: Combinatorial buying, selling and deploying
    // pools depends on the number of assets as well as the number of markets. But these parameters
    // depend on each other (the more markets, the more assets). The benchmark parameter is the
    // market count and the logarithm of the number of assets. This maximizes the number of markets
    // per asset.

    #[benchmark]
    fn combo_buy(n: Linear<1, 7>) {
        let market_count = n;

        let alice: T::AccountId = whitelisted_caller();
        let base_asset = Asset::Ztg;
        let asset_count = 2u16.pow(market_count);

        let mut market_ids = vec![];
        for _ in 0..market_count {
            let market_id = create_market::<T>(alice.clone(), base_asset, 2);
            market_ids.push(market_id);
        }

        let amount = (100 * _100).saturated_into();
        let total_cost = amount + T::MultiCurrency::minimum_balance(base_asset);
        assert_ok!(T::MultiCurrency::deposit(base_asset, &alice, total_cost));
        assert_ok!(NeoSwaps::<T>::deploy_combinatorial_pool(
            RawOrigin::Signed(alice).into(),
            asset_count,
            market_ids,
            amount,
            create_spot_prices::<T>(asset_count),
            CENT.saturated_into(),
            FuelOf::<T>::from_total(16),
        ));

        let pool_id = 0u8.into();
        let pool = <Pallet<T> as PoolStorage>::get(pool_id).unwrap();
        let assets = pool.assets();

        let amount_in = _1.saturated_into();
        let min_amount_out = Zero::zero();

        // Work is maximized by having no keep indicies.
        let middle = asset_count / 2;
        let buy_arg = (0..middle).map(|i| assets[i as usize]).collect::<Vec<_>>();
        let sell_arg = (middle..asset_count).map(|i| assets[i as usize]).collect::<Vec<_>>();

        let helper = BenchmarkHelper::<T>::new();
        let bob = helper.accounts().next().unwrap();
        assert_ok!(T::MultiCurrency::deposit(base_asset, &bob, amount_in));

        #[extrinsic_call]
        _(
            RawOrigin::Signed(bob),
            pool_id,
            asset_count,
            buy_arg,
            sell_arg,
            amount_in,
            min_amount_out,
        );
    }

    #[benchmark]
    fn combo_sell(n: Linear<1, 7>) {
        let market_count = n;

        let alice: T::AccountId = whitelisted_caller();
        let base_asset = Asset::Ztg;
        let asset_count = 2u16.pow(market_count);

        let mut market_ids = vec![];
        for _ in 0..market_count {
            let market_id = create_market::<T>(alice.clone(), base_asset, 2);
            market_ids.push(market_id);
        }

        let amount = (100 * _100).saturated_into();
        let total_cost = amount + T::MultiCurrency::minimum_balance(base_asset);
        assert_ok!(T::MultiCurrency::deposit(base_asset, &alice, total_cost));
        assert_ok!(NeoSwaps::<T>::deploy_combinatorial_pool(
            RawOrigin::Signed(alice).into(),
            asset_count,
            market_ids,
            amount,
            create_spot_prices::<T>(asset_count),
            CENT.saturated_into(),
            FuelOf::<T>::from_total(16),
        ));

        let pool_id = 0u8.into();
        let pool = <Pallet<T> as PoolStorage>::get(pool_id).unwrap();
        let assets = pool.assets();

        // Work is maximized by having as few sell indices as possible.
        let buy_arg = vec![assets[0]];
        let sell_arg = vec![assets[1]];
        let keep_arg = (2..asset_count).map(|i| assets[i as usize]).collect::<Vec<_>>();

        let amount_buy: BalanceOf<T> = (100 * _2).saturated_into();
        let amount_keep = if keep_arg.is_empty() {
            // If n = 1;
            Zero::zero()
        } else {
            _1.saturated_into()
        };
        let min_amount_out = Zero::zero();

        let helper = BenchmarkHelper::<T>::new();
        let bob = helper.accounts().next().unwrap();

        // We don't care about being precise here and just deposit a huge bunch of tokens for Bob.
        for &asset in assets.iter() {
            let amount_for_bob = amount_buy;
            assert_ok!(T::MultiCurrency::deposit(asset, &bob, amount_for_bob));
        }

        #[extrinsic_call]
        _(
            RawOrigin::Signed(bob),
            pool_id,
            asset_count,
            buy_arg,
            keep_arg,
            sell_arg,
            amount_buy,
            amount_keep,
            min_amount_out,
        );
    }

    #[benchmark]
    fn deploy_combinatorial_pool(n: Linear<1, 7>, m: Linear<32, 64>) {
        let market_count = n;
        let total = m;

        let alice: T::AccountId = whitelisted_caller();
        let base_asset = Asset::Ztg;
        let asset_count = 2u16.pow(market_count);

        let mut market_ids = vec![];
        for _ in 0..market_count {
            let market_id = create_market::<T>(alice.clone(), base_asset, 2);
            market_ids.push(market_id);
        }

        let amount = (100 * _100).saturated_into();
        let total_cost = amount + T::MultiCurrency::minimum_balance(base_asset);
        assert_ok!(T::MultiCurrency::deposit(base_asset, &alice, total_cost));

        let spot_prices = create_spot_prices::<T>(asset_count);
        let swap_fee = CENT.saturated_into();

        #[extrinsic_call]
        _(
            RawOrigin::Signed(alice),
            asset_count,
            market_ids,
            amount,
            spot_prices,
            swap_fee,
            FuelOf::<T>::from_total(total),
        );
    }

    #[benchmark]
    fn decision_market_oracle_evaluate() {
        let alice = whitelisted_caller();
        let base_asset = Asset::Ztg;
        let asset_count = 2;
        let market_id = create_market_and_deploy_pool::<T>(
            alice,
            base_asset,
            asset_count,
            (100 * _100).saturated_into(),
        );

        let pool = Pools::<T>::get(market_id).unwrap();
        let assets = pool.assets();

        let scoreboard = DecisionMarketOracleScoreboard::<T>::new(
            Zero::zero(),
            Zero::zero(),
            Zero::zero(),
            Zero::zero(),
        );
        let oracle = DecisionMarketOracle::<T>::new(market_id, assets[0], assets[1], scoreboard);

        #[block]
        {
            let _ = oracle.evaluate();
        }
    }

    #[benchmark]
    fn decision_market_oracle_update() {
        let alice = whitelisted_caller();
        let base_asset = Asset::Ztg;
        let asset_count = 2;
        let market_id = create_market_and_deploy_pool::<T>(
            alice,
            base_asset,
            asset_count,
            (100 * _100).saturated_into(),
        );

        let pool = Pools::<T>::get(market_id).unwrap();
        let assets = pool.assets();

        let scoreboard = DecisionMarketOracleScoreboard::<T>::new(
            Zero::zero(),
            Zero::zero(),
            Zero::zero(),
            Zero::zero(),
        );
        let mut oracle =
            DecisionMarketOracle::<T>::new(market_id, assets[0], assets[1], scoreboard);

        #[block]
        {
            let _ = oracle.update(1u8.into());
        }
    }

    impl_benchmark_test_suite!(
        NeoSwaps,
        crate::mock::ExtBuilder::default().build(),
        crate::mock::Runtime
    );
}
