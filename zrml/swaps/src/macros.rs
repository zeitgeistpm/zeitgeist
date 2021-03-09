// Common code for `exit_swap_pool_amount_in` and `exit_swap_extern_amount_out` methods.
macro_rules! exit_swap_amount {
    (
        initial_params: ($origin:expr, $pool_id:expr, $asset_out:expr),

        asset_amount_out: $asset_amount_out:expr,
        ensure_balance: $ensure_balance:expr,
        pool_amount_in: $pool_amount_in:expr
    ) => {{
        let who = ensure_signed($origin)?;

        let pool = Self::pool_by_id($pool_id)?;

        ensure!(pool.bound($asset_out), Error::<T>::AssetNotBound);
        let pool_account = Self::pool_account_id($pool_id);

        let balance_out = T::Shares::free_balance($asset_out, &pool_account);
        ($ensure_balance(balance_out) as DispatchResult)?;

        let pool_shares_id = Self::pool_shares_id($pool_id);
        let total_supply = T::Shares::total_supply(pool_shares_id);

        let asset_amount_out = ($asset_amount_out(balance_out, &pool, total_supply) as Result<BalanceOf<T>, DispatchError>)?;
        let pool_amount_in = ($pool_amount_in(balance_out, &pool, total_supply) as Result<BalanceOf<T>, DispatchError>)?;

        let exit_fee = bmul(pool_amount_in.saturated_into(), T::ExitFee::get().saturated_into()).saturated_into();
        Self::burn_pool_shares($pool_id, &who, pool_amount_in - exit_fee)?;
        // todo do something with exit fee
        T::Shares::transfer($asset_out, &pool_account, &who, asset_amount_out)?;

        Self::deposit_event(RawEvent::Swap(GenericPoolEvent {
            pool_id: $pool_id,
            who
        }));
    }}
}

// Common code for `join_swap_extern_amount_in` and `join_swap_pool_amount_out` methods.
macro_rules! join_swap_amount {
    (
        initial_params: ($origin:expr, $pool_id:expr, $asset_in:expr),

        asset_amount_in: $asset_amount_in:expr,
        pool_amount_out: $pool_amount_out:expr
    ) => {{
        let who = ensure_signed($origin)?;

        let pool = Self::pool_by_id($pool_id)?;

        ensure!(pool.bound($asset_in), Error::<T>::AssetNotBound);

        let pool_account_id = Self::pool_account_id($pool_id);
        let balance_in = T::Shares::free_balance($asset_in, &pool_account_id);
        let pool_shares_id = Self::pool_shares_id($pool_id);
        let total_supply = T::Shares::total_supply(pool_shares_id);

        let asset_amount_in = ($asset_amount_in(balance_in, &pool, total_supply) as Result<_, DispatchError>)?;
        let pool_amount_out = ($pool_amount_out(balance_in, &pool, total_supply) as Result<_, DispatchError>)?;

        Self::mint_pool_shares($pool_id, &who, pool_amount_out)?;
        T::Shares::transfer($asset_in, &who, &pool_account_id, asset_amount_in)?;

        Self::deposit_event(RawEvent::Swap(GenericPoolEvent {
            pool_id: $pool_id,
            who
        }));
    }}
}

// Common code for `join_pool` and `exit_pool` methods.
macro_rules! pool {
    (
        initial_params: ($asset_bounds:expr, $origin:expr, $pool_amount:expr, $pool_id:expr),

        event: $event:ident,
        transfer_asset: $transfer_asset:expr,
        transfer_pool: $transfer_pool:expr
    ) => {{
        let who = ensure_signed($origin)?;

        let pool_shares_id = Self::pool_shares_id($pool_id);
        let pool_shares_total = T::Shares::total_supply(pool_shares_id);
        
        let ratio: BalanceOf<T> = bdiv($pool_amount.saturated_into(), pool_shares_total.saturated_into()).saturated_into();
        ensure!(ratio != Zero::zero(), Error::<T>::MathApproximation);
        
        let pool = Self::pool_by_id($pool_id)?;
        
        check_provided_values_len_must_equal_assets_len::<T, _>(&pool.assets, &$asset_bounds)?;
        
        let pool_account_id = Self::pool_account_id($pool_id);
        
        for (asset, amount_bound) in pool.assets.into_iter().zip($asset_bounds) {
            let balance = T::Shares::free_balance(asset, &pool_account_id);
            let amount: BalanceOf<T> = bmul(ratio.saturated_into(), balance.saturated_into()).saturated_into();
            ensure!(amount != Zero::zero(), Error::<T>::MathApproximation);
            ($transfer_asset(amount, amount_bound, asset, &pool_account_id, &who) as DispatchResult)?;
        }

        ($transfer_pool(&pool_account_id, pool_shares_id, &who) as DispatchResult)?;
        
        Self::deposit_event(RawEvent::$event(GenericPoolEvent {
            pool_id: $pool_id,
            who
        }));
    }}
}

// Common code for `swap_exact_amount_in` and `swap_exact_amount_out` methods.
macro_rules! swap_exact_amount {
    (
        initial_params: (
            $asset_in:expr,
            $asset_out:expr,
            $max_price:expr,
            $origin:expr,
            $pool_id:expr
        ),

        asset_amount_in: $asset_amount_in:expr,
        asset_amount_out: $asset_amount_out:expr
    ) => {{
        let who = ensure_signed($origin)?;

        let pool = Self::pool_by_id($pool_id)?;
        ensure!(pool.bound($asset_in), Error::<T>::AssetNotBound);
        ensure!(pool.bound($asset_out), Error::<T>::AssetNotBound);
        let spot_price_before = Self::get_spot_price($pool_id, $asset_in, $asset_out);
        ensure!(spot_price_before <= $max_price, Error::<T>::BadLimitPrice);
        
        let pool_account_id = Self::pool_account_id($pool_id);
        let asset_amount_in = ($asset_amount_in(&pool, &pool_account_id) as Result<_, DispatchError>)?;
        let asset_amount_out = ($asset_amount_out(&pool, &pool_account_id) as Result<_, DispatchError>)?;
        
        T::Shares::transfer($asset_in, &who, &pool_account_id, asset_amount_in)?;
        T::Shares::transfer($asset_out, &pool_account_id, &who, asset_amount_out)?;
        
        let spot_price_after = Self::get_spot_price($pool_id, $asset_in, $asset_out);
        ensure!(spot_price_after >= spot_price_before, Error::<T>::MathApproximation);
        ensure!(spot_price_after <= $max_price, Error::<T>::BadLimitPrice);
        ensure!(spot_price_before <= bdiv(asset_amount_in.saturated_into(), asset_amount_out.saturated_into()).saturated_into(), Error::<T>::MathApproximation);

        Self::deposit_event(RawEvent::Swap(GenericPoolEvent {
            pool_id: $pool_id,
            who
        }));
    }}
}
