#![allow(clippy::let_and_return)]

use crate::{
    check_arithm_rslt::CheckArithmRslt,
    fixed::{bdiv, bmul, bpow},
};
use frame_support::dispatch::DispatchError;
use zeitgeist_primitives::constants::BASE;

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
    let numer = bdiv(asset_balance_in, asset_weight_in)?;
    let denom = bdiv(asset_balance_out, asset_weight_out)?;
    let ratio = bdiv(numer, denom)?;
    let scale = bdiv(BASE, BASE.check_sub_rslt(&swap_fee)?)?;
    let spot_price = bmul(ratio, scale);
    spot_price
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
    let weight_ratio = bdiv(asset_weight_in, asset_weight_out)?;
    let mut adjusted_in = BASE.check_sub_rslt(&swap_fee)?;
    adjusted_in = bmul(adjusted_in, asset_amount_in)?;
    let y = bdiv(asset_balance_in, asset_balance_in.check_add_rslt(&adjusted_in)?)?;
    let pow = bpow(y, weight_ratio)?;
    let bar = BASE.check_sub_rslt(&pow)?;
    let asset_amount_out = bmul(asset_balance_out, bar);
    asset_amount_out
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
    let weight_ratio = bdiv(asset_weight_out, asset_weight_in)?;
    let diff = asset_balance_out.check_sub_rslt(&asset_amount_out)?;
    let y = bdiv(asset_balance_out, diff)?;
    let pow = bpow(y, weight_ratio)?.check_sub_rslt(&BASE)?;
    let asset_amount_in = bdiv(bmul(asset_balance_in, pow)?, BASE.check_sub_rslt(&swap_fee)?);
    asset_amount_in
}

/// Calculate the amount of pool tokens received when joining the pool with a specified amount of
/// tokens of a single asset.
///
/// See _Single-Asset Deposit/Withdrawal_ of Martinelli-Mushegian: Balancer Whitepaper v2019-09-19
/// (https://balancer.fi/whitepaper.pdf) for details.
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
    let normalized_weight = bdiv(asset_weight_in, total_weight)?;
    let zaz = bmul(BASE.check_sub_rslt(&normalized_weight)?, swap_fee)?;
    let asset_amount_in_after_fee = bmul(asset_amount_in, BASE.check_sub_rslt(&zaz)?)?;
    let new_asset_balance_in = asset_balance_in.check_add_rslt(&asset_amount_in_after_fee)?;
    let asset_in_ratio = bdiv(new_asset_balance_in, asset_balance_in)?;

    let pool_ratio = bpow(asset_in_ratio, normalized_weight)?;
    let new_pool_supply = bmul(pool_ratio, pool_supply)?;
    let pool_amount_out = new_pool_supply.check_sub_rslt(&pool_supply);
    pool_amount_out
}

/// Calculate the required amount of tokens of a single asset to join the pool with to receive the
/// specified amount of pool tokens.
///
/// See _Single-Asset Deposit/Withdrawal_ of Martinelli-Mushegian: Balancer Whitepaper v2019-09-19
/// (https://balancer.fi/whitepaper.pdf) for details.
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
    let normalized_weight = bdiv(asset_weight_in, total_weight)?;
    let new_pool_supply = pool_supply.check_add_rslt(&pool_amount_out)?;
    let pool_ratio = bdiv(new_pool_supply, pool_supply)?;

    let boo = bdiv(BASE, normalized_weight)?;
    let asset_in_ratio = bpow(pool_ratio, boo)?;
    let new_asset_balance_in = bmul(asset_in_ratio, asset_balance_in)?;
    let asset_amount_in_after_fee = new_asset_balance_in.check_sub_rslt(&asset_balance_in)?;

    let zar = bmul(BASE.check_sub_rslt(&normalized_weight)?, swap_fee)?;
    let asset_amount_in = bdiv(asset_amount_in_after_fee, BASE.check_sub_rslt(&zar)?);
    asset_amount_in
}

/// Calculate the amount of tokens of a single asset received when exiting the pool with a specified amount of
/// pool tokens.
///
/// See _Single-Asset Deposit/Withdrawal_ of Martinelli-Mushegian: Balancer Whitepaper v2019-09-19
/// (https://balancer.fi/whitepaper.pdf) for details.
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
    let normalized_weight = bdiv(asset_weight_out, total_weight)?;

    let pool_amount_in_after_exit_fee = bmul(pool_amount_in, BASE.check_sub_rslt(&exit_fee)?)?;
    let new_pool_supply = pool_supply.check_sub_rslt(&pool_amount_in_after_exit_fee)?;
    let pool_ratio = bdiv(new_pool_supply, pool_supply)?;

    let exp = bdiv(BASE, normalized_weight)?;
    let asset_out_ratio = bpow(pool_ratio, exp)?;
    let new_asset_balance_out = bmul(asset_out_ratio, asset_balance_out)?;

    let asset_amount_before_swap_fee = asset_balance_out.check_sub_rslt(&new_asset_balance_out)?;

    let zaz = bmul(BASE.check_sub_rslt(&normalized_weight)?, swap_fee)?;
    let asset_amount_out = bmul(asset_amount_before_swap_fee, BASE.check_sub_rslt(&zaz)?);
    asset_amount_out
}

/// Calculate the required amount of pool tokens to exit the pool with to receive the specified number of tokens of a single asset.
///
/// See _Single-Asset Deposit/Withdrawal_ of Martinelli-Mushegian: Balancer Whitepaper v2019-09-19
/// (https://balancer.fi/whitepaper.pdf) for details.
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
    let normalized_weight = bdiv(asset_weight_out, total_weight)?;
    let zoo = BASE.check_sub_rslt(&normalized_weight)?;
    let zar = bmul(zoo, swap_fee)?;
    let asset_amount_out_before_swap_fee = bdiv(asset_amount_out, BASE.check_sub_rslt(&zar)?)?;

    let new_asset_balance_out =
        asset_balance_out.check_sub_rslt(&asset_amount_out_before_swap_fee)?;
    let asset_out_ratio = bdiv(new_asset_balance_out, asset_balance_out)?;

    let pool_ratio = bpow(asset_out_ratio, normalized_weight)?;
    let new_pool_supply = bmul(pool_ratio, pool_supply)?;

    let pool_amount_in_after_exit_fee = pool_supply.check_sub_rslt(&new_pool_supply)?;
    let pool_amount_in = bdiv(pool_amount_in_after_exit_fee, BASE.check_sub_rslt(&exit_fee)?);
    pool_amount_in
}
