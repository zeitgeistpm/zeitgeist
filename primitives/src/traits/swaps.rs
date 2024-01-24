// Copyright 2023-2024 Forecasting Technologies LTD.
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

use crate::types::PoolId;
use alloc::vec::Vec;
use frame_support::dispatch::{DispatchError, Weight};

pub trait Swaps<AccountId> {
    type Asset;
    type Balance;

    /// Creates an initial active pool.
    ///
    /// # Arguments
    ///
    /// * `who`: The account that is the creator of the pool. Must have enough
    /// funds for each of the assets to cover the minimum balance.
    /// * `assets`: The assets that are used in the pool.
    /// * `base_asset`: The base asset in a prediction market swap pool (usually a currency).
    /// * `market_id`: The market id of the market the pool belongs to.
    /// * `scoring_rule`: The scoring rule that's used to determine the asset prices.
    /// * `swap_fee`: The fee applied to each swap (in case the scoring rule doesn't provide fees).
    /// * `amount`: The amount of each asset added to the pool; **may** be `None` only if
    ///   `scoring_rule` is `RikiddoSigmoidFeeMarketEma`.
    /// * `weights`: These are the denormalized weights (the raw weights).
    fn create_pool(
        creator: AccountId,
        assets: Vec<Self::Asset>,
        swap_fee: Self::Balance,
        amount: Self::Balance,
        weights: Vec<Self::Balance>,
    ) -> Result<PoolId, DispatchError>;

    /// Close the specified pool.
    fn close_pool(pool_id: PoolId) -> Result<Weight, DispatchError>;

    /// Destroy CPMM pool, slash pool account assets and destroy pool shares of the liquidity providers.
    fn destroy_pool(pool_id: PoolId) -> Result<Weight, DispatchError>;

    fn open_pool(pool_id: PoolId) -> Result<Weight, DispatchError>;

    /// Pool - Exit with exact pool amount
    ///
    /// Takes an asset from `pool_id` and transfers to `origin`. Differently from `pool_exit`,
    /// this method injects the exactly amount of `asset_amount` to `origin`.
    ///
    /// # Arguments
    ///
    /// * `who`: Liquidity Provider (LP). The account whose assets should be received.
    /// * `pool_id`: Unique pool identifier.
    /// * `asset`: Self::Asset leaving the pool.
    /// * `asset_amount`: Self::Asset amount that is leaving the pool.
    /// * `max_pool_amount`: The calculated amount of assets for the pool must be equal or
    /// greater than the given value.
    fn pool_exit_with_exact_asset_amount(
        who: AccountId,
        pool_id: PoolId,
        asset: Self::Asset,
        asset_amount: Self::Balance,
        max_pool_amount: Self::Balance,
    ) -> Result<Weight, DispatchError>;

    /// Pool - Join with exact asset amount
    ///
    /// Joins an asset provided from `origin` to `pool_id`. Differently from `pool_join`,
    /// this method transfers the exactly amount of `asset_amount` to `pool_id`.
    ///
    /// # Arguments
    ///
    /// * `who`: Liquidity Provider (LP). The account whose assets should be received.
    /// * `pool_id`: Unique pool identifier.
    /// * `asset_in`: Self::Asset entering the pool.
    /// * `asset_amount`: Self::Asset amount that is entering the pool.
    /// * `min_pool_amount`: The calculated amount for the pool must be equal or greater
    /// than the given value.
    fn pool_join_with_exact_asset_amount(
        who: AccountId,
        pool_id: PoolId,
        asset_in: Self::Asset,
        asset_amount: Self::Balance,
        min_pool_amount: Self::Balance,
    ) -> Result<Weight, DispatchError>;

    /// Swap - Exact amount in
    ///
    /// Swaps a given `asset_amount_in` of the `asset_in/asset_out` pair to `pool_id`.
    ///
    /// # Arguments
    ///
    /// * `who`: The account whose assets should be transferred.
    /// * `pool_id`: Unique pool identifier.
    /// * `asset_in`: Self::Asset entering the pool.
    /// * `asset_amount_in`: Amount that will be transferred from the provider to the pool.
    /// * `asset_out`: Self::Asset leaving the pool.
    /// * `min_asset_amount_out`: Minimum asset amount that can leave the pool.
    /// * `max_price`: Market price must be equal or less than the provided value.
    /// * `handle_fees`: Whether additional fees are handled or not (sets LP fee to 0)
    #[allow(clippy::too_many_arguments)]
    fn swap_exact_amount_in(
        who: AccountId,
        pool_id: PoolId,
        asset_in: Self::Asset,
        asset_amount_in: Self::Balance,
        asset_out: Self::Asset,
        min_asset_amount_out: Option<Self::Balance>,
        max_price: Option<Self::Balance>,
    ) -> Result<Weight, DispatchError>;

    /// Swap - Exact amount out
    ///
    /// Swaps a given `asset_amount_out` of the `asset_in/asset_out` pair to `origin`.
    ///
    /// # Arguments
    ///
    /// * `who`: The account whose assets should be transferred.
    /// * `pool_id`: Unique pool identifier.
    /// * `asset_in`: Self::Asset entering the pool.
    /// * `max_amount_asset_in`: Maximum asset amount that can enter the pool.
    /// * `asset_out`: Self::Asset leaving the pool.
    /// * `asset_amount_out`: Amount that will be transferred from the pool to the provider.
    /// * `max_price`: Market price must be equal or less than the provided value.
    /// * `handle_fees`: Whether additional fees are handled or not (sets LP fee to 0)
    #[allow(clippy::too_many_arguments)]
    fn swap_exact_amount_out(
        who: AccountId,
        pool_id: PoolId,
        asset_in: Self::Asset,
        max_amount_asset_in: Option<Self::Balance>,
        asset_out: Self::Asset,
        asset_amount_out: Self::Balance,
        max_price: Option<Self::Balance>,
    ) -> Result<Weight, DispatchError>;
}
