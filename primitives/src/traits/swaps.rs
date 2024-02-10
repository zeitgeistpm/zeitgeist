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
use sp_runtime::DispatchResult;

pub trait Swaps<AccountId> {
    type Asset;
    type Balance;
    // TODO(#1216): Add weight type which implements `Into<Balance>` and `From<Balance>`

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
        weights: Vec<u128>,
    ) -> Result<PoolId, DispatchError>;

    /// Close the specified pool.
    fn close_pool(pool_id: PoolId) -> Result<Weight, DispatchError>;

    /// Destroy CPMM pool, slash pool account assets and destroy pool shares of the liquidity providers.
    fn destroy_pool(pool_id: PoolId) -> Result<Weight, DispatchError>;

    fn open_pool(pool_id: PoolId) -> Result<Weight, DispatchError>;

    /// Exchanges an LP's (liquidity provider's) pool shares for a proportionate and exact
    /// amount of _one_ of the pool's assets. The assets received are distributed according to
    /// the LP's percentage ownership of the pool.
    ///
    /// # Arguments
    ///
    /// * `who`: The LP.
    /// * `pool_id`: The ID of the pool to withdraw from.
    /// * `asset`: The asset received by the LP.
    /// * `asset_amount`: The amount of `asset` leaving the pool.
    /// * `max_pool_amount`: The maximum amount of pool shares the LP is willing to burn. The
    ///   transaction is rolled back if this bound is violated.
    fn pool_exit_with_exact_asset_amount(
        who: AccountId,
        pool_id: PoolId,
        asset: Self::Asset,
        asset_amount: Self::Balance,
        max_pool_amount: Self::Balance,
    ) -> DispatchResult;

    /// Exchanges an exact amount of an LP's (liquidity provider's) holds of _one_ of the assets in
    /// the pool for pool shares.
    ///
    /// # Arguments
    ///
    /// * `who`: The LP.
    /// * `pool_id`: The ID of the pool to withdraw from.
    /// * `asset_in`: The asset entering the pool.
    /// * `asset_amount`: Asset amount that is entering the pool.
    /// * `min_pool_amount`: The minimum amount of pool shares the LP asks to receive. The
    ///   transaction is rolled back if this bound is violated.
    fn pool_join_with_exact_asset_amount(
        who: AccountId,
        pool_id: PoolId,
        asset_in: Self::Asset,
        asset_amount: Self::Balance,
        min_pool_amount: Self::Balance,
    ) -> DispatchResult;

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
    fn swap_exact_amount_in(
        who: AccountId,
        pool_id: PoolId,
        asset_in: Self::Asset,
        asset_amount_in: Self::Balance,
        asset_out: Self::Asset,
        min_asset_amount_out: Option<Self::Balance>,
        max_price: Option<Self::Balance>,
    ) -> DispatchResult;

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
    fn swap_exact_amount_out(
        who: AccountId,
        pool_id: PoolId,
        asset_in: Self::Asset,
        max_amount_asset_in: Option<Self::Balance>,
        asset_out: Self::Asset,
        asset_amount_out: Self::Balance,
        max_price: Option<Self::Balance>,
    ) -> DispatchResult;
}
