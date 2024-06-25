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
//
// This file incorporates work covered by the license above but
// published without copyright notice by Balancer Labs
// (<https://balancer.finance>, contact@balancer.finance) in the
// balancer-core repository
// <https://github.com/balancer-labs/balancer-core>.

#![allow(clippy::let_and_return)]

use crate::fixed::bpow;
use sp_runtime::DispatchError;
use zeitgeist_primitives::{
    constants::BASE,
    math::{
        checked_ops_res::{CheckedAddRes, CheckedSubRes},
        fixed::{FixedDiv, FixedMul},
    },
};

/// Calculate the spot price of one asset in terms of another, including trading fees.
///
/// # Arguments
///
/// * `asset_balance_in` - The pool balance of the ingoing asset
/// * `asset_weight_in` - The weight of the ingoing asset
/// * `asset_balance_out` - The pool balance of the outgoing asset
/// * `asset_weight_out` - The weight of the outgoing asset
/// * `swap_fee` - The swap fee of the pool
///
/// # Errors
///
/// Returns `DispatchError::Other` if `swap_fee >= BASE`.
pub fn calc_spot_price(
    asset_balance_in: u128,
    asset_weight_in: u128,
    asset_balance_out: u128,
    asset_weight_out: u128,
    swap_fee: u128,
) -> Result<u128, DispatchError> {
    let numer = asset_balance_in.bdiv(asset_weight_in)?;
    let denom = asset_balance_out.bdiv(asset_weight_out)?;
    let ratio = numer.bdiv(denom)?;
    let scale = BASE.bdiv(BASE.checked_sub_res(&swap_fee)?)?;
    ratio.bmul(scale)
}

/// Calculate the amount of tokens received from the pool for swapping the specified amount of tokens in, including
/// trading fees.
///
/// # Arguments
///
/// * `asset_balance_in` - The pool balance of the ingoing asset
/// * `asset_weight_in` - The weight of the ingoing asset
/// * `asset_balance_out` - The pool balance of the outgoing asset
/// * `asset_weight_out` - The weight of the outgoing asset
/// * `asset_amount_in` - The amount of the ingoing asset to swap in
/// * `swap_fee` - The swap fee of the pool
///
/// # Errors
///
/// Returns `DispatchError::Other` if `swap_fee >= BASE`.
pub fn calc_out_given_in(
    asset_balance_in: u128,
    asset_weight_in: u128,
    asset_balance_out: u128,
    asset_weight_out: u128,
    asset_amount_in: u128,
    swap_fee: u128,
) -> Result<u128, DispatchError> {
    let weight_ratio = asset_weight_in.bdiv(asset_weight_out)?;
    let mut adjusted_in = BASE.checked_sub_res(&swap_fee)?;
    adjusted_in = adjusted_in.bmul(asset_amount_in)?;
    let y = asset_balance_in.bdiv(asset_balance_in.checked_add_res(&adjusted_in)?)?;
    let pow = bpow(y, weight_ratio)?;
    let bar = BASE.checked_sub_res(&pow)?;
    asset_balance_out.bmul(bar)
}

/// Calculate the required amount of tokens to swap in to receive a specified amount of tokens
/// from the pool, including trading fees.
///
/// # Arguments
///
/// * `asset_balance_in` - The pool balance of the ingoing asset
/// * `asset_weight_in` - The weight of the ingoing asset
/// * `asset_balance_out` - The pool balance of the outgoing asset
/// * `asset_weight_out` - The weight of the outgoing asset
/// * `asset_amount_out` - The amount of tokens of the outgoing asset to receive
/// * `swap_fee` - The swap fee of the pool
///
/// # Errors
///
/// Returns `DispatchError::Other` if `swap_fee >= BASE`.
pub fn calc_in_given_out(
    asset_balance_in: u128,
    asset_weight_in: u128,
    asset_balance_out: u128,
    asset_weight_out: u128,
    asset_amount_out: u128,
    swap_fee: u128,
) -> Result<u128, DispatchError> {
    let weight_ratio = asset_weight_out.bdiv(asset_weight_in)?;
    let diff = asset_balance_out.checked_sub_res(&asset_amount_out)?;
    let y = asset_balance_out.bdiv(diff)?;
    let pow = bpow(y, weight_ratio)?.checked_sub_res(&BASE)?;
    asset_balance_in.bmul(pow)?.bdiv(BASE.checked_sub_res(&swap_fee)?)
}

/// Calculate the amount of pool tokens received when joining the pool with a specified amount of
/// tokens of a single asset.
///
/// See _Single-Asset Deposit/Withdrawal_ of Martinelli-Mushegian: Balancer Whitepaper v2019-09-19
/// (<https://balancer.fi/whitepaper.pdf>) for details.
///
/// * `asset_balance_in` - The pool balance of the ingoing asset
/// * `asset_weight_in` - The weight of the ingoing asset
/// * `pool_supply` - The total supply of pool tokens
/// * `total_weight` - The total weight of the pool (sum of all weights)
/// * `asset_amount_in` - The amount of the ingoing asset to add to the pool
/// * `swap_fee` - The swap fee of the pool
///
/// # Errors
///
/// Returns `DispatchError::Other` if `swap_fee >= BASE`
pub fn calc_pool_out_given_single_in(
    asset_balance_in: u128,
    asset_weight_in: u128,
    pool_supply: u128,
    total_weight: u128,
    asset_amount_in: u128,
    swap_fee: u128,
) -> Result<u128, DispatchError> {
    // Charge the trading fee for the proportion of tokenAi
    //  which is implicitly traded to the other pool tokens.
    // That proportion is (1 - weightTokenIn)
    // tokenAiAfterFee = tAi * (1 - (1 - weighTi) * pool_fee)
    let normalized_weight = asset_weight_in.bdiv(total_weight)?;
    let zaz = BASE.checked_sub_res(&normalized_weight)?.bmul(swap_fee)?;
    let asset_amount_in_after_fee = asset_amount_in.bmul(BASE.checked_sub_res(&zaz)?)?;
    let new_asset_balance_in = asset_balance_in.checked_add_res(&asset_amount_in_after_fee)?;
    let asset_in_ratio = new_asset_balance_in.bdiv(asset_balance_in)?;

    let pool_ratio = bpow(asset_in_ratio, normalized_weight)?;
    let new_pool_supply = pool_ratio.bmul(pool_supply)?;
    new_pool_supply.checked_sub_res(&pool_supply)
}

/// Calculate the required amount of tokens of a single asset to join the pool with to receive the
/// specified amount of pool tokens.
///
/// See _Single-Asset Deposit/Withdrawal_ of Martinelli-Mushegian: Balancer Whitepaper v2019-09-19
/// (<https://balancer.fi/whitepaper.pdf>) for details.
///
/// * `asset_balance_in` - The pool balance of the ingoing asset
/// * `asset_weight_in` - The weight of the ingoing asset
/// * `pool_supply` - The total supply of pool tokens
/// * `total_weight` - The total weight of the pool (sum of all weights)
/// * `pool_amount_out` - The expected amount of pool tokens to receive
/// * `swap_fee` - The swap fee of the pool
///
/// # Errors
///
/// Returns `DispatchError::Other` if `swap_fee >= BASE`
pub fn calc_single_in_given_pool_out(
    asset_balance_in: u128,
    asset_weight_in: u128,
    pool_supply: u128,
    total_weight: u128,
    pool_amount_out: u128,
    swap_fee: u128,
) -> Result<u128, DispatchError> {
    let normalized_weight = asset_weight_in.bdiv(total_weight)?;
    let new_pool_supply = pool_supply.checked_add_res(&pool_amount_out)?;
    let pool_ratio = new_pool_supply.bdiv(pool_supply)?;

    let boo = BASE.bdiv(normalized_weight)?;
    let asset_in_ratio = bpow(pool_ratio, boo)?;
    let new_asset_balance_in = asset_in_ratio.bmul(asset_balance_in)?;
    let asset_amount_in_after_fee = new_asset_balance_in.checked_sub_res(&asset_balance_in)?;

    let zar = BASE.checked_sub_res(&normalized_weight)?.bmul(swap_fee)?;
    asset_amount_in_after_fee.bdiv(BASE.checked_sub_res(&zar)?)
}

/// Calculate the amount of tokens of a single asset received when exiting the pool with a specified amount of
/// pool tokens.
///
/// See _Single-Asset Deposit/Withdrawal_ of Martinelli-Mushegian: Balancer Whitepaper v2019-09-19
/// (<https://balancer.fi/whitepaper.pdf>) for details.
///
/// * `asset_balance_in` - The pool balance of the ingoing asset
/// * `asset_weight_in` - The weight of the ingoing asset
/// * `pool_supply` - The total supply of pool tokens
/// * `total_weight` - The total weight of the pool (sum of all weights)
/// * `pool_amount_in` - The amount of pool tokens to burn
/// * `swap_fee` - The swap fee of the pool
///
/// # Errors
///
/// Returns `DispatchError::Other` if `swap_fee >= BASE`
pub fn calc_single_out_given_pool_in(
    asset_balance_out: u128,
    asset_weight_out: u128,
    pool_supply: u128,
    total_weight: u128,
    pool_amount_in: u128,
    swap_fee: u128,
    exit_fee: u128,
) -> Result<u128, DispatchError> {
    let normalized_weight = asset_weight_out.bdiv(total_weight)?;

    let pool_amount_in_after_exit_fee = pool_amount_in.bmul(BASE.checked_sub_res(&exit_fee)?)?;
    let new_pool_supply = pool_supply.checked_sub_res(&pool_amount_in_after_exit_fee)?;
    let pool_ratio = new_pool_supply.bdiv(pool_supply)?;

    let exp = BASE.bdiv(normalized_weight)?;
    let asset_out_ratio = bpow(pool_ratio, exp)?;
    let new_asset_balance_out = asset_out_ratio.bmul(asset_balance_out)?;

    let asset_amount_before_swap_fee = asset_balance_out.checked_sub_res(&new_asset_balance_out)?;

    let zaz = BASE.checked_sub_res(&normalized_weight)?.bmul(swap_fee)?;
    asset_amount_before_swap_fee.bmul(BASE.checked_sub_res(&zaz)?)
}

/// Calculate the required amount of pool tokens to exit the pool with to receive the specified number of tokens of a single asset.
///
/// See _Single-Asset Deposit/Withdrawal_ of Martinelli-Mushegian: Balancer Whitepaper v2019-09-19
/// (<https://balancer.fi/whitepaper.pdf>) for details.
///
/// * `asset_balance_in` - The pool balance of the ingoing asset
/// * `asset_weight_in` - The weight of the ingoing asset
/// * `pool_supply` - The total supply of pool tokens
/// * `total_weight` - The total weight of the pool (sum of all weights)
/// * `asset_amount_out` - The expected amount of the outgoing asset to receive
/// * `swap_fee` - The swap fee of the pool
///
/// # Errors
///
/// Returns `DispatchError::Other` if `swap_fee >= BASE`
pub fn calc_pool_in_given_single_out(
    asset_balance_out: u128,
    asset_weight_out: u128,
    pool_supply: u128,
    total_weight: u128,
    asset_amount_out: u128,
    swap_fee: u128,
    exit_fee: u128,
) -> Result<u128, DispatchError> {
    let normalized_weight = asset_weight_out.bdiv(total_weight)?;
    let zoo = BASE.checked_sub_res(&normalized_weight)?;
    let zar = zoo.bmul(swap_fee)?;
    let asset_amount_out_before_swap_fee = asset_amount_out.bdiv(BASE.checked_sub_res(&zar)?)?;

    let new_asset_balance_out =
        asset_balance_out.checked_sub_res(&asset_amount_out_before_swap_fee)?;
    let asset_out_ratio = new_asset_balance_out.bdiv(asset_balance_out)?;

    let pool_ratio = bpow(asset_out_ratio, normalized_weight)?;
    let new_pool_supply = pool_ratio.bmul(pool_supply)?;

    let pool_amount_in_after_exit_fee = pool_supply.checked_sub_res(&new_pool_supply)?;
    pool_amount_in_after_exit_fee.bdiv(BASE.checked_sub_res(&exit_fee)?)
}
