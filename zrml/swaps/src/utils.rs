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

use crate::{
    events::{CommonPoolEventParams, PoolAssetEvent, PoolAssetsEvent, SwapEvent},
    AssetOf, BalanceOf, Config, Error, Pallet, PoolOf,
};
use alloc::vec::Vec;
use frame_support::{dispatch::DispatchResult, ensure};
use orml_traits::MultiCurrency;
use sp_runtime::{traits::Zero, DispatchError};
use zeitgeist_primitives::{
    math::{
        checked_ops_res::CheckedSubRes,
        fixed::{FixedDiv, FixedMul},
    },
    types::PoolId,
};

// Common code for `pool_exit_with_exact_pool_amount` and `pool_exit_with_exact_asset_amount` methods.
pub(crate) fn pool_exit_with_exact_amount<F1, F2, F3, F4, T>(
    mut p: PoolExitWithExactAmountParams<'_, F1, F2, F3, F4, T>,
) -> DispatchResult
where
    F1: FnMut(BalanceOf<T>, BalanceOf<T>) -> Result<BalanceOf<T>, DispatchError>,
    F2: FnMut(BalanceOf<T>) -> DispatchResult,
    F3: FnMut(PoolAssetEvent<T::AccountId, AssetOf<T>, BalanceOf<T>>),
    F4: FnMut(BalanceOf<T>, BalanceOf<T>) -> Result<BalanceOf<T>, DispatchError>,
    T: Config,
{
    Pallet::<T>::ensure_pool_is_active(p.pool)?;
    ensure!(p.pool.bound(&p.asset), Error::<T>::AssetNotInPool);
    let pool_account = Pallet::<T>::pool_account_id(&p.pool_id);

    let asset_balance = T::AssetManager::free_balance(p.asset, &pool_account);
    (p.ensure_balance)(asset_balance)?;

    let pool_shares_id = Pallet::<T>::pool_shares_id(p.pool_id);
    let total_issuance = T::AssetManager::total_issuance(pool_shares_id);

    let asset_amount = (p.asset_amount)(asset_balance, total_issuance)?;
    let pool_amount = (p.pool_amount)(asset_balance, total_issuance)?;

    Pallet::<T>::burn_pool_shares(p.pool_id, &p.who, pool_amount)?;
    T::AssetManager::transfer(p.asset, &pool_account, &p.who, asset_amount)?;

    (p.event)(PoolAssetEvent {
        asset: p.asset,
        bound: p.bound,
        cpep: CommonPoolEventParams { pool_id: p.pool_id, who: p.who },
        transferred: asset_amount,
        pool_amount,
    });

    Ok(())
}

// Common code for `pool_join_with_exact_asset_amount` and `pool_join_with_exact_pool_amount` methods.
pub(crate) fn pool_join_with_exact_amount<F1, F2, F3, T>(
    mut p: PoolJoinWithExactAmountParams<'_, F1, F2, F3, T>,
) -> DispatchResult
where
    F1: FnMut(BalanceOf<T>, BalanceOf<T>) -> Result<BalanceOf<T>, DispatchError>,
    F2: FnMut(PoolAssetEvent<T::AccountId, AssetOf<T>, BalanceOf<T>>),
    F3: FnMut(BalanceOf<T>, BalanceOf<T>) -> Result<BalanceOf<T>, DispatchError>,
    T: Config,
{
    Pallet::<T>::ensure_pool_is_active(p.pool)?;
    let pool_shares_id = Pallet::<T>::pool_shares_id(p.pool_id);
    let pool_account_id = Pallet::<T>::pool_account_id(&p.pool_id);
    let total_issuance = T::AssetManager::total_issuance(pool_shares_id);

    ensure!(p.pool.bound(&p.asset), Error::<T>::AssetNotInPool);
    let asset_balance = T::AssetManager::free_balance(p.asset, p.pool_account_id);

    let asset_amount = (p.asset_amount)(asset_balance, total_issuance)?;
    let pool_amount = (p.pool_amount)(asset_balance, total_issuance)?;

    Pallet::<T>::mint_pool_shares(p.pool_id, &p.who, pool_amount)?;
    T::AssetManager::transfer(p.asset, &p.who, &pool_account_id, asset_amount)?;

    (p.event)(PoolAssetEvent {
        asset: p.asset,
        bound: p.bound,
        cpep: CommonPoolEventParams { pool_id: p.pool_id, who: p.who },
        transferred: asset_amount,
        pool_amount,
    });

    Ok(())
}

// Common code for `pool_join` and `pool_exit` methods.
pub(crate) fn pool<F1, F2, F3, F4, T>(mut p: PoolParams<'_, F1, F2, F3, F4, T>) -> DispatchResult
where
    F1: FnMut(PoolAssetsEvent<T::AccountId, AssetOf<T>, BalanceOf<T>>),
    F2: FnMut(BalanceOf<T>, BalanceOf<T>, AssetOf<T>) -> DispatchResult,
    F3: FnMut() -> DispatchResult,
    F4: FnMut(BalanceOf<T>) -> Result<BalanceOf<T>, DispatchError>,
    T: Config,
{
    let pool_shares_id = Pallet::<T>::pool_shares_id(p.pool_id);
    let total_issuance = T::AssetManager::total_issuance(pool_shares_id);

    let ratio = p.pool_amount.bdiv(total_issuance)?;
    Pallet::<T>::check_provided_values_len_must_equal_assets_len(&p.pool.assets, &p.asset_bounds)?;
    ensure!(ratio != Zero::zero(), Error::<T>::MathApproximation);

    let mut transferred = Vec::with_capacity(p.asset_bounds.len());

    for (asset, amount_bound) in p.pool.assets.iter().cloned().zip(p.asset_bounds.iter().cloned()) {
        let balance = T::AssetManager::free_balance(asset, p.pool_account_id);
        // Dusting may result in zero balances in the pool; just ignore these.
        if balance.is_zero() {
            continue;
        }
        let amount = ratio.bmul(balance)?;
        let fee = (p.fee)(amount)?;
        let amount_minus_fee = amount.checked_sub_res(&fee)?;
        transferred.push(amount_minus_fee);
        ensure!(amount_minus_fee != Zero::zero(), Error::<T>::MathApproximation);
        (p.transfer_asset)(amount_minus_fee, amount_bound, asset)?;
    }

    (p.transfer_pool)()?;

    (p.event)(PoolAssetsEvent {
        assets: p.pool.assets.clone().into_inner(),
        bounds: p.asset_bounds,
        cpep: CommonPoolEventParams { pool_id: p.pool_id, who: p.who },
        transferred,
        pool_amount: p.pool_amount,
    });

    Ok(())
}

// Common code for `swap_exact_amount_in` and `swap_exact_amount_out` methods.
pub(crate) fn swap_exact_amount<F1, F2, T>(
    mut p: SwapExactAmountParams<'_, F1, F2, T>,
) -> DispatchResult
where
    F1: FnMut() -> Result<[BalanceOf<T>; 2], DispatchError>,
    F2: FnMut(SwapEvent<T::AccountId, AssetOf<T>, BalanceOf<T>>),
    T: crate::Config,
{
    Pallet::<T>::ensure_pool_is_active(p.pool)?;
    ensure!(p.pool.assets.binary_search(&p.asset_in).is_ok(), Error::<T>::AssetNotInPool);
    ensure!(p.pool.assets.binary_search(&p.asset_out).is_ok(), Error::<T>::AssetNotInPool);
    ensure!(p.pool.bound(&p.asset_in), Error::<T>::AssetNotInPool);
    ensure!(p.pool.bound(&p.asset_out), Error::<T>::AssetNotInPool);

    let spot_price_before =
        Pallet::<T>::get_spot_price(&p.pool_id, &p.asset_in, &p.asset_out, true)?;
    // Duplicate call can be optimized
    let spot_price_before_without_fees =
        Pallet::<T>::get_spot_price(&p.pool_id, &p.asset_in, &p.asset_out, false)?;

    if let Some(max_price) = p.max_price {
        ensure!(spot_price_before <= max_price, Error::<T>::BadLimitPrice);
    }

    let [asset_amount_in, asset_amount_out] = (p.asset_amounts)()?;

    T::AssetManager::transfer(p.asset_in, &p.who, p.pool_account_id, asset_amount_in)?;
    T::AssetManager::transfer(p.asset_out, p.pool_account_id, &p.who, asset_amount_out)?;

    let spot_price_after =
        Pallet::<T>::get_spot_price(&p.pool_id, &p.asset_in, &p.asset_out, true)?;

    ensure!(spot_price_after >= spot_price_before, Error::<T>::MathApproximation);

    if let Some(max_price) = p.max_price {
        ensure!(spot_price_after <= max_price, Error::<T>::BadLimitPrice);
    }

    ensure!(
        spot_price_before_without_fees <= asset_amount_in.bdiv(asset_amount_out)?,
        Error::<T>::MathApproximation
    );

    (p.event)(SwapEvent {
        asset_amount_in,
        asset_amount_out,
        asset_bound: p.asset_bound,
        asset_in: p.asset_in,
        asset_out: p.asset_out,
        cpep: CommonPoolEventParams { pool_id: p.pool_id, who: p.who },
        max_price: p.max_price,
    });

    Ok(())
}

pub(crate) struct PoolExitWithExactAmountParams<'a, F1, F2, F3, F4, T>
where
    T: Config,
{
    pub(crate) asset_amount: F1,
    pub(crate) asset: AssetOf<T>,
    pub(crate) bound: BalanceOf<T>,
    pub(crate) ensure_balance: F2,
    pub(crate) event: F3,
    pub(crate) who: T::AccountId,
    pub(crate) pool_amount: F4,
    pub(crate) pool_id: PoolId,
    pub(crate) pool: &'a PoolOf<T>,
}

pub(crate) struct PoolJoinWithExactAmountParams<'a, F1, F2, F3, T>
where
    T: Config,
{
    pub(crate) asset: AssetOf<T>,
    pub(crate) asset_amount: F1,
    pub(crate) bound: BalanceOf<T>,
    pub(crate) event: F2,
    pub(crate) who: T::AccountId,
    pub(crate) pool_account_id: &'a T::AccountId,
    pub(crate) pool_amount: F3,
    pub(crate) pool_id: PoolId,
    pub(crate) pool: &'a PoolOf<T>,
}

pub(crate) struct PoolParams<'a, F1, F2, F3, F4, T>
where
    T: Config,
{
    pub(crate) asset_bounds: Vec<BalanceOf<T>>,
    pub(crate) event: F1,
    pub(crate) pool_account_id: &'a T::AccountId,
    pub(crate) pool_amount: BalanceOf<T>,
    pub(crate) pool_id: PoolId,
    pub(crate) pool: &'a PoolOf<T>,
    pub(crate) transfer_asset: F2,
    pub(crate) transfer_pool: F3,
    pub(crate) fee: F4,
    pub(crate) who: T::AccountId,
}

pub(crate) struct SwapExactAmountParams<'a, F1, F2, T>
where
    T: Config,
{
    pub(crate) asset_amounts: F1,
    pub(crate) asset_bound: Option<BalanceOf<T>>,
    pub(crate) asset_in: AssetOf<T>,
    pub(crate) asset_out: AssetOf<T>,
    pub(crate) event: F2,
    pub(crate) max_price: Option<BalanceOf<T>>,
    pub(crate) pool_account_id: &'a T::AccountId,
    pub(crate) pool_id: PoolId,
    pub(crate) pool: &'a PoolOf<T>,
    pub(crate) who: T::AccountId,
}
