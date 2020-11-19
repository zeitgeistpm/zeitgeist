use crate::consts::*;
use crate::fixed::*;

pub fn calc_spot_price(
    asset_balance_in: u128,
    asset_weight_in: u128,
    asset_balance_out: u128,
    asset_weight_out: u128,
    swap_fee: u128,
) -> u128 // spot_price
{
    let numer = bdiv(asset_balance_in, asset_weight_in);
    if numer == 0 { panic!("numer is zero"); }; //debug
    let denom = bdiv(asset_balance_out, asset_weight_out);
    if denom == 0 { panic!("denom is zero"); }; //debug
    let ratio = bdiv(numer, denom);
    if ratio == 0 { panic!("ratio is zero"); }; //debug
    let scale = bdiv(BASE, BASE - swap_fee);
    bmul(ratio,  scale)
}

pub fn calc_out_given_in(
    asset_balance_in: u128,
    asset_weight_in: u128,
    asset_balance_out: u128,
    asset_weight_out: u128,
    asset_amount_in: u128,
    swap_fee: u128,
) -> u128 // asset_amount_out
{
    let weight_ratio = bdiv(asset_weight_in, asset_weight_out);
    let mut adjusted_in = BASE - swap_fee;
    adjusted_in = bmul(adjusted_in, asset_amount_in);
    let y = bdiv(asset_balance_in, asset_balance_in + adjusted_in);
    let foo = bpow(y, weight_ratio);
    let bar = BASE - foo;
    bmul(asset_balance_out, bar)
}

pub fn calc_in_given_out(
    asset_balance_in: u128,
    asset_weight_in: u128,
    asset_balance_out: u128,
    asset_weight_out: u128,
    asset_amount_out: u128,
    swap_fee: u128,
) -> u128 // asset_amount_in
{
    let weight_ratio = bdiv(asset_weight_in, asset_weight_out);
    let diff = asset_balance_out - asset_amount_out;
    let y = bdiv(asset_balance_out, diff);
    let foo = bpow(y, weight_ratio);
    let mut asset_amount_in = BASE - swap_fee;
    asset_amount_in = bdiv(bmul(asset_balance_in, foo), asset_amount_in);
    asset_amount_in
}

pub fn calc_pool_out_given_single_in(
    asset_balance_in: u128,
    asset_weight_in: u128,
    pool_supply: u128,
    total_weight: u128,
    asset_amount_in: u128,
    swap_fee: u128,
) -> u128 // pool_amount_out
{
    // Charge the trading fee for the proportion of tokenAi
    //  which is implicitly traded to the other pool tokens.
    // That proportion is (1 - weightTokenIn)
    // tokenAiAfterFee = tAi * (1 - (1 - weighTi) * pool_fee)
    let normalized_weight = bdiv(asset_weight_in, total_weight);
    let zaz = bmul(BASE - normalized_weight, swap_fee);
    let asset_amount_in_after_fee = bmul(asset_amount_in, BASE - zaz);

    let new_asset_balance_in = asset_balance_in + asset_amount_in_after_fee;
    let asset_in_ratio = bdiv(new_asset_balance_in, asset_balance_in);

    let pool_ratio = bpow(asset_in_ratio, normalized_weight);
    let new_pool_supply = bmul(pool_ratio, pool_supply);
    let pool_amount_out = new_pool_supply - pool_supply;
    pool_amount_out
}

pub fn calc_single_in_given_pool_out(
    asset_balance_in: u128,
    asset_weight_in: u128,
    pool_supply: u128,
    total_weight: u128,
    pool_amount_out: u128,
    swap_fee: u128,
) -> u128 // asset_amount_in
{
    let normalized_weight = bdiv(asset_weight_in, total_weight);
    let new_pool_supply = pool_supply + pool_amount_out;
    let pool_ratio = bdiv(new_pool_supply, pool_supply);

    let _boo = bdiv(BASE, normalized_weight);
    let asset_in_ratio = bpow(pool_ratio, normalized_weight);
    let new_asset_balance_in = bmul(asset_in_ratio, asset_balance_in);
    let asset_amount_in_after_fee = new_asset_balance_in - asset_balance_in;

    let zar = bmul(BASE - normalized_weight, swap_fee);
    let asset_amount_in = bdiv(asset_amount_in_after_fee, BASE - zar);
    asset_amount_in
}

pub fn calc_single_out_given_pool_in(
    asset_balance_out: u128,
    asset_weight_out: u128,
    pool_supply: u128,
    total_weight: u128,
    pool_amount_in: u128,
    swap_fee: u128,
) -> u128 // asset_amount_out
{
    let normalized_weight = bdiv(asset_weight_out, total_weight);

    let pool_amount_in_after_exit_fee = bmul(pool_amount_in, BASE - EXIT_FEE);
    let new_pool_supply = pool_supply - pool_amount_in_after_exit_fee;
    let pool_ratio = bdiv(new_pool_supply, pool_supply);

    let exp = bdiv(BASE, normalized_weight);
    let asset_out_ratio = bpow(pool_ratio, exp);
    let new_asset_balance_out = bmul(asset_out_ratio, asset_balance_out);

    let asset_amount_before_swap_fee = asset_balance_out - new_asset_balance_out;

    let zaz = bmul(BASE - normalized_weight, swap_fee);
    let asset_amount_out = bmul(asset_amount_before_swap_fee, BASE - zaz);
    asset_amount_out
}

pub fn calc_pool_in_given_single_out(
    asset_balance_out: u128,
    asset_weight_out: u128,
    pool_supply: u128,
    total_weight: u128,
    asset_amount_out: u128,
    swap_fee: u128,
) -> u128 // pool_amount_in
{
    let normalized_weight = bdiv(asset_weight_out, total_weight);
    let zoo = BASE - normalized_weight;
    let zar = bmul(zoo, swap_fee);
    let asset_amount_out_before_swap_fee = bdiv(asset_amount_out, BASE - zar);

    let new_asset_balance_out = asset_balance_out - asset_amount_out_before_swap_fee;
    let asset_out_ratio = bdiv(new_asset_balance_out, asset_balance_out);

    let pool_ratio = bpow(asset_out_ratio, normalized_weight);
    let new_pool_supply = bmul(pool_ratio, pool_supply);
    let pool_amount_in_after_exit_fee = pool_supply - new_pool_supply;

    let pool_amount_in = bdiv(pool_amount_in_after_exit_fee, BASE - EXIT_FEE);
    pool_amount_in
}
