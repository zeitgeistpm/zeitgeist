// Copyright 2022-2023 Forecasting Technologies LTD.
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
use crate::{Config, Event, MarketIdOf};
use frame_benchmarking::{benchmarks, vec, whitelisted_caller, Vec};
use frame_support::{dispatch::DispatchResult, traits::Get};
use frame_system::RawOrigin;
use orml_traits::MultiCurrency;
use sp_runtime::{
    traits::{CheckedSub, SaturatedConversion, Zero},
    Perbill,
};
use zeitgeist_primitives::{
    constants::{BASE, CENT},
    math::fixed::FixedMul,
    traits::{MarketCommonsPalletApi, Swaps as _},
    types::{
        Asset, Deadlines, Market, MarketBonds, MarketCreation, MarketDisputeMechanism,
        MarketPeriod, MarketStatus, MarketType, PoolId, PoolStatus, ScoringRule,
    },
};

const DEFAULT_CREATOR_FEE: Perbill = Perbill::from_perthousand(1);
const LIQUIDITY: u128 = 100 * BASE;

type MarketOf<T> = zeitgeist_primitives::types::Market<
    <T as frame_system::Config>::AccountId,
    BalanceOf<T>,
    <<T as Config>::MarketCommons as MarketCommonsPalletApi>::BlockNumber,
    <<T as Config>::MarketCommons as MarketCommonsPalletApi>::Moment,
    Asset<<<T as Config>::MarketCommons as MarketCommonsPalletApi>::MarketId>,
>;

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
    frame_system::Pallet::<T>::assert_last_event(generic_event.into());
}

fn set_default_creator_fee<T: Config>(market_id: MarketIdOf<T>) -> DispatchResult {
    T::MarketCommons::mutate_market(&market_id, |market: &mut MarketOf<T>| {
        market.creator_fee = DEFAULT_CREATOR_FEE;
        Ok(())
    })
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
            _ => LIQUIDITY.saturated_into(),
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

fn push_default_market<T: Config>(caller: T::AccountId, oracle: T::AccountId) -> MarketIdOf<T> {
    let market = Market {
        base_asset: Asset::Ztg,
        creation: MarketCreation::Permissionless,
        creator_fee: Perbill::zero(),
        creator: caller,
        market_type: MarketType::Categorical(3),
        dispute_mechanism: Some(MarketDisputeMechanism::Authorized),
        metadata: vec![0; 50],
        oracle,
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
        bonds: MarketBonds::default(),
        early_close: None,
    };

    T::MarketCommons::push_market(market).unwrap()
}

fn initialize_pool<T: Config>(
    caller: &T::AccountId,
    asset_count: Option<usize>,
    asset_amount: Option<BalanceOf<T>>,
    weights: Option<Vec<u128>>,
) -> (PoolId, Vec<Asset<MarketIdOf<T>>>, MarketIdOf<T>) {
    let asset_count_unwrapped: usize = {
        match asset_count {
            Some(ac) => ac,
            _ => T::MaxAssets::get().into(),
        }
    };

    let assets = generate_assets::<T>(caller, asset_count_unwrapped, asset_amount);
    let some_weights = if weights.is_some() {
        weights
    } else {
        Some(vec![T::MinWeight::get(); asset_count_unwrapped])
    };
    let base_asset = *assets.last().unwrap();
    let market_id = push_default_market::<T>(caller.clone(), caller.clone());

    let pool_id = Pallet::<T>::create_pool(
        caller.clone(),
        assets.clone(),
        base_asset,
        market_id,
        Some(Zero::zero()),
        Some(LIQUIDITY.saturated_into()),
        some_weights,
    )
    .unwrap();

    (pool_id, assets, market_id)
}

// Creates a pool containing `asset_count` (default: max assets) assets.
// Returns `PoolId`, `Vec<Asset<...>>`, ``MarketId`
fn bench_create_pool<T: Config>(
    caller: T::AccountId,
    asset_count: Option<usize>,
    asset_amount: Option<BalanceOf<T>>,
    weights: Option<Vec<u128>>,
) -> (u128, Vec<Asset<MarketIdOf<T>>>, MarketIdOf<T>) {
    let (pool_id, assets, market_id) =
        initialize_pool::<T>(&caller, asset_count, asset_amount, weights);
    let _ = Pallet::<T>::open_pool(pool_id);
    (pool_id, assets, market_id)
}

benchmarks! {
    pool_exit {
        let a in 2 .. T::MaxAssets::get().into();
        let caller: T::AccountId = whitelisted_caller();
        let (pool_id, ..) = bench_create_pool::<T>(
            caller.clone(),
            Some(a as usize),
            Some((2u128 * LIQUIDITY).saturated_into()),
            None,
        );
        let pool_amount = (LIQUIDITY / 2u128).saturated_into();
        let min_assets_out = vec![0u32.into(); a as usize];
    }: _(RawOrigin::Signed(caller), pool_id, pool_amount, min_assets_out)

    pool_exit_with_exact_asset_amount {
        let a = T::MaxAssets::get();
        let caller: T::AccountId = whitelisted_caller();
        let (pool_id, assets, ..) = bench_create_pool::<T>(
            caller.clone(),
            Some(a as usize),
            None,
            None,
        );
        let asset_amount: BalanceOf<T> = BASE.saturated_into();
        let pool_amount = LIQUIDITY.saturated_into();
    }: _(RawOrigin::Signed(caller), pool_id, assets[0], asset_amount, pool_amount)

    pool_exit_with_exact_pool_amount {
        let a = T::MaxAssets::get();
        let caller: T::AccountId = whitelisted_caller();
        let (pool_id, assets, ..) = bench_create_pool::<T>(
            caller.clone(),
            Some(a as usize),
            None,
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
            Some((2u128 * LIQUIDITY).saturated_into()),
            None,
        );
        let pool_amount = LIQUIDITY.saturated_into();
        let max_assets_in = vec![LIQUIDITY.saturated_into(); a as usize];
    }: _(RawOrigin::Signed(caller), pool_id, pool_amount, max_assets_in)

    pool_join_with_exact_asset_amount {
        let a = T::MaxAssets::get();
        let caller: T::AccountId = whitelisted_caller();
        let (pool_id, assets, ..) = bench_create_pool::<T>(
            caller.clone(),
            Some(a as usize),
            Some((2u128 * LIQUIDITY).saturated_into()),
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
            Some((2u128 * LIQUIDITY).saturated_into()),
            None,
        );
        let pool_amount = BASE.saturated_into();
        let max_asset_amount: BalanceOf<T> = LIQUIDITY.saturated_into();
    }: _(RawOrigin::Signed(caller), pool_id, assets[0], pool_amount, max_asset_amount)

    swap_exact_amount_in_cpmm {
        // We're trying to get as many iterations in `bpow_approx` as possible. Experiments have
        // shown that y = 3/4, weight_ratio=1/2 (almost) maximizes the number of iterations for
        // calculating y^r within the set of values allowed in `swap_exact_amount_in` (see
        // `math::calc_out_given_in`). To get these values, we use the following parameters:
        // amount_in = 1/3 * balance_in, weight_in = 1, weight_out = 2.
        let asset_count = T::MaxAssets::get();
        let balance: BalanceOf<T> = LIQUIDITY.saturated_into();
        let asset_amount_in: BalanceOf<T> = balance.bmul(T::MaxInRatio::get()).unwrap();
        let weight_in = T::MinWeight::get();
        let weight_out = 2 * weight_in;
        let mut weights = vec![weight_in; asset_count as usize];
        weights[asset_count as usize - 1] = weight_out;
        let caller: T::AccountId = whitelisted_caller();
        let (pool_id, assets, market_id) = bench_create_pool::<T>(
            caller.clone(),
            Some(asset_count as usize),
            Some(balance),
            Some(weights),
        );
        set_default_creator_fee::<T>(market_id)?;
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

    swap_exact_amount_out_cpmm {
        // We're trying to get as many iterations in `bpow_approx` as possible. Experiments have
        // shown that y = 3/2, weight_ratio=1/4 (almost) maximizes the number of iterations for
        // calculating y^r within the set of values allowed in `swap_exact_amount_out` (see
        // `math::calc_in_given_out`). To get these values, we use the following parameters:
        // amount_out = 1/3 * balance_out, weight_out = 1, weight_in = 4.
        let asset_count = T::MaxAssets::get();
        let balance: BalanceOf<T> = LIQUIDITY.saturated_into();
        let mut asset_amount_out: BalanceOf<T> = balance.bmul(T::MaxOutRatio::get()).unwrap();
        asset_amount_out = Perbill::one()
            .checked_sub(&DEFAULT_CREATOR_FEE)
            .unwrap()
            .mul_floor(asset_amount_out);
        let weight_out = T::MinWeight::get();
        let weight_in = 4 * weight_out;
        let mut weights = vec![weight_out; asset_count as usize];
        weights[0] = weight_in;
        let caller: T::AccountId = whitelisted_caller();
        let (pool_id, assets, market_id) = bench_create_pool::<T>(
            caller.clone(),
            Some(asset_count as usize),
            Some(balance),
            Some(weights),
        );
        set_default_creator_fee::<T>(market_id)?;
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
        let (pool_id, ..) = initialize_pool::<T>(
            &caller,
            Some(a as usize),
            None,
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

    impl_benchmark_test_suite!(
        Swaps,
        crate::mock::ExtBuilder::default().build(),
        crate::mock::Runtime,
    );
}
