#![no_main]

use libfuzzer_sys::fuzz_target;
use zrml_swaps::mock::{ExtBuilder, Origin, Swaps};
mod pool_creation;
use pool_creation::{asset, get_valid_pool_id, ValidPoolData};

fuzz_target!(|data: InputSwapExactAmountPoolJoiningData| {
    let mut ext = ExtBuilder::default().build();
    let _ = ext.execute_with(|| {
        if let Ok(pool_id) = get_valid_pool_id(data.pool_creation) {
            let _ = Swaps::swap_exact_amount_in(
                Origin::signed(data.origin.into()),
                pool_id,
                asset(data.asset_in),
                data.asset_amount_in,
                asset(data.asset_out),
                data.min_asset_amount_out,
                data.max_price,
            );
        }
    });
    let _ = ext.commit_all();
});

#[derive(Debug, arbitrary::Arbitrary)]
struct InputSwapExactAmountPoolJoiningData {
    origin: u8,
    asset_in: (u128, u16),
    asset_amount_in: u128,
    asset_out: (u128, u16),
    min_asset_amount_out: u128,
    max_price: u128,
    pool_creation: ValidPoolData,
}
