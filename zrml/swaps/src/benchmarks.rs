#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::Config;
#[cfg(test)]
use crate::Pallet as Swaps;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, vec, whitelisted_caller, Vec};
use frame_support::dispatch::UnfilteredDispatchable;
use frame_support::traits::Get;
use frame_system::RawOrigin;
use orml_traits::MultiCurrency;
use sp_runtime::traits::SaturatedConversion;
use zeitgeist_primitives::{constants::BASE, types::Asset};

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
        T::Shares::deposit(asset, owner, asset_amount_unwrapped).unwrap();
    }

    assets
}

// Creates a pool containing `asset_count` (default: max assets) assets.
// Returns ``pool_id``, ``Vec<Asset<...>>``
fn bench_create_pool<T: Config>(
    caller: T::AccountId,
    asset_count: Option<usize>,
    asset_amount: Option<BalanceOf<T>>,
) -> (u128, Vec<Asset<T::MarketId>>) {
    let asset_count_unwrapped: usize = {
        match asset_count {
            Some(ac) => ac,
            _ => T::MaxAssets::get(),
        }
    };

    let assets = generate_assets::<T>(&caller, asset_count_unwrapped, asset_amount);
    let weights = vec![T::MinWeight::get(); asset_count_unwrapped];
    let _ = Call::<T>::create_pool(assets.clone(), weights)
        .dispatch_bypass_filter(RawOrigin::Signed(caller).into());
    (<NextPoolId<T>>::get() - 1, assets)
}

benchmarks! {
    create_pool {
        // Step through every possible amount of assets <= MaxAssets
        let a in 0..T::MaxAssets::get() as u32;
        let caller = whitelisted_caller();
        let assets = generate_assets::<T>(&caller, a as usize, None);
        let weights = vec![T::MinWeight::get(); a as usize];

    }: _(RawOrigin::Signed(caller), assets, weights)

    pool_exit {
        let a in 0..T::MaxAssets::get() as u32;
        let caller: T::AccountId = whitelisted_caller();
        let (pool_id, _) = bench_create_pool::<T>(caller.clone(), Some(a as usize), None);
        let pool_amount = T::MinLiquidity::get();
        let min_assets_out = vec![T::MinLiquidity::get(); a as usize];
    }: _(RawOrigin::Signed(caller), pool_id, pool_amount, min_assets_out)

    pool_exit_with_exact_asset_amount {
        let a = T::MaxAssets::get();
        let caller: T::AccountId = whitelisted_caller();
        let (pool_id, assets) = bench_create_pool::<T>(caller.clone(), Some(a as usize), None);
        let asset_amount: BalanceOf<T> = BASE.saturated_into();
        let pool_amount = T::MinLiquidity::get();
    }: _(RawOrigin::Signed(caller), pool_id, assets[0], asset_amount, pool_amount)

    pool_exit_with_exact_pool_amount {
        let a = T::MaxAssets::get();
        let caller: T::AccountId = whitelisted_caller();
        let (pool_id, assets) = bench_create_pool::<T>(caller.clone(), Some(a as usize), None);
        let asset_amount: BalanceOf<T> = BASE.saturated_into();
        let pool_amount = 0u32.into();
    }: _(RawOrigin::Signed(caller), pool_id, assets[0], asset_amount, pool_amount)

    pool_join {
        let a in 0..T::MaxAssets::get() as u32;
        let caller: T::AccountId = whitelisted_caller();
        let (pool_id, _) = bench_create_pool::<T>(caller.clone(), Some(a as usize),
            Some(T::MinLiquidity::get() * 2u32.into()));
        let pool_amount = T::MinLiquidity::get();
        let max_assets_in = vec![T::MinLiquidity::get(); a as usize];
    }: _(RawOrigin::Signed(caller), pool_id, pool_amount, max_assets_in)

    pool_join_with_exact_asset_amount {
        let a = T::MaxAssets::get();
        let caller: T::AccountId = whitelisted_caller();
        let (pool_id, assets) = bench_create_pool::<T>(caller.clone(), Some(a as usize),
            Some(T::MinLiquidity::get() * 2u32.into()));
        let asset_amount: BalanceOf<T> = BASE.saturated_into();
        let min_pool_amount = 0u32.into();
    }: _(RawOrigin::Signed(caller), pool_id, assets[0], asset_amount, min_pool_amount)

    pool_join_with_exact_pool_amount {
        let a = T::MaxAssets::get();
        let caller: T::AccountId = whitelisted_caller();
        let (pool_id, assets) = bench_create_pool::<T>(caller.clone(), Some(a as usize),
            Some(T::MinLiquidity::get() * 2u32.into()));
        let pool_amount = BASE.saturated_into();
        let max_asset_amount: BalanceOf<T> = T::MinLiquidity::get();
    }: _(RawOrigin::Signed(caller), pool_id, assets[0], pool_amount, max_asset_amount)

    swap_exact_amount_in {
        let a = T::MaxAssets::get();
        let caller: T::AccountId = whitelisted_caller();
        let (pool_id, assets) = bench_create_pool::<T>(caller.clone(), Some(a as usize),
            Some(T::MinLiquidity::get() * 2u32.into()));
        let asset_amount_in: BalanceOf<T> = BASE.saturated_into();
        let min_asset_amount_out: BalanceOf<T> = 0u32.into();
        let max_price = T::MinLiquidity::get() * 2u32.into();
    }: _(RawOrigin::Signed(caller), pool_id, assets[0], asset_amount_in,
            assets[T::MaxAssets::get() as usize - 1], min_asset_amount_out, max_price)

    swap_exact_amount_out {
        let a = T::MaxAssets::get();
        let caller: T::AccountId = whitelisted_caller();
        let (pool_id, assets) = bench_create_pool::<T>(caller.clone(), Some(a as usize),
            Some(T::MinLiquidity::get() * 2u32.into()));
        let max_ammount_asset_in: BalanceOf<T> = T::MinLiquidity::get();
        let asset_amount_out: BalanceOf<T> = BASE.saturated_into();
        let max_price = T::MinLiquidity::get() * 2u32.into();
    }: _(RawOrigin::Signed(caller), pool_id, assets[0], max_ammount_asset_in,
            assets[T::MaxAssets::get() as usize - 1], asset_amount_out, max_price)
}

impl_benchmark_test_suite!(
    Swaps,
    crate::mock::ExtBuilder::default().build(),
    crate::mock::Runtime
);
