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
use frame_support::weights::Weight;
use sp_runtime::{DispatchError, DispatchResult};

pub trait Swaps<AccountId> {
    type Asset;
    type Balance;

    /// Creates a new pool.
    ///
    /// # Arguments
    ///
    /// * `who`: The account that is the creator of the pool.
    /// * `assets`: The assets that are used in the pool.
    /// * `swap_fee`: The fee applied to each swap.
    /// * `amount`: The amount of each asset added to the pool.
    /// * `weights`: The denormalized weights.
    fn create_pool(
        who: AccountId,
        assets: Vec<Self::Asset>,
        swap_fee: Self::Balance,
        amount: Self::Balance,
        weights: Vec<Self::Balance>,
    ) -> Result<PoolId, DispatchError>;

    /// Close the specified pool.
    fn close_pool(pool_id: PoolId) -> Result<Weight, DispatchError>;

    /// Destroy pool, slash pool account assets and destroy pool shares of the liquidity providers.
    fn destroy_pool(pool_id: PoolId) -> Result<Weight, DispatchError>;

    /// Open the specified pool.
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

    /// Buy the `asset_out`/`asset_in` pair from the pool for an exact amount of `asset_in`.
    ///
    /// This function will error if both `min_asset_amount_out` and `max_price` are `None`.
    ///
    /// # Arguments
    ///
    /// * `who`: The user executing the trade.
    /// * `pool_id`: The pool to execute the trade on.
    /// * `asset_in`: Asset entering the pool.
    /// * `asset_amount_in`: Exact mount that will be transferred from the user to the pool.
    /// * `asset_out`: Asset leaving the pool.
    /// * `min_asset_amount_out`: Minimum asset amount requested by the user. The trade is rolled
    ///   back if this limit is violated. If this is `None`, there is no limit.
    /// * `max_price`: The maximum price _after execution_ the user is willing to accept. The trade
    ///   is rolled back if this limit is violated. If this is `None`, there is no limit.
    fn swap_exact_amount_in(
        who: AccountId,
        pool_id: PoolId,
        asset_in: Self::Asset,
        asset_amount_in: Self::Balance,
        asset_out: Self::Asset,
        min_asset_amount_out: Option<Self::Balance>,
        max_price: Option<Self::Balance>,
    ) -> DispatchResult;

    /// Buy the `asset_out`/`asset_in` pair from the pool, receiving an exact amount of `asset_out`.
    ///
    /// This function will error if both `min_asset_amount_out` and `max_price` are `None`.
    ///
    /// # Arguments
    ///
    /// * `who`: The user executing the trade.
    /// * `pool_id`: The pool to execute the trade on.
    /// * `asset_in`: Asset entering the pool.
    /// * `max_asset_amount_out`: Maximum asset amount the user is willing to pay. The trade is
    ///   rolled back if this limit is violated.
    /// * `asset_out`: Asset leaving the pool.
    /// * `asset_amount_out`: Exact amount that will be transferred from the user to the pool.
    /// * `max_price`: The maximum price _after execution_ the user is willing to accept. The trade
    ///   is rolled back if this limit is violated. If this is `None`, there is no limit.
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
