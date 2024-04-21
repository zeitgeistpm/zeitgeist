// Copyright 2022-2024 Forecasting Technologies LTD.
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
#![allow(clippy::arithmetic_side_effects, clippy::indexing_slicing)]
#![cfg(feature = "runtime-benchmarks")]

use super::*;
#[cfg(test)]
use crate::Pallet as Swaps;
use crate::{types::PoolStatus, AssetOf, Config, Event, MAX_IN_RATIO, MAX_OUT_RATIO};
use alloc::{vec, vec::Vec};
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_support::traits::Get;
use frame_system::RawOrigin;
use orml_traits::MultiCurrency;
use sp_runtime::traits::{SaturatedConversion, Zero};
use zeitgeist_primitives::{
    constants::{BASE, CENT},
    math::fixed::FixedMul,
    traits::{Swaps as SwapsApi, ZeitgeistAssetEnumerator},
};

const LIQUIDITY: u128 = 100 * BASE;

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
    frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

fn generate_assets<T>(
    owner: &T::AccountId,
    asset_count: usize,
    opt_asset_amount: Option<BalanceOf<T>>,
) -> (Vec<AssetOf<T>>, BalanceOf<T>)
where
    T: Config,
    T::Asset: ZeitgeistAssetEnumerator<u128>,
{
    let mut assets: Vec<AssetOf<T>> = Vec::new();
    let asset_amount = opt_asset_amount.unwrap_or_else(|| LIQUIDITY.saturated_into());
    for i in 0..asset_count {
        let asset = T::Asset::create_asset_id(i as u128);
        assets.push(asset);
        T::AssetManager::deposit(asset, owner, asset_amount).unwrap()
    }
    (assets, asset_amount)
}

// Creates a pool containing `asset_count` (default: max assets) assets.
// Returns `PoolId`, `Vec<Asset<...>>`, ``MarketId`
fn bench_create_pool<T>(
    caller: T::AccountId,
    asset_count: usize,
    opt_asset_amount: Option<BalanceOf<T>>,
    opt_weights: Option<Vec<BalanceOf<T>>>,
    open: bool,
) -> (u128, Vec<AssetOf<T>>, BalanceOf<T>)
where
    T: Config,
    T::Asset: ZeitgeistAssetEnumerator<u128>,
{
    let (assets, asset_amount) = generate_assets::<T>(&caller, asset_count, opt_asset_amount);
    let weights = opt_weights.unwrap_or_else(|| vec![T::MinWeight::get(); asset_count]);
    let pool_id = Pallet::<T>::create_pool(
        caller.clone(),
        assets.clone(),
        Zero::zero(),
        asset_amount,
        weights,
    )
    .unwrap();
    if open {
        Pallet::<T>::open_pool(pool_id).unwrap();
    }
    (pool_id, assets, asset_amount)
}

benchmarks! {
    where_clause {
        where
            T::Asset: ZeitgeistAssetEnumerator<u128>,
    }

    pool_exit {
        let a in 2 .. T::MaxAssets::get().into();
        let caller: T::AccountId = whitelisted_caller();
        let (pool_id, _, asset_amount) =
            bench_create_pool::<T>(caller.clone(), a as usize, None, None, true);
        let pool_amount = (asset_amount / 2u8.into()).saturated_into();
        let min_assets_out = vec![0u32.into(); a as usize];
    }: _(RawOrigin::Signed(caller), pool_id, pool_amount, min_assets_out)

    pool_exit_with_exact_asset_amount {
        let a = T::MaxAssets::get();
        let caller: T::AccountId = whitelisted_caller();
        let (pool_id, assets, asset_amount) =
            bench_create_pool::<T>(caller.clone(), a as usize, None, None, true);
        let asset_amount: BalanceOf<T> = BASE.saturated_into();
        let pool_amount = asset_amount.saturated_into();
    }: _(RawOrigin::Signed(caller), pool_id, assets[0], asset_amount, pool_amount)

    pool_exit_with_exact_pool_amount {
        let a = T::MaxAssets::get();
        let caller: T::AccountId = whitelisted_caller();
        let (pool_id, assets, ..) =
            bench_create_pool::<T>(caller.clone(), a as usize, None, None, true);
        let min_asset_amount = 0u32.into();
        let pool_amount: BalanceOf<T> = CENT.saturated_into();
    }: _(RawOrigin::Signed(caller), pool_id, assets[0], pool_amount, min_asset_amount)

    pool_join {
        let a in 2 .. T::MaxAssets::get().into(); // asset_count
        let caller: T::AccountId = whitelisted_caller();
        let (pool_id, _, asset_amount) =
            bench_create_pool::<T>(caller.clone(), a as usize, None, None, true);
        generate_assets::<T>(&caller, a as usize, Some(asset_amount));
        let pool_amount = asset_amount.saturated_into();
        let max_assets_in = vec![u128::MAX.saturated_into(); a as usize];
    }: _(RawOrigin::Signed(caller), pool_id, pool_amount, max_assets_in)

    pool_join_with_exact_asset_amount {
        let a = T::MaxAssets::get();
        let caller: T::AccountId = whitelisted_caller();
        let (pool_id, assets, ..) =
            bench_create_pool::<T>(caller.clone(), a as usize, None, None, true);
        let asset_amount: BalanceOf<T> = BASE.saturated_into();
        generate_assets::<T>(&caller, a as usize, Some(asset_amount));
        let min_pool_amount = 0u32.into();
    }: _(RawOrigin::Signed(caller), pool_id, assets[0], asset_amount, min_pool_amount)

    pool_join_with_exact_pool_amount {
        // TODO(#1219): Explicitly state liquidity here!
        let a = T::MaxAssets::get();
        let caller: T::AccountId = whitelisted_caller();
        let (pool_id, assets, asset_amount) =
            bench_create_pool::<T>(caller.clone(), a as usize, None, None, true);
        let pool_amount = BASE.saturated_into();
        generate_assets::<T>(&caller, a as usize, Some(asset_amount));
        let max_asset_amount: BalanceOf<T> = u128::MAX.saturated_into();
    }: _(RawOrigin::Signed(caller), pool_id, assets[0], pool_amount, max_asset_amount)

    swap_exact_amount_in {
        // We're trying to get as many iterations in `bpow_approx` as possible. Experiments have
        // shown that y = 3/4, weight_ratio=1/2 (almost) maximizes the number of iterations for
        // calculating y^r within the set of values allowed in `swap_exact_amount_in` (see
        // `math::calc_out_given_in`). To get these values, we use the following parameters:
        // amount_in = 1/3 * balance_in, weight_in = 1, weight_out = 2.
        let asset_count = T::MaxAssets::get();
        let balance: BalanceOf<T> = LIQUIDITY.saturated_into();
        let asset_amount_in: BalanceOf<T> = balance.bmul(MAX_IN_RATIO.saturated_into()).unwrap();
        let weight_in = T::MinWeight::get();
        let weight_out = weight_in * 2u8.into();
        let mut weights = vec![weight_in; asset_count as usize];
        weights[asset_count as usize - 1] = weight_out;
        let caller: T::AccountId = whitelisted_caller();
        let (pool_id, assets, market_id) = bench_create_pool::<T>(
            caller.clone(),
            asset_count as usize,
            Some(balance),
            Some(weights),
            true,
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

    swap_exact_amount_out {
        // We're trying to get as many iterations in `bpow_approx` as possible. Experiments have
        // shown that y = 3/2, weight_ratio=1/4 (almost) maximizes the number of iterations for
        // calculating y^r within the set of values allowed in `swap_exact_amount_out` (see
        // `math::calc_in_given_out`). To get these values, we use the following parameters:
        // amount_out = 1/3 * balance_out, weight_out = 1, weight_in = 4.
        let asset_count = T::MaxAssets::get();
        let balance: BalanceOf<T> = LIQUIDITY.saturated_into();
        let asset_amount_out: BalanceOf<T> = balance.bmul(MAX_OUT_RATIO.saturated_into()).unwrap();
        let weight_out = T::MinWeight::get();
        let weight_in = weight_out * 4u8.into();
        let mut weights = vec![weight_out; asset_count as usize];
        weights[0] = weight_in;
        let caller: T::AccountId = whitelisted_caller();
        let (pool_id, assets, market_id) = bench_create_pool::<T>(
            caller.clone(),
            asset_count as usize,
            Some(balance),
            Some(weights),
            true,
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

    open_pool {
        let a in 2..T::MaxAssets::get().into();

        let caller: T::AccountId = whitelisted_caller();
        let (pool_id, ..) = bench_create_pool::<T>(caller, a as usize, None, None, false);
        let pool = Pallet::<T>::pool_by_id(pool_id).unwrap();
        assert_eq!(pool.status, PoolStatus::Closed);
    }: {
        Pallet::<T>::open_pool(pool_id).unwrap();
    } verify {
        let pool = Pallet::<T>::pool_by_id(pool_id).unwrap();
        assert_eq!(pool.status, PoolStatus::Open);
    }

    close_pool {
        let a in 2..T::MaxAssets::get().into();

        let caller: T::AccountId = whitelisted_caller();
        let (pool_id, ..) = bench_create_pool::<T>(caller, a as usize, None, None, true);
        let pool = Pallet::<T>::pool_by_id(pool_id).unwrap();
        assert_eq!(pool.status, PoolStatus::Open);
    }: {
        Pallet::<T>::close_pool(pool_id).unwrap();
    } verify {
        let pool = Pallet::<T>::pool_by_id(pool_id).unwrap();
        assert_eq!(pool.status, PoolStatus::Closed);
    }

    destroy_pool {
        let a in 2..T::MaxAssets::get().into();

        let caller: T::AccountId = whitelisted_caller();
        let (pool_id, ..) = bench_create_pool::<T>(caller, a as usize, None, None, true);
        assert!(Pallet::<T>::pool_by_id(pool_id).is_ok());
    }: {
        Pallet::<T>::destroy_pool(pool_id).unwrap();
    } verify {
        assert!(Pallet::<T>::pool_by_id(pool_id).is_err());
        assert_last_event::<T>(Event::PoolDestroyed::<T>{ pool_id }.into());
    }

    impl_benchmark_test_suite!(
        Swaps,
        crate::mock::ExtBuilder::default().build(),
        crate::mock::Runtime,
    );
}
