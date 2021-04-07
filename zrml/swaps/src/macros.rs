// Common code for `pool_exit_with_exact_pool_amount` and `pool_exit_with_exact_asset_amount` methods.
macro_rules! pool_exit_with_exact_amount {
    (
    initial_params:
    ($origin:expr, $pool_id:expr, $asset:expr),asset_amount:
    $asset_amount:expr,bound:
    $bound:expr,ensure_balance:
    $ensure_balance:expr,event:
    $event:ident,pool_amount:
    $pool_amount:expr
  ) => {{
        let who = ensure_signed($origin)?;

        let pool = Self::pool_by_id($pool_id)?;

        ensure!(pool.bound(&$asset), Error::<T>::AssetNotBound);
        let pool_account = Self::pool_account_id($pool_id);

        let asset_balance = T::Shares::free_balance($asset, &pool_account);
        ($ensure_balance(asset_balance) as DispatchResult)?;

        let pool_shares_id = Self::pool_shares_id($pool_id);
        let total_issuance = T::Shares::total_issuance(pool_shares_id);

        let asset_amount = ($asset_amount(&pool, asset_balance, total_issuance)
            as Result<BalanceOf<T>, DispatchError>)?;
        let pool_amount = ($pool_amount(&pool, asset_balance, total_issuance)
            as Result<BalanceOf<T>, DispatchError>)?;

        let exit_fee = bmul(
            pool_amount.saturated_into(),
            T::ExitFee::get().saturated_into(),
        )?
        .saturated_into();
        Self::burn_pool_shares($pool_id, &who, pool_amount.check_sub_rslt(&exit_fee)?)?;
        // todo do something with exit fee
        T::Shares::transfer($asset, &pool_account, &who, asset_amount)?;

        Self::deposit_event(Event::$event(PoolAssetEvent {
            bound: $bound,
            cpep: CommonPoolEventParams {
                pool_id: $pool_id,
                who,
            },
            transferred: asset_amount,
        }));

        Ok(())
    }};
}

// Common code for `pool_join_with_exact_asset_amount` and `pool_join_with_exact_pool_amount` methods.
macro_rules! pool_join_with_exact_amount {
    (
    initial_params:
    ($origin:expr, $pool_id:expr, $asset:expr),asset_amount:
    $asset_amount:expr,bound:
    $bound:expr,event:
    $event:ident,pool_amount:
    $pool_amount:expr
  ) => {{
        let who = ensure_signed($origin)?;

        let pool = Self::pool_by_id($pool_id)?;
        let pool_shares_id = Self::pool_shares_id($pool_id);
        let pool_account_id = Self::pool_account_id($pool_id);
        let total_issuance = T::Shares::total_issuance(pool_shares_id);

        ensure!(pool.bound(&$asset), Error::<T>::AssetNotBound);
        let asset_balance = T::Shares::free_balance($asset, &pool_account_id);

        let asset_amount =
            ($asset_amount(&pool, asset_balance, total_issuance) as Result<_, DispatchError>)?;
        let pool_amount =
            ($pool_amount(&pool, asset_balance, total_issuance) as Result<_, DispatchError>)?;

        Self::mint_pool_shares($pool_id, &who, pool_amount)?;
        T::Shares::transfer($asset, &who, &pool_account_id, asset_amount)?;

        Self::deposit_event(Event::$event(PoolAssetEvent {
            bound: $bound,
            cpep: CommonPoolEventParams {
                pool_id: $pool_id,
                who,
            },
            transferred: asset_amount,
        }));

        Ok(())
    }};
}

// Common code for `pool_join` and `pool_exit` methods.
macro_rules! pool {
    (
    initial_params:
    ($asset_bounds:expr, $origin:expr, $pool_amount:expr, $pool_id:expr),event:
    $event:ident,transfer_asset:
    $transfer_asset:expr,transfer_pool:
    $transfer_pool:expr
  ) => {{
        let who = ensure_signed($origin)?;

        let pool = Self::pool_by_id($pool_id)?;
        let pool_shares_id = Self::pool_shares_id($pool_id);
        let pool_account_id = Self::pool_account_id($pool_id);
        let total_issuance = T::Shares::total_issuance(pool_shares_id);

        let ratio: BalanceOf<T> = bdiv(
            $pool_amount.saturated_into(),
            total_issuance.saturated_into(),
        )?
        .saturated_into();
        Self::check_provided_values_len_must_equal_assets_len(&pool.assets, &$asset_bounds)?;
        ensure!(ratio != Zero::zero(), Error::<T>::MathApproximation);

        let mut transferred = Vec::with_capacity($asset_bounds.len());

        for (asset, amount_bound) in pool.assets.into_iter().zip($asset_bounds.iter().cloned()) {
            let balance = T::Shares::free_balance(asset, &pool_account_id);
            let amount: BalanceOf<T> =
                bmul(ratio.saturated_into(), balance.saturated_into())?.saturated_into();
            transferred.push(amount);
            ensure!(amount != Zero::zero(), Error::<T>::MathApproximation);
            ($transfer_asset(amount, amount_bound, asset, &pool_account_id, &who)
                as DispatchResult)?;
        }

        ($transfer_pool(&pool_account_id, pool_shares_id, &who) as DispatchResult)?;

        Self::deposit_event(Event::$event(PoolAssetsEvent {
            bounds: $asset_bounds,
            cpep: CommonPoolEventParams {
                pool_id: $pool_id,
                who,
            },
            transferred,
        }));

        Ok(())
    }};
}

// Common code for `swap_exact_amount_in` and `swap_exact_amount_out` methods.
macro_rules! swap_exact_amount {
    (
    initial_params:
    ($asset_in:expr, $asset_out:expr, $max_price:expr, $origin:expr, $pool_id:expr),asset_amount_in:
    $asset_amount_in:expr,asset_amount_out:
    $asset_amount_out:expr,asset_bound:
    $asset_bound:expr,event:
    $event:ident
  ) => {{
        let who = ensure_signed($origin)?;

        let pool = Self::pool_by_id($pool_id)?;
        ensure!(pool.bound(&$asset_in), Error::<T>::AssetNotBound);
        ensure!(pool.bound(&$asset_out), Error::<T>::AssetNotBound);
        let spot_price_before = Self::get_spot_price($pool_id, $asset_in, $asset_out)?;
        ensure!(spot_price_before <= $max_price, Error::<T>::BadLimitPrice);

        let pool_account_id = Self::pool_account_id($pool_id);
        let asset_amount_in =
            ($asset_amount_in(&pool, &pool_account_id) as Result<_, DispatchError>)?;
        let asset_amount_out =
            ($asset_amount_out(&pool, &pool_account_id) as Result<_, DispatchError>)?;

        T::Shares::transfer($asset_in, &who, &pool_account_id, asset_amount_in)?;

        T::Shares::transfer($asset_out, &pool_account_id, &who, asset_amount_out)?;

        let spot_price_after = Self::get_spot_price($pool_id, $asset_in, $asset_out)?;
        ensure!(
            spot_price_after >= spot_price_before,
            Error::<T>::MathApproximation
        );
        ensure!(spot_price_after <= $max_price, Error::<T>::BadLimitPrice);
        ensure!(
            spot_price_before
                <= bdiv(
                    asset_amount_in.saturated_into(),
                    asset_amount_out.saturated_into()
                )?
                .saturated_into(),
            Error::<T>::MathApproximation
        );

        Self::deposit_event(Event::$event(SwapEvent {
            asset_amount_in: asset_amount_in,
            asset_amount_out: asset_amount_out,
            asset_bound: $asset_bound,
            cpep: CommonPoolEventParams {
                pool_id: $pool_id,
                who,
            },
            max_price: $max_price,
        }));

        Ok(())
    }};
}
