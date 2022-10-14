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
//
// This file incorporates work covered by the license above but
// published without copyright notice by Balancer Labs
// (<https://balancer.finance>, contact@balancer.finance) in the
// balancer-core repository
// <https://github.com/balancer-labs/balancer-core>.

// Auto-generated code is a no man's land
#![allow(clippy::integer_arithmetic, clippy::indexing_slicing)]
#![cfg(feature = "runtime-benchmarks")]

use super::*;
#[cfg(test)]
use crate::Pallet as Swaps;
use crate::{fixed::bmul, pallet::ARBITRAGE_MAX_ITERATIONS, Config, Event, MarketIdOf};
use frame_benchmarking::{
    account, benchmarks, impl_benchmark_test_suite, vec, whitelisted_caller, Vec,
};
use frame_support::{dispatch::UnfilteredDispatchable, traits::Get, weights::Weight};
use frame_system::RawOrigin;
use orml_traits::MultiCurrency;
use sp_runtime::{
    traits::{SaturatedConversion, Zero},
    DispatchError,
};
use zeitgeist_primitives::{
    constants::{BASE, CENT},
    traits::Swaps as _,
    types::{
        Asset, Deadlines, Market, MarketCreation, MarketDisputeMechanism, MarketPeriod,
        MarketStatus, MarketType, OutcomeReport, PoolId, PoolStatus, ScoringRule,
    },
};
use zrml_market_commons::MarketCommonsPalletApi;

fn assert_last_event<T: Config>(generic_event: <T as Config>::Event) {
    frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

// Generates `acc_total` accounts, of which `acc_asset` account do own `asset`
fn generate_accounts_with_assets<T: Config>(
    acc_total: u32,
    acc_asset: u16,
    acc_amount: BalanceOf<T>,
) -> Result<Vec<T::AccountId>, &'static str> {
    let mut accounts = Vec::new();

    for i in 0..acc_total {
        let acc = account("AssetHolder", i, 0);

        for j in 0..acc_asset {
            let asset = Asset::CategoricalOutcome::<MarketIdOf<T>>(0u32.into(), j);
            T::AssetManager::deposit(asset, &acc, acc_amount)?;
        }

        accounts.push(acc);
    }

    Ok(accounts)
}

// Generates ``asset_count`` assets
fn generate_assets<T: Config>(
    owner: &T::AccountId,
    asset_count: usize,
    asset_amount: Option<BalanceOf<T>>,
) -> Vec<Asset<MarketIdOf<T>>> {
    let mut assets: Vec<Asset<MarketIdOf<T>>> = Vec::new();

    let asset_amount_unwrapped: BalanceOf<T> = {
        match asset_amount {
            Some(ac) => ac,
            _ => T::MinLiquidity::get(),
        }
    };

    // Generate MaxAssets assets and generate enough liquidity
    for i in 0..asset_count {
        let asset = Asset::CategoricalOutcome(0u32.into(), i.saturated_into());
        assets.push(asset);

        T::AssetManager::deposit(asset, owner, asset_amount_unwrapped).unwrap()
    }

    assets
}

fn initialize_pool<T: Config>(
    caller: &T::AccountId,
    asset_count: Option<usize>,
    asset_amount: Option<BalanceOf<T>>,
    scoring_rule: ScoringRule,
    weights: Option<Vec<u128>>,
) -> (PoolId, Asset<MarketIdOf<T>>, Vec<Asset<MarketIdOf<T>>>, MarketIdOf<T>) {
    let asset_count_unwrapped: usize = {
        match asset_count {
            Some(ac) => ac,
            _ => T::MaxAssets::get().into(),
        }
    };

    let market_id = MarketIdOf::<T>::from(0u8);
    let assets = generate_assets::<T>(caller, asset_count_unwrapped, asset_amount);
    let some_weights = if weights.is_some() {
        weights
    } else {
        Some(vec![T::MinWeight::get(); asset_count_unwrapped])
    };
    let base_asset = *assets.last().unwrap();

    let pool_id = Pallet::<T>::create_pool(
        caller.clone(),
        assets.clone(),
        base_asset,
        market_id,
        scoring_rule,
        if scoring_rule == ScoringRule::CPMM { Some(Zero::zero()) } else { None },
        if scoring_rule == ScoringRule::CPMM { Some(T::MinLiquidity::get()) } else { None },
        if scoring_rule == ScoringRule::CPMM { some_weights } else { None },
    )
    .unwrap();

    (pool_id, base_asset, assets, market_id)
}
// Creates a pool containing `asset_count` (default: max assets) assets.
// Returns `PoolId`, `Vec<Asset<...>>`, ``MarketId`
fn bench_create_pool<T: Config>(
    caller: T::AccountId,
    asset_count: Option<usize>,
    asset_amount: Option<BalanceOf<T>>,
    scoring_rule: ScoringRule,
    subsidize: bool,
    weights: Option<Vec<u128>>,
) -> (u128, Vec<Asset<MarketIdOf<T>>>, MarketIdOf<T>) {
    let (pool_id, base_asset, assets, market_id) =
        initialize_pool::<T>(&caller, asset_count, asset_amount, scoring_rule, weights);

    if scoring_rule == ScoringRule::CPMM {
        let _ = Pallet::<T>::open_pool(pool_id);
    }

    if subsidize {
        let min_subsidy = T::MinSubsidy::get();
        T::AssetManager::deposit(base_asset, &caller, min_subsidy).unwrap();
        let _ = Call::<T>::pool_join_subsidy { pool_id, amount: T::MinSubsidy::get() }
            .dispatch_bypass_filter(RawOrigin::Signed(caller).into())
            .unwrap();
        let _ = Pallet::<T>::end_subsidy_phase(pool_id).unwrap();
    }

    (pool_id, assets, market_id)
}

benchmarks! {
    admin_clean_up_pool_cpmm_categorical {
        // We're excluding the case of two assets, which would leave us with only one outcome
        // token and cause `create_market` to error.
        let a in 3..T::MaxAssets::get().into();
        let category_count = (a - 1) as u16;
        let caller: T::AccountId = whitelisted_caller();
        let market_id = T::MarketCommons::push_market(
            Market {
                creation: MarketCreation::Permissionless,
                creator_fee: 0,
                creator: caller.clone(),
                market_type: MarketType::Categorical(category_count),
                dispute_mechanism: MarketDisputeMechanism::Authorized(caller.clone()),
                metadata: vec![0; 50],
                oracle: caller.clone(),
                period: MarketPeriod::Block(0u32.into()..1u32.into()),
                deadlines: Deadlines {
                    grace_period: 1_u32.into(),
                    oracle_duration: 1_u32.into(),
                    dispute_duration: 1_u32.into(),
                },
                report: None,
                resolved_outcome: None,
                scoring_rule: ScoringRule::CPMM,
                status: MarketStatus::Active,
            }
        )?;
        let pool_id: PoolId = 0;
        let _ = T::MarketCommons::insert_market_pool(market_id, pool_id);
        let _ = bench_create_pool::<T>(
            caller,
            Some(a as usize),
            None,
            ScoringRule::CPMM,
            false,
            None,
        );
        let _ = Pallet::<T>::mutate_pool(pool_id, |pool| {
            pool.pool_status = PoolStatus::Closed;
            Ok(())
        });
    }: admin_clean_up_pool(RawOrigin::Root, market_id, OutcomeReport::Categorical(0))

    admin_clean_up_pool_cpmm_scalar {
        let caller: T::AccountId = whitelisted_caller();
        let market_id = T::MarketCommons::push_market(
            Market {
                creation: MarketCreation::Permissionless,
                creator_fee: 0,
                creator: caller.clone(),
                market_type: MarketType::Scalar(0..=99),
                dispute_mechanism: MarketDisputeMechanism::Authorized(caller.clone()),
                metadata: vec![0; 50],
                oracle: caller.clone(),
                period: MarketPeriod::Block(0u32.into()..1u32.into()),
                deadlines: Deadlines {
                    grace_period: 1_u32.into(),
                    oracle_duration: 1_u32.into(),
                    dispute_duration: 1_u32.into(),
                },
                report: None,
                resolved_outcome: None,
                scoring_rule: ScoringRule::CPMM,
                status: MarketStatus::Active,
            }
        )?;
        let pool_id: PoolId = 0;
        let asset_count = 3;
        let _ = T::MarketCommons::insert_market_pool(market_id, pool_id);
        let _ = bench_create_pool::<T>(
            caller,
            Some(asset_count),
            None,
            ScoringRule::CPMM,
            false,
            None,
        );
        let _ = Pallet::<T>::mutate_pool(pool_id, |pool| {
            pool.pool_status = PoolStatus::Closed;
            Ok(())
        });
    }: admin_clean_up_pool(RawOrigin::Root, market_id, OutcomeReport::Scalar(33))

    // This is a worst-case benchmark for arbitraging a number of pools.
    apply_to_cached_pools_execute_arbitrage {
        let a in 0..63; // The number of cached pools.

        let caller: T::AccountId = whitelisted_caller();
        let asset_count = T::MaxAssets::get();
        let balance: BalanceOf<T> = (10_000_000_000 * BASE).saturated_into();
        let total_amount_required = balance * a.saturated_into();
        let assets = generate_assets::<T>(&caller, asset_count.into(), Some(total_amount_required));
        let base_asset = *assets.last().unwrap();

        // Set weights to [1, 1, ..., 1, 64].
        let outcome_count = asset_count - 1;
        let outcome_weight = T::MinWeight::get();
        let mut weights = vec![outcome_weight; (asset_count - 1) as usize];
        weights.push(outcome_count as u128 * outcome_weight);

        // Create `a` pools with huge balances and only a relatively small difference between them
        // to cause maximum iterations.
        for i in 0..a {
            let market_id = i.into();
            let pool_id = Pallet::<T>::create_pool(
                caller.clone(),
                assets.clone(),
                base_asset,
                market_id,
                ScoringRule::CPMM,
                Some(Zero::zero()),
                Some(balance),
                Some(weights.clone()),
            )
            .unwrap();

            let pool_account_id = Pallet::<T>::pool_account_id(pool_id);
            T::AssetManager::withdraw(
                *assets.last().unwrap(),
                &pool_account_id,
                balance / 9u8.saturated_into()
            )
            .unwrap();

            // Add enough funds for arbitrage to the prize pool.
            T::AssetManager::deposit(
                base_asset,
                &T::MarketCommons::market_account(market_id),
                balance,
            )
            .unwrap();

            PoolsCachedForArbitrage::<T>::insert(pool_id, ());
        }
        let mutation =
            |pool_id: PoolId| Pallet::<T>::execute_arbitrage(pool_id, ARBITRAGE_MAX_ITERATIONS);
    }: {
        Pallet::<T>::apply_to_cached_pools(a, mutation, Weight::MAX)
    } verify {
        // Ensure that all pools have been arbitraged.
        assert_eq!(PoolsCachedForArbitrage::<T>::iter().count(), 0);
    }

    apply_to_cached_pools_noop {
        let a in 0..63; // The number of cached pools.
        for i in 0..a {
            let pool_id: PoolId = i.into();
            PoolsCachedForArbitrage::<T>::insert(pool_id, ());
        }
        let noop = |_: PoolId| -> Result<Weight, DispatchError> { Ok(0) };
    }: {
        Pallet::<T>::apply_to_cached_pools(a, noop, Weight::MAX)
    }

    destroy_pool_in_subsidy_phase {
        // Total subsidy providers
        let a in 0..10;
        let min_assets_plus_base_asset = 3u16;

        // Create pool with assets
        let caller: T::AccountId = whitelisted_caller();
        let (pool_id, _, _) = bench_create_pool::<T>(
            caller,
            Some(min_assets_plus_base_asset.into()),
            None,
            ScoringRule::RikiddoSigmoidFeeMarketEma,
            false,
            None,
        );
        let amount = T::MinSubsidy::get();

        // Create a accounts, add MinSubsidy base assets and join subsidy
        let accounts =
            generate_accounts_with_assets::<T>(a, min_assets_plus_base_asset, amount).unwrap();

        // Join subsidy with each account
        for account in accounts {
            let _ = Call::<T>::pool_join_subsidy { pool_id, amount }
                .dispatch_bypass_filter(RawOrigin::Signed(account).into())?;
        }
    }: { Pallet::<T>::destroy_pool_in_subsidy_phase(pool_id)? }

    distribute_pool_share_rewards {
        // Total accounts
        let a in 10..20;
        // Total pool share holders
        let b in 0..10;

        let min_assets_plus_base_asset = 3u16;
        let amount = T::MinSubsidy::get();

        // Create a accounts, add MinSubsidy base assets
        let accounts =
            generate_accounts_with_assets::<T>(a, min_assets_plus_base_asset, amount).unwrap();

        let (pool_id, _, _) = bench_create_pool::<T>(
            accounts[0].clone(),
            Some(min_assets_plus_base_asset.into()),
            None,
            ScoringRule::RikiddoSigmoidFeeMarketEma,
            false,
            None,
        );

        // Join subsidy with b accounts
        for account in accounts[0..b.saturated_into()].iter() {
            let _ = Call::<T>::pool_join_subsidy { pool_id, amount }
                .dispatch_bypass_filter(RawOrigin::Signed(account.clone()).into())?;
        }

        Pallet::<T>::end_subsidy_phase(pool_id)?;
        let pool = <Pools<T>>::get(pool_id).unwrap();
    }: {
        Pallet::<T>::distribute_pool_share_rewards(
            &pool,
            pool_id,
            pool.base_asset,
            Asset::CategoricalOutcome(1337u16.saturated_into(), 1337u16.saturated_into()),
            &account("ScrapCollector", 0, 0)
        );
    }

    end_subsidy_phase {
        // Total assets
        let a in (T::MinAssets::get().into())..T::MaxAssets::get().into();
        // Total subsidy providers
        let b in 0..10;

        // Create pool with a assets
        let caller: T::AccountId = whitelisted_caller();
        let (pool_id, _, _) = bench_create_pool::<T>(
            caller,
            Some(a.saturated_into()),
            None,
            ScoringRule::RikiddoSigmoidFeeMarketEma,
            false,
            None,
        );
        let amount = T::MinSubsidy::get();

        // Create b accounts, add MinSubsidy base assets and join subsidy
        let accounts = generate_accounts_with_assets::<T>(b, a.saturated_into(), amount).unwrap();

        // Join subsidy with each account
        for account in accounts {
            let _ = Call::<T>::pool_join_subsidy { pool_id, amount }
                .dispatch_bypass_filter(RawOrigin::Signed(account).into())?;
        }
    }: { Pallet::<T>::end_subsidy_phase(pool_id)? }

    execute_arbitrage_buy_burn {
        let a in 2..T::MaxAssets::get().into(); // The number of assets in the pool.
        let b in 0..ARBITRAGE_MAX_ITERATIONS.try_into().unwrap(); // The number of iterations.
        let asset_count = a as usize;
        let iteration_count = b as usize;

        let caller: T::AccountId = whitelisted_caller();
        let balance: BalanceOf<T> = (10_000_000_000 * BASE).saturated_into();
        let assets = generate_assets::<T>(&caller, asset_count, Some(balance));
        let base_asset = *assets.last().unwrap();

        // Set weights to [1, 1, ..., 1, a].
        let outcome_count = asset_count - 1;
        let outcome_weight = T::MinWeight::get();
        let mut weights = vec![outcome_weight; outcome_count];
        weights.push(outcome_count as u128 * outcome_weight);

        // Create a pool with huge balances and only a relatively small difference between them to
        // cause at least 30 iterations.
        let market_id = 0u8.into();
        let pool_id = Pallet::<T>::create_pool(
            caller,
            assets.clone(),
            base_asset,
            market_id,
            ScoringRule::CPMM,
            Some(Zero::zero()),
            Some(balance),
            Some(weights.clone()),
        )
        .unwrap();
        let pool_account_id = Pallet::<T>::pool_account_id(pool_id);
        let asset = *assets.last().unwrap();
        T::AssetManager::withdraw(
            asset,
            &pool_account_id,
            balance / 9u8.saturated_into(),
        )
        .unwrap();
        let balance_before = T::AssetManager::free_balance(asset, &pool_account_id);

        // Add enough funds for arbitrage to the prize pool.
        T::AssetManager::deposit(
            base_asset,
            &T::MarketCommons::market_account(market_id),
            (u128::MAX / 2).saturated_into(),
        )
        .unwrap();
    }: {
        // In order to cap the number of iterations, we just set the `max_iterations` to `b`.
        Pallet::<T>::execute_arbitrage(pool_id, iteration_count)?
    } verify {
        // We don't care about the exact arbitrage amount and just want to verify that the correct
        // event was emitted.
        let arbitrage_amount =
            T::AssetManager::free_balance(asset, &pool_account_id) - balance_before;
        assert_last_event::<T>(Event::ArbitrageBuyBurn::<T>(pool_id, arbitrage_amount).into());
    }

    execute_arbitrage_mint_sell {
        let a in 2..T::MaxAssets::get().into(); // The number of assets in the pool.
        let b in 0..ARBITRAGE_MAX_ITERATIONS.try_into().unwrap(); // The number of iterations.
        let asset_count = a as usize;
        let iteration_count = b as usize;

        let caller: T::AccountId = whitelisted_caller();
        let balance: BalanceOf<T> = (10_000_000_000 * BASE).saturated_into();
        let assets = generate_assets::<T>(&caller, asset_count, Some(balance));
        let base_asset = *assets.last().unwrap();

        // Set weights to [1, 1, ..., 1, a].
        let outcome_count = asset_count - 1;
        let outcome_weight = T::MinWeight::get();
        let mut weights = vec![outcome_weight; outcome_count];
        weights.push(outcome_count as u128 * outcome_weight);

        // Create a pool with huge balances and only a relatively small difference between them to
        // cause at least 30 iterations.
        let pool_id = Pallet::<T>::create_pool(
            caller,
            assets.clone(),
            base_asset,
            0u8.into(),
            ScoringRule::CPMM,
            Some(Zero::zero()),
            Some(balance),
            Some(weights.clone()),
        )
        .unwrap();
        let pool_account_id = Pallet::<T>::pool_account_id(pool_id);
        for asset in assets.iter().filter(|a| **a != base_asset) {
            T::AssetManager::withdraw(
                *asset,
                &pool_account_id,
                balance / 9u8.saturated_into(),
            )
            .unwrap();
        }
        let asset = assets[0];
        let balance_before = T::AssetManager::free_balance(asset, &pool_account_id);
    }: {
        // In order to cap the number of iterations, we just set the `max_iterations` to `b`.
        Pallet::<T>::execute_arbitrage(pool_id, iteration_count)?
    } verify {
        // We don't care about the exact arbitrage amount and just want to verify that the correct
        // event was emitted.
        let arbitrage_amount =
            T::AssetManager::free_balance(asset, &pool_account_id) - balance_before;
        assert_last_event::<T>(Event::ArbitrageMintSell::<T>(pool_id, arbitrage_amount).into());
    }

    execute_arbitrage_skipped {
        let a in 2..T::MaxAssets::get().into(); // The number of assets in the pool.
        let asset_count = a as usize;

        let caller: T::AccountId = whitelisted_caller();
        let balance: BalanceOf<T> = (10_000_000_000 * BASE).saturated_into();
        let assets = generate_assets::<T>(&caller, asset_count, Some(balance));
        let base_asset = *assets.last().unwrap();

        // Set weights to [1, 1, ..., 1, a].
        let outcome_count = asset_count - 1;
        let outcome_weight = T::MinWeight::get();
        let mut weights = vec![outcome_weight; outcome_count];
        weights.push(outcome_count as u128 * outcome_weight);

        // Create a pool with equal balances to ensure that the total spot price is equal to 1.
        let pool_id = Pallet::<T>::create_pool(
            caller,
            assets,
            base_asset,
            0u8.into(),
            ScoringRule::CPMM,
            Some(Zero::zero()),
            Some(balance),
            Some(weights.clone()),
        )
        .unwrap();
    }: {
        // In order to cap the number of iterations, we just set the `max_iterations` to `b`.
        Pallet::<T>::execute_arbitrage(pool_id, ARBITRAGE_MAX_ITERATIONS)?
    } verify {
        assert_last_event::<T>(Event::ArbitrageSkipped::<T>(pool_id).into());
    }

    pool_exit {
        let a in 2 .. T::MaxAssets::get().into();
        let caller: T::AccountId = whitelisted_caller();
        let (pool_id, ..) = bench_create_pool::<T>(
            caller.clone(),
            Some(a as usize),
            Some(T::MinLiquidity::get() * 2u32.into()),
            ScoringRule::CPMM,
            false,
            None,
        );
        let pool_amount = T::MinLiquidity::get() / 2u32.into();
        let min_assets_out = vec![0u32.into(); a as usize];
    }: _(RawOrigin::Signed(caller), pool_id, pool_amount, min_assets_out)

    pool_exit_subsidy {
        let caller: T::AccountId = whitelisted_caller();
        let (pool_id, ..) = bench_create_pool::<T>(
            caller.clone(),
            None,
            Some(T::MinSubsidy::get()),
            ScoringRule::RikiddoSigmoidFeeMarketEma,
            false,
            None,
        );
        let _ = Call::<T>::pool_join_subsidy {
            pool_id,
            amount: T::MinSubsidy::get(),
        }
        .dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())?;
    }: _(RawOrigin::Signed(caller), pool_id, T::MinSubsidy::get())

    pool_exit_with_exact_asset_amount {
        let a = T::MaxAssets::get();
        let caller: T::AccountId = whitelisted_caller();
        let (pool_id, assets, ..) = bench_create_pool::<T>(
            caller.clone(),
            Some(a as usize),
            None,
            ScoringRule::CPMM,
            false,
            None,
        );
        let asset_amount: BalanceOf<T> = BASE.saturated_into();
        let pool_amount = T::MinLiquidity::get();
    }: _(RawOrigin::Signed(caller), pool_id, assets[0], asset_amount, pool_amount)

    pool_exit_with_exact_pool_amount {
        let a = T::MaxAssets::get();
        let caller: T::AccountId = whitelisted_caller();
        let (pool_id, assets, ..) = bench_create_pool::<T>(
            caller.clone(),
            Some(a as usize),
            None,
            ScoringRule::CPMM,
            false,
            None,
        );
        let min_asset_amount = 0u32.into();
        let pool_amount: BalanceOf<T> = CENT.saturated_into();
    }: _(RawOrigin::Signed(caller), pool_id, assets[0], pool_amount, min_asset_amount)

    pool_join {
        let a in 2 .. T::MaxAssets::get().into();
        let caller: T::AccountId = whitelisted_caller();
        let (pool_id, ..) = bench_create_pool::<T>(
            caller.clone(),
            Some(a as usize),
            Some(T::MinLiquidity::get() * 2u32.into()),
            ScoringRule::CPMM,
            false,
            None,
        );
        let pool_amount = T::MinLiquidity::get();
        let max_assets_in = vec![T::MinLiquidity::get(); a as usize];
    }: _(RawOrigin::Signed(caller), pool_id, pool_amount, max_assets_in)

    pool_join_subsidy {
        let caller: T::AccountId = whitelisted_caller();
        let (pool_id, ..) = bench_create_pool::<T>(
            caller.clone(),
            None,
            Some(T::MinSubsidy::get()),
            ScoringRule::RikiddoSigmoidFeeMarketEma,
            false,
            None,
        );
    }: _(RawOrigin::Signed(caller), pool_id, T::MinSubsidy::get())

    pool_join_with_exact_asset_amount {
        let a = T::MaxAssets::get();
        let caller: T::AccountId = whitelisted_caller();
        let (pool_id, assets, ..) = bench_create_pool::<T>(
            caller.clone(),
            Some(a as usize),
            Some(T::MinLiquidity::get() * 2u32.into()),
            ScoringRule::CPMM,
            false,
            None,
        );
        let asset_amount: BalanceOf<T> = BASE.saturated_into();
        let min_pool_amount = 0u32.into();
    }: _(RawOrigin::Signed(caller), pool_id, assets[0], asset_amount, min_pool_amount)

    pool_join_with_exact_pool_amount {
        let a = T::MaxAssets::get();
        let caller: T::AccountId = whitelisted_caller();
        let (pool_id, assets, ..) = bench_create_pool::<T>(
            caller.clone(),
            Some(a as usize),
            Some(T::MinLiquidity::get() * 2u32.into()),
            ScoringRule::CPMM,
            false,
            None,
        );
        let pool_amount = BASE.saturated_into();
        let max_asset_amount: BalanceOf<T> = T::MinLiquidity::get();
    }: _(RawOrigin::Signed(caller), pool_id, assets[0], pool_amount, max_asset_amount)

    clean_up_pool_categorical_without_reward_distribution {
        // Total possible outcomes
        let a in 3..T::MaxAssets::get().into();

        let amount = T::MinSubsidy::get();

        // Create pool with a assets
        let caller = whitelisted_caller();

        let (pool_id, _, _) = bench_create_pool::<T>(
            caller,
            Some(a.saturated_into()),
            None,
            ScoringRule::CPMM,
            false,
            None,
        );
        let _ = Pallet::<T>::mutate_pool(pool_id, |pool| {
            pool.pool_status = PoolStatus::Closed;
            Ok(())
        });
    }: {
        Pallet::<T>::clean_up_pool_categorical(
            pool_id,
            &OutcomeReport::Categorical(0),
            &account("ScrapCollector", 0, 0),
        )?;
    }

    swap_exact_amount_in_cpmm {
        // We're trying to get as many iterations in `bpow_approx` as possible. Experiments have
        // shown that y = 3/4, weight_ratio=1/2 (almost) maximizes the number of iterations for
        // calculating y^r within the set of values allowed in `swap_exact_amount_in` (see
        // `math::calc_out_given_in`). To get these values, we use the following parameters:
        // amount_in = 1/3 * balance_in, weight_in = 1, weight_out = 2.
        let asset_count = T::MaxAssets::get();
        let balance: BalanceOf<T> = T::MinLiquidity::get();
        let asset_amount_in: BalanceOf<T> = bmul(
            balance.saturated_into(),
            T::MaxInRatio::get().saturated_into(),
        )
        .unwrap()
        .saturated_into();
        let weight_in = T::MinWeight::get();
        let weight_out = 2 * weight_in;
        let mut weights = vec![weight_in; asset_count as usize];
        weights[asset_count as usize - 1] = weight_out;
        let caller: T::AccountId = whitelisted_caller();
        let (pool_id, assets, ..) = bench_create_pool::<T>(
            caller.clone(),
            Some(asset_count as usize),
            Some(balance),
            ScoringRule::CPMM,
            false,
            Some(weights),
        );
        let asset_in = assets[0];
        T::AssetManager::deposit(asset_in, &caller, u64::MAX.saturated_into()).unwrap();
        let asset_out = assets[asset_count as usize - 1];
        let min_asset_amount_out: Option<BalanceOf<T>> = Some(0u128.saturated_into());
        let max_price = Some(u128::MAX.saturated_into());
    }: swap_exact_amount_in(
        RawOrigin::Signed(caller),
        pool_id,
        asset_in,
        asset_amount_in,
        asset_out,
        min_asset_amount_out,
        max_price
    )

    swap_exact_amount_in_rikiddo {
        let a in 3 .. T::MaxAssets::get().into();
        let caller: T::AccountId = whitelisted_caller();
        let (pool_id, assets, ..) = bench_create_pool::<T>(
            caller.clone(),
            Some(a as usize),
            Some(BASE.saturated_into()),
            ScoringRule::RikiddoSigmoidFeeMarketEma,
            true,
            None,
        );
        let asset_amount_in: BalanceOf<T> = BASE.saturated_into();
        let min_asset_amount_out: Option<BalanceOf<T>> = Some(0u32.into());
        let max_price = Some((BASE * 1024).saturated_into());
    }: swap_exact_amount_in(
        RawOrigin::Signed(caller),
        pool_id,
        assets[0],
        asset_amount_in,
        *assets.last().unwrap(),
        min_asset_amount_out,
        max_price
    )

    swap_exact_amount_out_cpmm {
        // We're trying to get as many iterations in `bpow_approx` as possible. Experiments have
        // shown that y = 3/2, weight_ratio=1/4 (almost) maximizes the number of iterations for
        // calculating y^r within the set of values allowed in `swap_exact_amount_out` (see
        // `math::calc_in_given_out`). To get these values, we use the following parameters:
        // amount_out = 1/3 * balance_out, weight_out = 1, weight_in = 4.
        let asset_count = T::MaxAssets::get();
        let balance: BalanceOf<T> = T::MinLiquidity::get();
        let asset_amount_out: BalanceOf<T> = bmul(
            balance.saturated_into(),
            T::MaxOutRatio::get().saturated_into(),
        )
        .unwrap()
        .saturated_into();
        let weight_out = T::MinWeight::get();
        let weight_in = 4 * weight_out;
        let mut weights = vec![weight_out; asset_count as usize];
        weights[0] = weight_in;
        let caller: T::AccountId = whitelisted_caller();
        let (pool_id, assets, ..) = bench_create_pool::<T>(
            caller.clone(),
            Some(asset_count as usize),
            Some(balance),
            ScoringRule::CPMM,
            false,
            Some(weights),
        );
        let asset_in = assets[0];
        T::AssetManager::deposit(asset_in, &caller, u64::MAX.saturated_into()).unwrap();
        let asset_out = assets[asset_count as usize - 1];
        let max_asset_amount_in: Option<BalanceOf<T>> = Some(u128::MAX.saturated_into());
        let max_price = Some(u128::MAX.saturated_into());
    }: swap_exact_amount_out(
        RawOrigin::Signed(caller),
        pool_id,
        asset_in,
        max_asset_amount_in,
        asset_out,
        asset_amount_out,
        max_price
    )

    swap_exact_amount_out_rikiddo {
        let a in 3 .. T::MaxAssets::get().into();
        let caller: T::AccountId = whitelisted_caller();
        let (pool_id, assets, ..) = bench_create_pool::<T>(
            caller.clone(),
            Some(a as usize),
            Some(BASE.saturated_into()),
            ScoringRule::RikiddoSigmoidFeeMarketEma,
            true,
            None,
        );
        let max_asset_amount_in: Option<BalanceOf<T>> = Some((BASE * 1024).saturated_into());
        let asset_amount_out: BalanceOf<T> = BASE.saturated_into();
        let max_price = Some((BASE * 1024).saturated_into());
    }: swap_exact_amount_out(
        RawOrigin::Signed(caller),
        pool_id,
        *assets.last().unwrap(),
        max_asset_amount_in,
        assets[0],
        asset_amount_out,
        max_price
    )

    open_pool {
        let a in 2..T::MaxAssets::get().into();

        let caller: T::AccountId = whitelisted_caller();
        let (pool_id, ..) = initialize_pool::<T>(
            &caller,
            Some(a as usize),
            None,
            ScoringRule::CPMM,
            None,
        );
        let pool = Pallet::<T>::pool_by_id(pool_id).unwrap();
        assert_eq!(pool.pool_status, PoolStatus::Initialized);
    }: {
        Pallet::<T>::open_pool(pool_id).unwrap();
    } verify {
        let pool = Pallet::<T>::pool_by_id(pool_id).unwrap();
        assert_eq!(pool.pool_status, PoolStatus::Active);
    }

    close_pool {
        let a in 2..T::MaxAssets::get().into();

        let caller: T::AccountId = whitelisted_caller();
        let (pool_id, ..) = bench_create_pool::<T>(
            caller,
            Some(a as usize),
            None,
            ScoringRule::CPMM,
            false,
            None,
        );
        let pool = Pallet::<T>::pool_by_id(pool_id).unwrap();
        assert_eq!(pool.pool_status, PoolStatus::Active);
    }: {
        Pallet::<T>::close_pool(pool_id).unwrap();
    } verify {
        let pool = Pallet::<T>::pool_by_id(pool_id).unwrap();
        assert_eq!(pool.pool_status, PoolStatus::Closed);
    }

    destroy_pool {
        let a in 2..T::MaxAssets::get().into();

        let caller: T::AccountId = whitelisted_caller();
        let (pool_id, ..) = bench_create_pool::<T>(
            caller,
            Some(a as usize),
            None,
            ScoringRule::CPMM,
            false,
            None,
        );
        assert!(Pallet::<T>::pool_by_id(pool_id).is_ok());
    }: {
        Pallet::<T>::destroy_pool(pool_id).unwrap();
    } verify {
        assert!(Pallet::<T>::pool_by_id(pool_id).is_err());
        assert_last_event::<T>(Event::PoolDestroyed::<T>(
            pool_id,
        ).into());
    }
}

impl_benchmark_test_suite!(Swaps, crate::mock::ExtBuilder::default().build(), crate::mock::Runtime);
