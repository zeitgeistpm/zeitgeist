use crate::consts::*;

pub fn calc_spot_price(
    asset_balanace_in: u128,
    asset_weight_in: u128,
    asset_balance_out: u128,
    asset_weight_out: u128,
    swap_fee: u128,
) -> u128 // spot_price
{
    let numer = asset_balanace_in / asset_weight_in;
    let denom = asset_balance_out / asset_weight_out;
    let ratio = numer / denom;
    let scale = BASE / (BASE - swap_fee);
    ratio * scale
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
    let weight_ratio = asset_weight_in / asset_weight_out;
    let mut adjusted_in = BASE - swap_fee;
    adjusted_in = adjusted_in * asset_amount_in;
    let y = asset_balance_in / (asset_balance_in + adjusted_in);
    let foo = y.pow(weight_ratio as u32);
    let bar = BASE - foo;
    asset_balance_out * bar
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
    let weight_ratio = asset_weight_in / asset_weight_out;
    let diff = asset_balance_out - asset_amount_out;
    let y = asset_balance_out / diff;
    let foo = y.pow(weight_ratio as u32);
    let mut asset_amount_in = BASE - swap_fee;
    asset_amount_in = asset_balance_in * foo / asset_amount_in;
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
    let normalized_weight = asset_weight_in / total_weight;
    let zaz = (BASE - normalized_weight) * swap_fee;
    let asset_amount_in_after_fee = asset_amount_in * (BASE - zaz);

    let new_asset_balance_in = asset_balance_in + asset_amount_in_after_fee;
    let asset_in_ratio = new_asset_balance_in / asset_balance_in;

    let pool_ratio = asset_in_ratio.pow(normalized_weight as u32);
    let new_pool_supply = pool_ratio * pool_supply;
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
    let normalized_weight = asset_weight_in / total_weight;
    let new_pool_supply = pool_supply + pool_amount_out;
    let pool_ratio = new_pool_supply / pool_supply;

    let _boo = BASE / normalized_weight;
    let asset_in_ratio = pool_ratio.pow(normalized_weight as u32);
    let new_asset_balance_in = asset_in_ratio * asset_balance_in;
    let asset_amount_in_after_fee = new_asset_balance_in - asset_balance_in;

    let zar = (BASE - normalized_weight) * swap_fee;
    let asset_amount_in = asset_amount_in_after_fee / (BASE - zar);
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
    let normalized_weight = asset_weight_out / total_weight;

    let pool_amount_in_after_exit_fee = pool_amount_in * (BASE - EXIT_FEE);
    let new_pool_supply = pool_supply - pool_amount_in_after_exit_fee;
    let pool_ratio = new_pool_supply / pool_supply;

    let exp = BASE / normalized_weight;
    let asset_out_ratio = pool_ratio.pow(exp as u32);
    let new_asset_balance_out = asset_out_ratio * asset_balance_out;

    let asset_amount_before_swap_fee = asset_balance_out - new_asset_balance_out;

    let zaz = (BASE - normalized_weight) * swap_fee;
    let asset_amount_out = asset_amount_before_swap_fee * (BASE - zaz);
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
    let normalized_weight = asset_weight_out / total_weight;
    let zoo = BASE - normalized_weight;
    let zar = zoo * swap_fee;
    let asset_amount_out_before_swap_fee = asset_amount_out / (BASE - zar);

    let new_asset_balance_out = asset_balance_out - asset_amount_out_before_swap_fee;
    let asset_out_ratio = new_asset_balance_out / asset_balance_out;

    let pool_ratio = asset_out_ratio.pow(normalized_weight as u32);
    let new_pool_supply = pool_ratio * pool_supply;
    let pool_amount_in_after_exit_fee = pool_supply - new_pool_supply;

    let pool_amount_in = pool_amount_in_after_exit_fee / (BASE - EXIT_FEE);
    pool_amount_in
}
