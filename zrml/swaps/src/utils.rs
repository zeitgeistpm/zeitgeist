use crate::{
    check_arithm_rslt::CheckArithmRslt,
    events::{CommonPoolEventParams, PoolAssetEvent, PoolAssetsEvent, SwapEvent},
    fixed::{bdiv, bmul},
    BalanceOf, Config, Error, Pallet,
};
use alloc::vec::Vec;
use frame_support::{dispatch::DispatchResult, ensure, traits::Get};
use frame_system::ensure_signed;
use orml_traits::MultiCurrency;
use sp_runtime::{traits::Zero, DispatchError, SaturatedConversion};
use zeitgeist_primitives::types::{Asset, Pool, PoolId};

// Common code for `pool_exit_with_exact_pool_amount` and `pool_exit_with_exact_asset_amount` methods.
pub(crate) fn pool_exit_with_exact_amount<F1, F2, F3, F4, T>(
    mut p: PoolExitWithExactAmountParams<'_, F1, F2, F3, F4, T>,
) -> DispatchResult
where
    F1: FnMut(BalanceOf<T>, BalanceOf<T>) -> Result<BalanceOf<T>, DispatchError>,
    F2: FnMut(BalanceOf<T>) -> DispatchResult,
    F3: FnMut(PoolAssetEvent<T::AccountId, BalanceOf<T>>),
    F4: FnMut(BalanceOf<T>, BalanceOf<T>) -> Result<BalanceOf<T>, DispatchError>,
    T: Config,
{
    let who = ensure_signed(p.origin)?;

    ensure!(p.pool.bound(&p.asset), Error::<T>::AssetNotBound);
    let pool_account = Pallet::<T>::pool_account_id(p.pool_id);

    let asset_balance = T::Shares::free_balance(p.asset, &pool_account);
    (p.ensure_balance)(asset_balance)?;

    let pool_shares_id = Pallet::<T>::pool_shares_id(p.pool_id);
    let total_issuance = T::Shares::total_issuance(pool_shares_id);

    let asset_amount = (p.asset_amount)(asset_balance, total_issuance)?;
    let pool_amount = (p.pool_amount)(asset_balance, total_issuance)?;

    let exit_fee =
        bmul(pool_amount.saturated_into(), T::ExitFee::get().saturated_into())?.saturated_into();
    Pallet::<T>::burn_pool_shares(p.pool_id, &who, pool_amount.check_sub_rslt(&exit_fee)?)?;
    // todo do something with exit fee
    T::Shares::transfer(p.asset, &pool_account, &who, asset_amount)?;

    (p.event)(PoolAssetEvent {
        bound: p.bound,
        cpep: CommonPoolEventParams { pool_id: p.pool_id, who },
        transferred: asset_amount,
    });

    Ok(())
}

// Common code for `pool_join_with_exact_asset_amount` and `pool_join_with_exact_pool_amount` methods.
pub(crate) fn pool_join_with_exact_amount<F1, F2, F3, T>(
    mut p: PoolJoinWithExactAmountParams<'_, F1, F2, F3, T>,
) -> DispatchResult
where
    F1: FnMut(BalanceOf<T>, BalanceOf<T>) -> Result<BalanceOf<T>, DispatchError>,
    F2: FnMut(PoolAssetEvent<T::AccountId, BalanceOf<T>>),
    F3: FnMut(BalanceOf<T>, BalanceOf<T>) -> Result<BalanceOf<T>, DispatchError>,
    T: Config,
{
    let who = ensure_signed(p.origin)?;

    Pallet::<T>::check_if_pool_is_active(p.pool)?;
    let pool_shares_id = Pallet::<T>::pool_shares_id(p.pool_id);
    let pool_account_id = Pallet::<T>::pool_account_id(p.pool_id);
    let total_issuance = T::Shares::total_issuance(pool_shares_id);

    ensure!(p.pool.bound(&p.asset), Error::<T>::AssetNotBound);
    let asset_balance = T::Shares::free_balance(p.asset, &p.pool_account_id);

    let asset_amount = (p.asset_amount)(asset_balance, total_issuance)?;
    let pool_amount = (p.pool_amount)(asset_balance, total_issuance)?;

    Pallet::<T>::mint_pool_shares(p.pool_id, &who, pool_amount)?;
    T::Shares::transfer(p.asset, &who, &pool_account_id, asset_amount)?;

    (p.event)(PoolAssetEvent {
        bound: p.bound,
        cpep: CommonPoolEventParams { pool_id: p.pool_id, who },
        transferred: asset_amount,
    });

    Ok(())
}

// Common code for `pool_join` and `pool_exit` methods.
pub(crate) fn pool<F1, F2, F3, T>(mut p: PoolParams<'_, F1, F2, F3, T>) -> DispatchResult
where
    F1: FnMut(PoolAssetsEvent<T::AccountId, BalanceOf<T>>),
    F2: FnMut(BalanceOf<T>, BalanceOf<T>, Asset<T::MarketId>) -> DispatchResult,
    F3: FnMut(Asset<T::MarketId>) -> DispatchResult,
    T: Config,
{
    let pool_shares_id = Pallet::<T>::pool_shares_id(p.pool_id);
    let total_issuance = T::Shares::total_issuance(pool_shares_id);

    let ratio: BalanceOf<T> =
        bdiv(p.pool_amount.saturated_into(), total_issuance.saturated_into())?.saturated_into();
    Pallet::<T>::check_provided_values_len_must_equal_assets_len(&p.pool.assets, &p.asset_bounds)?;
    ensure!(ratio != Zero::zero(), Error::<T>::MathApproximation);

    let mut transferred = Vec::with_capacity(p.asset_bounds.len());

    for (asset, amount_bound) in p.pool.assets.iter().cloned().zip(p.asset_bounds.iter().cloned()) {
        let balance = T::Shares::free_balance(asset, p.pool_account_id);
        let amount = bmul(ratio.saturated_into(), balance.saturated_into())?.saturated_into();
        transferred.push(amount);
        ensure!(amount != Zero::zero(), Error::<T>::MathApproximation);
        (p.transfer_asset)(amount, amount_bound, asset)?;
    }

    (p.transfer_pool)(pool_shares_id)?;

    (p.event)(PoolAssetsEvent {
        bounds: p.asset_bounds,
        cpep: CommonPoolEventParams { pool_id: p.pool_id, who: p.who },
        transferred,
    });

    Ok(())
}

// Common code for `swap_exact_amount_in` and `swap_exact_amount_out` methods.
pub(crate) fn swap_exact_amount<F1, F2, T>(
    mut p: SwapExactAmountParams<'_, F1, F2, T>,
) -> DispatchResult
where
    F1: FnMut() -> Result<[BalanceOf<T>; 2], DispatchError>,
    F2: FnMut(SwapEvent<T::AccountId, BalanceOf<T>>),
    T: Config,
{
    let who = ensure_signed(p.origin)?;

    Pallet::<T>::check_if_pool_is_active(p.pool)?;
    ensure!(p.pool.bound(&p.asset_in), Error::<T>::AssetNotBound);
    ensure!(p.pool.bound(&p.asset_out), Error::<T>::AssetNotBound);
    let spot_price_before = Pallet::<T>::get_spot_price(p.pool_id, p.asset_in, p.asset_out)?;
    ensure!(spot_price_before <= p.max_price, Error::<T>::BadLimitPrice);

    let [asset_amount_in, asset_amount_out] = (p.asset_amounts)()?;

    T::Shares::transfer(p.asset_in, &who, p.pool_account_id, asset_amount_in)?;
    T::Shares::transfer(p.asset_out, p.pool_account_id, &who, asset_amount_out)?;

    let spot_price_after = Pallet::<T>::get_spot_price(p.pool_id, p.asset_in, p.asset_out)?;
    ensure!(spot_price_after >= spot_price_before, Error::<T>::MathApproximation);
    ensure!(spot_price_after <= p.max_price, Error::<T>::BadLimitPrice);
    ensure!(
        spot_price_before
            <= bdiv(asset_amount_in.saturated_into(), asset_amount_out.saturated_into())?
                .saturated_into(),
        Error::<T>::MathApproximation
    );

    (p.event)(SwapEvent {
        asset_amount_in,
        asset_amount_out,
        asset_bound: p.asset_bound,
        cpep: CommonPoolEventParams { pool_id: p.pool_id, who },
        max_price: p.max_price,
    });

    Ok(())
}

pub(crate) struct PoolExitWithExactAmountParams<'a, F1, F2, F3, F4, T>
where
    T: Config,
{
    pub(crate) asset_amount: F1,
    pub(crate) asset: Asset<T::MarketId>,
    pub(crate) bound: BalanceOf<T>,
    pub(crate) ensure_balance: F2,
    pub(crate) event: F3,
    pub(crate) origin: T::Origin,
    pub(crate) pool_amount: F4,
    pub(crate) pool_id: PoolId,
    pub(crate) pool: &'a Pool<BalanceOf<T>, T::MarketId>,
}

pub(crate) struct PoolJoinWithExactAmountParams<'a, F1, F2, F3, T>
where
    T: Config,
{
    pub(crate) asset: Asset<T::MarketId>,
    pub(crate) asset_amount: F1,
    pub(crate) bound: BalanceOf<T>,
    pub(crate) event: F2,
    pub(crate) origin: T::Origin,
    pub(crate) pool_account_id: &'a T::AccountId,
    pub(crate) pool_amount: F3,
    pub(crate) pool_id: PoolId,
    pub(crate) pool: &'a Pool<BalanceOf<T>, T::MarketId>,
}

pub(crate) struct PoolParams<'a, F1, F2, F3, T>
where
    T: Config,
{
    pub(crate) asset_bounds: Vec<BalanceOf<T>>,
    pub(crate) event: F1,
    pub(crate) pool_account_id: &'a T::AccountId,
    pub(crate) pool_amount: BalanceOf<T>,
    pub(crate) pool_id: PoolId,
    pub(crate) pool: &'a Pool<BalanceOf<T>, T::MarketId>,
    pub(crate) transfer_asset: F2,
    pub(crate) transfer_pool: F3,
    pub(crate) who: T::AccountId,
}

pub(crate) struct SwapExactAmountParams<'a, F1, F2, T>
where
    T: Config,
{
    pub(crate) asset_amounts: F1,
    pub(crate) asset_bound: BalanceOf<T>,
    pub(crate) asset_in: Asset<T::MarketId>,
    pub(crate) asset_out: Asset<T::MarketId>,
    pub(crate) event: F2,
    pub(crate) max_price: BalanceOf<T>,
    pub(crate) origin: T::Origin,
    pub(crate) pool_account_id: &'a T::AccountId,
    pub(crate) pool_id: PoolId,
    pub(crate) pool: &'a Pool<BalanceOf<T>, T::MarketId>,
}
