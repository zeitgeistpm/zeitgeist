#![no_main]

use libfuzzer_sys::fuzz_target;
use zrml_swaps::mock::{ExtBuilder, Origin, Swaps};
mod pool_creation;
use pool_creation::{get_valid_pool_id, ValidPoolData};

fuzz_target!(|data: GeneralPoolExitingData| {
    let mut ext = ExtBuilder::default().build();
    let _ = ext.execute_with(|| {
        if let Ok(pool_id) = get_valid_pool_id(data.pool_creation) {
            let _ = Swaps::pool_exit(
                Origin::signed(data.origin.into()),
                pool_id,
                data.pool_amount,
                data.min_assets_out,
            );
        }
    });
    let _ = ext.commit_all();
});

#[derive(Debug, arbitrary::Arbitrary)]
struct GeneralPoolExitingData {
    origin: u8,
    pool_amount: u128,
    min_assets_out: Vec<u128>,
    pool_creation: ValidPoolData,
}
