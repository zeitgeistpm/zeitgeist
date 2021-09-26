// Auto-generated code is a no man's land
#![allow(clippy::integer_arithmetic, clippy::indexing_slicing)]
#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::Config;
#[cfg(test)]
use crate::Pallet as Swaps;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, vec, whitelisted_caller, Vec};
use frame_support::{dispatch::UnfilteredDispatchable, traits::Get};
use frame_system::RawOrigin;
use orml_traits::MultiCurrency;
use sp_runtime::traits::{SaturatedConversion, Zero};
use zeitgeist_primitives::{
    constants::BASE,
    traits::Swaps as _,
    types::{Asset, MarketType, OutcomeReport, ScoringRule},
};

// Generates ``asset_count`` assets
fn generate_assets<T: Config>(
    owner: &T::AccountId,
    asset_count: usize,
    asset_amount: Option<BalanceOf<T>>,
    deposit: bool,
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

        if deposit {
            T::Shares::deposit(asset, owner, asset_amount_unwrapped).unwrap()
        } else if !deposit && i == asset_count - 1 {
            T::Shares::deposit(asset, owner, asset_amount_unwrapped).unwrap()
        };
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
) -> (u128, Vec<Asset<T::MarketId>>, T::MarketId) {
    let asset_count_unwrapped: usize = {
        match asset_count {
            Some(ac) => ac,
            _ => T::MaxAssets::get().into(),
        }
    };

    let market_id = T::MarketId::from(0u8);
    let deposit = if scoring_rule == ScoringRule::CPMM { true } else { false };
    let assets = generate_assets::<T>(&caller, asset_count_unwrapped, asset_amount, deposit);
    let weights = vec![T::MinWeight::get(); asset_count_unwrapped];
    let base_asset = Some(*assets.last().unwrap());

    let _ = Pallet::<T>::create_pool(
        caller,
        assets.clone(),
        base_asset,
        market_id,
        scoring_rule,
        if scoring_rule == ScoringRule::CPMM { Some(Zero::zero()) } else { None },
        if scoring_rule == ScoringRule::CPMM { Some(weights) } else { None },
    )
    .unwrap();
    let pool_id = <NextPoolId<T>>::get() - 1;

    (pool_id, assets, market_id)
}

benchmarks! {
    admin_set_pool_as_stale {
        let caller: T::AccountId = whitelisted_caller();
        let (pool_id, ..) = bench_create_pool::<T>(caller, Some(T::MaxAssets::get().into()), None, ScoringRule::CPMM);
    }: _(RawOrigin::Root, MarketType::Categorical(0), pool_id as _, OutcomeReport::Categorical(0))

    pool_exit {
        let a in 2 .. T::MaxAssets::get().into();
        let caller: T::AccountId = whitelisted_caller();
        let (pool_id, ..) = bench_create_pool::<T>(caller.clone(), Some(a as usize), None, ScoringRule::CPMM);
        let pool_amount = T::MinLiquidity::get();
        let min_assets_out = vec![T::MinLiquidity::get(); a as usize];
    }: _(RawOrigin::Signed(caller), pool_id, pool_amount, min_assets_out)

    pool_exit_subsidy {
        let caller: T::AccountId = whitelisted_caller();
        let (pool_id, ..) = bench_create_pool::<T>(caller.clone(), None, Some(T::MinSubsidy::get()), ScoringRule::RikiddoSigmoidFeeMarketEma);
        let _ = Call::<T>::pool_join_subsidy(pool_id, T::MinSubsidy::get())
            .dispatch_bypass_filter(RawOrigin::Signed(caller.clone()).into())?;
    }: _(RawOrigin::Signed(caller), pool_id, T::MinSubsidy::get())

    pool_exit_with_exact_asset_amount {
        let a = T::MaxAssets::get();
        let caller: T::AccountId = whitelisted_caller();
        let (pool_id, assets, ..) = bench_create_pool::<T>(caller.clone(), Some(a as usize), None, ScoringRule::CPMM);
        let asset_amount: BalanceOf<T> = BASE.saturated_into();
        let pool_amount = T::MinLiquidity::get();
    }: _(RawOrigin::Signed(caller), pool_id, assets[0], asset_amount, pool_amount)

    pool_exit_with_exact_pool_amount {
        let a = T::MaxAssets::get();
        let caller: T::AccountId = whitelisted_caller();
        let (pool_id, assets, ..) = bench_create_pool::<T>(caller.clone(), Some(a as usize), None, ScoringRule::CPMM);
        let asset_amount: BalanceOf<T> = BASE.saturated_into();
        let pool_amount = 0u32.into();
    }: _(RawOrigin::Signed(caller), pool_id, assets[0], asset_amount, pool_amount)

    pool_join {
        let a in 2 .. T::MaxAssets::get().into();
        let caller: T::AccountId = whitelisted_caller();
        let (pool_id, ..) = bench_create_pool::<T>(caller.clone(), Some(a as usize),
            Some(T::MinLiquidity::get() * 2u32.into()), ScoringRule::CPMM);
        let pool_amount = T::MinLiquidity::get();
        let max_assets_in = vec![T::MinLiquidity::get(); a as usize];
    }: _(RawOrigin::Signed(caller), pool_id, pool_amount, max_assets_in)

    pool_join_subsidy {
        let caller: T::AccountId = whitelisted_caller();
        let (pool_id, ..) = bench_create_pool::<T>(caller.clone(), None, Some(T::MinSubsidy::get()), ScoringRule::RikiddoSigmoidFeeMarketEma);
    }: _(RawOrigin::Signed(caller), pool_id, T::MinSubsidy::get())

    pool_join_with_exact_asset_amount {
        let a = T::MaxAssets::get();
        let caller: T::AccountId = whitelisted_caller();
        let (pool_id, assets, ..) = bench_create_pool::<T>(caller.clone(), Some(a as usize),
            Some(T::MinLiquidity::get() * 2u32.into()), ScoringRule::CPMM);
        let asset_amount: BalanceOf<T> = BASE.saturated_into();
        let min_pool_amount = 0u32.into();
    }: _(RawOrigin::Signed(caller), pool_id, assets[0], asset_amount, min_pool_amount)

    pool_join_with_exact_pool_amount {
        let a = T::MaxAssets::get();
        let caller: T::AccountId = whitelisted_caller();
        let (pool_id, assets, ..) = bench_create_pool::<T>(caller.clone(), Some(a as usize),
            Some(T::MinLiquidity::get() * 2u32.into()), ScoringRule::CPMM);
        let pool_amount = BASE.saturated_into();
        let max_asset_amount: BalanceOf<T> = T::MinLiquidity::get();
    }: _(RawOrigin::Signed(caller), pool_id, assets[0], pool_amount, max_asset_amount)

    swap_exact_amount_in {
        let a = T::MaxAssets::get();
        let caller: T::AccountId = whitelisted_caller();
        let (pool_id, assets, ..) = bench_create_pool::<T>(caller.clone(), Some(a as usize),
            Some(T::MinLiquidity::get() * 2u32.into()), ScoringRule::CPMM);
        let asset_amount_in: BalanceOf<T> = BASE.saturated_into();
        let min_asset_amount_out: BalanceOf<T> = 0u32.into();
        let max_price = T::MinLiquidity::get() * 2u32.into();
    }: _(RawOrigin::Signed(caller), pool_id, assets[0], asset_amount_in,
            assets[T::MaxAssets::get() as usize - 1], min_asset_amount_out, max_price)

    swap_exact_amount_out {
        let a = T::MaxAssets::get();
        let caller: T::AccountId = whitelisted_caller();
        let (pool_id, assets, ..) = bench_create_pool::<T>(caller.clone(), Some(a as usize),
            Some(T::MinLiquidity::get() * 2u32.into()), ScoringRule::CPMM);
        let max_ammount_asset_in: BalanceOf<T> = T::MinLiquidity::get();
        let asset_amount_out: BalanceOf<T> = BASE.saturated_into();
        let max_price = T::MinLiquidity::get() * 2u32.into();
    }: _(RawOrigin::Signed(caller), pool_id, assets[0], max_ammount_asset_in,
            assets[T::MaxAssets::get() as usize - 1], asset_amount_out, max_price)
}

impl_benchmark_test_suite!(Swaps, crate::mock::ExtBuilder::default().build(), crate::mock::Runtime);
