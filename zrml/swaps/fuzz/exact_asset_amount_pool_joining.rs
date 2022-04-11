#![no_main]

use libfuzzer_sys::fuzz_target;
use zrml_swaps::mock::{ExtBuilder, Origin, Swaps};
mod pool_creation;
use pool_creation::{asset, get_valid_pool_id, ValidPoolData};

fuzz_target!(|data: ExactAssetAmountPoolJoiningData| {
    let mut ext = ExtBuilder::default().build();
    let _ = ext.execute_with(|| {
        if let Ok(pool_id) = get_valid_pool_id(data.pool_creation) {
            let _ = Swaps::pool_join_with_exact_asset_amount(
                Origin::signed(data.origin.into()),
                pool_id,
                asset(data.asset_in),
                data.asset_amount,
                data.min_pool_amount,
            );
        }
    });
    let _ = ext.commit_all();
});

#[derive(Debug, arbitrary::Arbitrary)]
struct ExactAssetAmountPoolJoiningData {
    origin: u8,
    asset_in: (u128, u16),
    asset_amount: u128,
    min_pool_amount: u128,
    pool_creation: ValidPoolData,
}
