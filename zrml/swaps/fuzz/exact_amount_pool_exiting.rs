#![no_main]

use libfuzzer_sys::fuzz_target;
use zrml_swaps::mock::{ExtBuilder, Origin, Swaps};

mod pool_creation;
use pool_creation::{asset, get_valid_pool_id, ValidPoolData};

fuzz_target!(|data: ExactAmountPoolExitingData| {
    let mut ext = ExtBuilder::default().build();
    let _ = ext.execute_with(|| {
        if let Ok(pool_id) = get_valid_pool_id(data.pool_creation) {
            let _ = Swaps::pool_exit_with_exact_pool_amount(
                Origin::signed(data.origin.into()),
                pool_id,
                asset(data.asset),
                data.pool_amount,
                data.min_asset_amount,
            );
        }
    });
    let _ = ext.commit_all();
});

#[derive(Debug, arbitrary::Arbitrary)]
struct ExactAmountPoolExitingData {
    origin: u8,
    asset: (u128, u16),
    pool_amount: u128,
    min_asset_amount: u128,
    pool_creation: ValidPoolData,
}
