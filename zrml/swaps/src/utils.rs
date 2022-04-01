use crate::{
    check_arithm_rslt::CheckArithmRslt,
    events::{CommonPoolEventParams, PoolAssetEvent, PoolAssetsEvent, SwapEvent},
    fixed::{bdiv, bmul},
    BalanceOf, Config, Error, Pallet,
};
use alloc::vec::Vec;
use frame_support::{dispatch::DispatchResult, ensure, traits::Get};
use orml_traits::MultiCurrency;
use sp_runtime::{
    traits::{Saturating, Zero},
    DispatchError, SaturatedConversion,
};
use zeitgeist_primitives::types::{Asset, Pool, PoolId, ScoringRule};
use zrml_rikiddo::traits::RikiddoMVPallet;

// Common code for `pool_exit_with_exact_pool_amount` and `pool_exit_with_exact_asset_amount` methods.
pub(crate) fn pool_exit_with_exact_amount<F1, F2, F3, F4, T>(
    mut p: PoolExitWithExactAmountParams<'_, F1, F2, F3, F4, T>,
) -> DispatchResult
where
    F1: FnMut(BalanceOf<T>, BalanceOf<T>) -> Result<BalanceOf<T>, DispatchError>,
    F2: FnMut(BalanceOf<T>) -> DispatchResult,
    F3: FnMut(PoolAssetEvent<T::AccountId, Asset<T::MarketId>, BalanceOf<T>>),
    F4: FnMut(BalanceOf<T>, BalanceOf<T>) -> Result<BalanceOf<T>, DispatchError>,
    T: Config,
{
    ensure!(p.pool.scoring_rule == ScoringRule::CPMM, Error::<T>::InvalidScoringRule);
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
    Pallet::<T>::burn_pool_shares(p.pool_id, &p.who, pool_amount.check_sub_rslt(&exit_fee)?)?;
    // todo do something with exit fee
    T::Shares::transfer(p.asset, &pool_account, &p.who, asset_amount)?;

    (p.event)(PoolAssetEvent {
        asset: p.asset,
        bound: p.bound,
        cpep: CommonPoolEventParams { pool_id: p.pool_id, who: p.who },
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
    F2: FnMut(PoolAssetEvent<T::AccountId, Asset<T::MarketId>, BalanceOf<T>>),
    F3: FnMut(BalanceOf<T>, BalanceOf<T>) -> Result<BalanceOf<T>, DispatchError>,
    T: Config,
{
    ensure!(p.pool.scoring_rule == ScoringRule::CPMM, Error::<T>::InvalidScoringRule);
    Pallet::<T>::check_if_pool_is_active(p.pool)?;
    let pool_shares_id = Pallet::<T>::pool_shares_id(p.pool_id);
    let pool_account_id = Pallet::<T>::pool_account_id(p.pool_id);
    let total_issuance = T::Shares::total_issuance(pool_shares_id);

    ensure!(p.pool.bound(&p.asset), Error::<T>::AssetNotBound);
    let asset_balance = T::Shares::free_balance(p.asset, p.pool_account_id);

    let asset_amount = (p.asset_amount)(asset_balance, total_issuance)?;
    let pool_amount = (p.pool_amount)(asset_balance, total_issuance)?;

    Pallet::<T>::mint_pool_shares(p.pool_id, &p.who, pool_amount)?;
    T::Shares::transfer(p.asset, &p.who, &pool_account_id, asset_amount)?;

    (p.event)(PoolAssetEvent {
        asset: p.asset,
        bound: p.bound,
        cpep: CommonPoolEventParams { pool_id: p.pool_id, who: p.who },
        transferred: asset_amount,
    });

    Ok(())
}

// Common code for `pool_join` and `pool_exit` methods.
pub(crate) fn pool<F1, F2, F3, F4, T>(mut p: PoolParams<'_, F1, F2, F3, F4, T>) -> DispatchResult
where
    F1: FnMut(PoolAssetsEvent<T::AccountId, Asset<T::MarketId>, BalanceOf<T>>),
    F2: FnMut(BalanceOf<T>, BalanceOf<T>, Asset<T::MarketId>) -> DispatchResult,
    F3: FnMut() -> DispatchResult,
    F4: FnMut(BalanceOf<T>) -> Result<BalanceOf<T>, DispatchError>,
    T: Config,
{
    ensure!(p.pool.scoring_rule == ScoringRule::CPMM, Error::<T>::InvalidScoringRule);
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
        let fee = (p.fee)(amount)?;
        let amount_minus_fee = amount.check_sub_rslt(&fee)?;
        transferred.push(amount_minus_fee);
        ensure!(amount_minus_fee != Zero::zero(), Error::<T>::MathApproximation);
        (p.transfer_asset)(amount_minus_fee, amount_bound, asset)?;
    }

    (p.transfer_pool)()?;

    (p.event)(PoolAssetsEvent {
        assets: p.pool.assets.clone(),
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
    F2: FnMut(SwapEvent<T::AccountId, Asset<T::MarketId>, BalanceOf<T>>),
    T: crate::Config,
{
    Pallet::<T>::check_if_pool_is_active(p.pool)?;
    ensure!(p.pool.assets.binary_search(&p.asset_in).is_ok(), Error::<T>::AssetNotInPool);
    ensure!(p.pool.assets.binary_search(&p.asset_out).is_ok(), Error::<T>::AssetNotInPool);

    if p.pool.scoring_rule == ScoringRule::CPMM {
        ensure!(p.pool.bound(&p.asset_in), Error::<T>::AssetNotBound);
        ensure!(p.pool.bound(&p.asset_out), Error::<T>::AssetNotBound);
    }

    let spot_price_before = Pallet::<T>::get_spot_price(p.pool_id, p.asset_in, p.asset_out)?;
    ensure!(spot_price_before <= p.max_price, Error::<T>::BadLimitPrice);

    let [asset_amount_in, asset_amount_out] = (p.asset_amounts)()?;

    if p.pool.scoring_rule == ScoringRule::CPMM {
        T::Shares::transfer(p.asset_in, &p.who, p.pool_account_id, asset_amount_in)?;
        T::Shares::transfer(p.asset_out, p.pool_account_id, &p.who, asset_amount_out)?;
    } else {
        let base_asset = p.pool.base_asset.ok_or(Error::<T>::BaseAssetNotFound)?;

        if p.asset_in == base_asset {
            T::Shares::transfer(p.asset_in, &p.who, p.pool_account_id, asset_amount_in)?;
            T::Shares::deposit(p.asset_out, &p.who, asset_amount_out)?;
        } else if p.asset_out == base_asset {
            // We can use the lightweight withdraw here, since event assets are not reserved.
            T::Shares::withdraw(p.asset_in, &p.who, asset_amount_in)?;
            T::Shares::transfer(p.asset_out, p.pool_account_id, &p.who, asset_amount_out)?;
        } else {
            // Just for safety, should already be checked in p.asset_amounts.
            return Err(Error::<T>::UnsupportedTrade.into());
        }
    }

    let spot_price_after = Pallet::<T>::get_spot_price(p.pool_id, p.asset_in, p.asset_out)?;

    // Allow little tolerance
    if p.pool.scoring_rule == ScoringRule::RikiddoSigmoidFeeMarketEma {
        ensure!(
            spot_price_before.saturating_sub(spot_price_after) < 20u8.into(),
            Error::<T>::MathApproximation
        );
    } else {
        ensure!(spot_price_after >= spot_price_before, Error::<T>::MathApproximation);
    }

    ensure!(spot_price_after <= p.max_price, Error::<T>::BadLimitPrice);

    if p.pool.scoring_rule == ScoringRule::CPMM {
        ensure!(
            spot_price_before
                <= bdiv(asset_amount_in.saturated_into(), asset_amount_out.saturated_into())?
                    .saturated_into(),
            Error::<T>::MathApproximation
        );
    }

    if p.pool.scoring_rule == ScoringRule::RikiddoSigmoidFeeMarketEma {
        // Currently the only allowed trades are base_currency <-> event asset. We count the
        // volume in base_currency.
        let base_asset = p.pool.base_asset.ok_or(Error::<T>::BaseAssetNotFound)?;
        let volume = if p.asset_in == base_asset { asset_amount_in } else { asset_amount_out };
        T::RikiddoSigmoidFeeMarketEma::update_volume(p.pool_id, volume)?;
    }

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
    pub(crate) asset: Asset<T::MarketId>,
    pub(crate) bound: BalanceOf<T>,
    pub(crate) ensure_balance: F2,
    pub(crate) event: F3,
    pub(crate) who: T::AccountId,
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
    pub(crate) who: T::AccountId,
    pub(crate) pool_account_id: &'a T::AccountId,
    pub(crate) pool_amount: F3,
    pub(crate) pool_id: PoolId,
    pub(crate) pool: &'a Pool<BalanceOf<T>, T::MarketId>,
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
    pub(crate) pool: &'a Pool<BalanceOf<T>, T::MarketId>,
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
    pub(crate) asset_bound: BalanceOf<T>,
    pub(crate) asset_in: Asset<T::MarketId>,
    pub(crate) asset_out: Asset<T::MarketId>,
    pub(crate) event: F2,
    pub(crate) max_price: BalanceOf<T>,
    pub(crate) pool_account_id: &'a T::AccountId,
    pub(crate) pool_id: PoolId,
    pub(crate) pool: &'a Pool<BalanceOf<T>, T::MarketId>,
    pub(crate) who: T::AccountId,
}
