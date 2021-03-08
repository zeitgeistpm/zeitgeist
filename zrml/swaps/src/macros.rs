// Common code for `join_pool` and `exit_pool` methods.
macro_rules! pool_in_and_out {
    (
        params: ($asset_bounds:expr, $origin:expr, $pool_amount:expr, $pool_id:expr),

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
        
        Self::deposit_event(RawEvent::JoinedPool(GenericPoolEvent {
            pool_id: $pool_id,
            who
        }));
    }}
}
