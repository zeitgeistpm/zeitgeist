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
use crate::{fixed::bmul, Config};
use frame_benchmarking::{
    account, benchmarks, impl_benchmark_test_suite, vec, whitelisted_caller, Vec,
};
use frame_support::{dispatch::UnfilteredDispatchable, traits::Get};
use frame_system::RawOrigin;
use orml_traits::MultiCurrency;
use sp_runtime::traits::{SaturatedConversion, Zero};
use zeitgeist_primitives::{
    constants::BASE,
    traits::Swaps as _,
    types::{
        Asset, Market, MarketCreation, MarketDisputeMechanism, MarketPeriod, MarketStatus,
        MarketType, OutcomeReport, PoolId, PoolStatus, ScoringRule,
    },
};
use zrml_market_commons::MarketCommonsPalletApi;

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
            let asset = Asset::CategoricalOutcome::<T::MarketId>(0u32.into(), j);
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
) -> Vec<Asset<T::MarketId>> {
    let mut assets: Vec<Asset<T::MarketId>> = Vec::new();

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

// Creates a pool containing `asset_count` (default: max assets) assets.
// Returns `PoolId`, `Vec<Asset<...>>`, ``MarketId`
fn bench_create_pool<T: Config>(
    caller: T::AccountId,
    asset_count: Option<usize>,
    asset_amount: Option<BalanceOf<T>>,
    scoring_rule: ScoringRule,
    subsidize: bool,
    weights: Option<Vec<u128>>,
) -> (u128, Vec<Asset<T::MarketId>>, T::MarketId) {
    let asset_count_unwrapped: usize = {
        match asset_count {
            Some(ac) => ac,
            _ => T::MaxAssets::get().into(),
        }
    };

    let market_id = T::MarketId::from(0u8);
    let assets = generate_assets::<T>(&caller, asset_count_unwrapped, asset_amount);
    let some_weights = if weights.is_some() {
        weights
    } else {
        Some(vec![T::MinWeight::get(); asset_count_unwrapped])
    };
    let base_asset = *assets.last().unwrap();

    let _ = Pallet::<T>::create_pool(
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
    let pool_id = <NextPoolId<T>>::get() - 1;

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
        // token, which makes no sense in the context of prediction markets.
        let a in 3..T::MaxAssets::get().into();
        let category_count = (a - 1) as u16;
        let caller: T::AccountId = whitelisted_caller();
        T::MarketCommons::push_market(
            Market {
                creation: MarketCreation::Permissionless,
                creator_fee: 0,
                creator: caller.clone(),
                market_type: MarketType::Categorical(category_count),
                dispute_mechanism: MarketDisputeMechanism::Authorized(caller.clone()),
                metadata: vec![0; 50],
                oracle: caller.clone(),
                period: MarketPeriod::Block(0u32.into()..1u32.into()),
                report: None,
                resolved_outcome: None,
                scoring_rule: ScoringRule::CPMM,
                status: MarketStatus::Active,
            }
        )?;
        let market_id = T::MarketCommons::latest_market_id()?;
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

        let call = Call::<T>::admin_clean_up_pool {
            market_id,
            outcome_report: OutcomeReport::Categorical(0),
        };
    }: {
        call.dispatch_bypass_filter(RawOrigin::Root.into())?;
    }

    admin_clean_up_pool_cpmm_scalar {
        let caller: T::AccountId = whitelisted_caller();
        T::MarketCommons::push_market(
            Market {
                creation: MarketCreation::Permissionless,
                creator_fee: 0,
                creator: caller.clone(),
                market_type: MarketType::Scalar(0..=99),
                dispute_mechanism: MarketDisputeMechanism::Authorized(caller.clone()),
                metadata: vec![0; 50],
                oracle: caller.clone(),
                period: MarketPeriod::Block(0u32.into()..1u32.into()),
                report: None,
                resolved_outcome: None,
                scoring_rule: ScoringRule::CPMM,
                status: MarketStatus::Active,
            }
        )?;
        let market_id = T::MarketCommons::latest_market_id()?;
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

        let call = Call::<T>::admin_clean_up_pool {
            market_id,
            outcome_report: OutcomeReport::Scalar(33),
        };
    }: {
        call.dispatch_bypass_filter(RawOrigin::Root.into())?;
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
        let accounts = generate_accounts_with_assets::<T>(
            b,
            a.saturated_into(),
            amount,
        ).unwrap();

        // Join subsidy with each account
        for account in accounts {
            let _ = Call::<T>::pool_join_subsidy { pool_id, amount }
                .dispatch_bypass_filter(RawOrigin::Signed(account).into())?;
        }
    }: { Pallet::<T>::end_subsidy_phase(pool_id)? }

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
        let accounts = generate_accounts_with_assets::<T>(
            a,
            min_assets_plus_base_asset,
            amount,
        ).unwrap();

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
        let accounts = generate_accounts_with_assets::<T>(
            a,
            min_assets_plus_base_asset,
            amount,
        ).unwrap();

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
        let _ = Call::<T>::pool_join_subsidy { pool_id, amount: T::MinSubsidy::get() }
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
        let asset_amount: BalanceOf<T> = BASE.saturated_into();
        let pool_amount = 0u32.into();
    }: _(RawOrigin::Signed(caller), pool_id, assets[0], asset_amount, pool_amount)

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
        let asset_count = T::MaxAssets::get();
        let balance: BalanceOf<T> = T::MinLiquidity::get();
        let asset_amount_in: BalanceOf<T> = bmul(
            balance.saturated_into(),
            T::MaxInRatio::get().saturated_into(),
        )
        .unwrap()
        .saturated_into();
        let weight_in = T::MinWeight::get();
        let weight_out = 10 * weight_in;
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
    }: swap_exact_amount_in(RawOrigin::Signed(caller), pool_id, assets[0], asset_amount_in,
            *assets.last().unwrap(), min_asset_amount_out, max_price)

    swap_exact_amount_out_cpmm {
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
        let max_asset_amount_in: Option<BalanceOf<T>> = Some(T::MinLiquidity::get());
        let asset_amount_out: BalanceOf<T> = BASE.saturated_into();
        let max_price = Some(T::MinLiquidity::get() * 2u32.into());
    }: swap_exact_amount_out(RawOrigin::Signed(caller), pool_id, assets[0], max_asset_amount_in,
            assets[T::MaxAssets::get() as usize - 1], asset_amount_out, max_price)

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
    }: swap_exact_amount_out(RawOrigin::Signed(caller), pool_id, *assets.last().unwrap(), max_asset_amount_in,
           assets[0], asset_amount_out, max_price)
}

impl_benchmark_test_suite!(Swaps, crate::mock::ExtBuilder::default().build(), crate::mock::Runtime);
