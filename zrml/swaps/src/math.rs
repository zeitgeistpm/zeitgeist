use crate::{CheckArithmRslt, EXIT_FEE};
use frame_support::dispatch::DispatchError;
use sp_runtime::{FixedPointNumber, FixedU128};

pub fn calc_spot_price(
    asset_balance_in: FixedU128,
    asset_weight_in: FixedU128,
    asset_balance_out: FixedU128,
    asset_weight_out: FixedU128,
    swap_fee: FixedU128,
) -> Result<FixedU128, DispatchError> {
    let numer = asset_balance_in.check_div_rslt(&asset_weight_in)?;
    let denom = asset_balance_out.check_div_rslt(&asset_weight_out)?;
    let ratio = numer.check_div_rslt(&denom)?;
    let scale = FixedU128::one().check_div_rslt(&FixedU128::one().check_sub_rslt(&swap_fee)?)?;
    let spot_price = ratio.check_mul_rslt(&scale);
    spot_price
}

pub fn calc_out_given_in(
    asset_balance_in: FixedU128,
    asset_weight_in: FixedU128,
    asset_balance_out: FixedU128,
    asset_weight_out: FixedU128,
    asset_amount_in: FixedU128,
    swap_fee: FixedU128,
) -> Result<FixedU128, DispatchError> {
    let weight_ratio = asset_weight_in.check_div_rslt(&asset_weight_out)?;
    let mut adjusted_in = FixedU128::one().check_sub_rslt(&swap_fee)?;
    adjusted_in = adjusted_in.check_mul_rslt(&asset_amount_in)?;
    let y = asset_balance_in.check_div_rslt(&asset_balance_in.check_add_rslt(&adjusted_in)?)?;
    let foo = fixedu128_pow(y, weight_ratio)?;
    let bar = FixedU128::one().check_sub_rslt(&foo)?;
    let asset_amount_out = asset_balance_out.check_mul_rslt(&foo);
    asset_amount_out
}

pub fn calc_in_given_out(
    asset_balance_in: FixedU128,
    asset_weight_in: FixedU128,
    asset_balance_out: FixedU128,
    asset_weight_out: FixedU128,
    asset_amount_out: FixedU128,
    swap_fee: FixedU128,
) -> Result<FixedU128, DispatchError> {
    let weight_ratio = asset_weight_out.check_div_rslt(&asset_weight_in)?;
    let diff = asset_balance_out.check_sub_rslt(&asset_amount_out)?;
    let y = asset_balance_out.check_div_rslt(&diff)?;
    let foo = fixedu128_pow(y, weight_ratio)?.check_sub_rslt(&FixedU128::one())?;
    let mut asset_amount_in = FixedU128::one().check_sub_rslt(&swap_fee)?;
    asset_amount_in = asset_balance_in
        .check_mul_rslt(&foo)?
        .check_div_rslt(&asset_amount_in)?;
    Ok(asset_amount_in)
}

pub fn calc_pool_out_given_single_in(
    asset_balance_in: FixedU128,
    asset_weight_in: FixedU128,
    pool_supply: FixedU128,
    total_weight: FixedU128,
    asset_amount_in: FixedU128,
    swap_fee: FixedU128,
) -> Result<FixedU128, DispatchError> {
    // Charge the trading fee for the proportion of tokenAi
    //  which is implicitly traded to the other pool tokens.
    // That proportion is (1 - weightTokenIn)
    // tokenAiAfterFee = tAi * (1 - (1 - weighTi) * pool_fee)
    let normalized_weight = asset_weight_in.check_div_rslt(&total_weight)?;
    let zaz = FixedU128::one()
        .check_sub_rslt(&normalized_weight)?
        .check_mul_rslt(&swap_fee)?;
    let asset_amount_in_after_fee =
        asset_amount_in.check_mul_rslt(&FixedU128::one().check_sub_rslt(&zaz)?)?;

    let new_asset_balance_in = asset_balance_in.check_add_rslt(&asset_amount_in_after_fee)?;
    let asset_in_ratio = new_asset_balance_in.check_div_rslt(&asset_balance_in)?;

    let pool_ratio = fixedu128_pow(asset_in_ratio, normalized_weight)?;
    let new_pool_supply = pool_ratio.check_mul_rslt(&pool_supply)?;
    let pool_amount_out = new_pool_supply.check_sub_rslt(&pool_supply);
    pool_amount_out
}

pub fn calc_single_in_given_pool_out(
    asset_balance_in: FixedU128,
    asset_weight_in: FixedU128,
    pool_supply: FixedU128,
    total_weight: FixedU128,
    pool_amount_out: FixedU128,
    swap_fee: FixedU128,
) -> Result<FixedU128, DispatchError> {
    let normalized_weight = asset_weight_in.check_div_rslt(&total_weight)?;
    let new_pool_supply = pool_supply.check_add_rslt(&pool_amount_out)?;
    let pool_ratio = new_pool_supply.check_div_rslt(&pool_supply)?;

    let asset_in_ratio = fixedu128_pow(pool_ratio, normalized_weight)?;
    let new_asset_balance_in = asset_in_ratio.check_mul_rslt(&asset_balance_in)?;
    let asset_amount_in_after_fee = new_asset_balance_in.check_sub_rslt(&asset_balance_in)?;

    let zar = FixedU128::one()
        .check_sub_rslt(&normalized_weight)?
        .check_mul_rslt(&swap_fee)?;
    let asset_amount_in =
        asset_amount_in_after_fee.check_div_rslt(&FixedU128::one().check_sub_rslt(&zar)?);
    asset_amount_in
}

pub fn calc_single_out_given_pool_in(
    asset_balance_out: FixedU128,
    asset_weight_out: FixedU128,
    pool_supply: FixedU128,
    total_weight: FixedU128,
    pool_amount_in: FixedU128,
    swap_fee: FixedU128,
) -> Result<FixedU128, DispatchError> {
    let normalized_weight = asset_weight_out.check_div_rslt(&total_weight)?;

    let pool_amount_in_after_exit_fee =
        pool_amount_in.check_mul_rslt(&FixedU128::one().check_div_rslt(&EXIT_FEE)?)?;
    let new_pool_supply = pool_supply.check_sub_rslt(&pool_amount_in_after_exit_fee)?;
    let pool_ratio = new_pool_supply.check_div_rslt(&pool_supply)?;

    let exp = FixedU128::one().check_div_rslt(&normalized_weight)?;
    let asset_out_ratio = fixedu128_pow(pool_ratio, exp)?;
    let new_asset_balance_out = asset_out_ratio.check_mul_rslt(&asset_balance_out)?;

    let asset_amount_before_swap_fee = asset_balance_out.check_sub_rslt(&new_asset_balance_out)?;

    let zaz = FixedU128::one()
        .check_sub_rslt(&normalized_weight)?
        .check_mul_rslt(&swap_fee)?;
    let asset_amount_out =
        asset_amount_before_swap_fee.check_mul_rslt(&FixedU128::one().check_sub_rslt(&zaz)?);
    asset_amount_out
}

pub fn calc_pool_in_given_single_out(
    asset_balance_out: FixedU128,
    asset_weight_out: FixedU128,
    pool_supply: FixedU128,
    total_weight: FixedU128,
    asset_amount_out: FixedU128,
    swap_fee: FixedU128,
) -> Result<FixedU128, DispatchError> {
    let normalized_weight = asset_weight_out.check_div_rslt(&total_weight)?;
    let zoo = FixedU128::one().check_sub_rslt(&normalized_weight)?;
    let zar = zoo.check_mul_rslt(&swap_fee)?;
    let asset_amount_out_before_swap_fee =
        asset_amount_out.check_div_rslt(&FixedU128::one().check_sub_rslt(&zar)?)?;

    let new_asset_balance_out =
        asset_balance_out.check_sub_rslt(&asset_amount_out_before_swap_fee)?;
    let asset_out_ratio = new_asset_balance_out.check_div_rslt(&asset_balance_out)?;

    let pool_ratio = fixedu128_pow(asset_out_ratio, normalized_weight)?;
    let new_pool_supply = pool_ratio.check_mul_rslt(&pool_supply)?;
    let pool_amount_in_after_exit_fee = pool_supply.check_sub_rslt(&new_pool_supply)?;

    let pool_amount_in =
        pool_amount_in_after_exit_fee.check_div_rslt(&FixedU128::one().check_sub_rslt(&EXIT_FEE)?);
    pool_amount_in
}

#[inline]
fn fixedu128_pow(a: FixedU128, b: FixedU128) -> Result<FixedU128, DispatchError> {
    Ok(a.into_inner().check_pow_rslt(&b.into_inner())?.into())
}
