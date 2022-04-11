#![no_main]

use libfuzzer_sys::fuzz_target;

use zrml_swaps::mock::{ExtBuilder, Origin, Swaps};
mod pool_creation;
use pool_creation::{get_valid_pool_id, ValidPoolData};

fuzz_target!(|data: GeneralPoolJoiningData| {
    let mut ext = ExtBuilder::default().build();
    let _ = ext.execute_with(|| {
        if let Ok(pool_id) = get_valid_pool_id(data.pool_creation) {
            // join a pool with a valid pool id
            let _ = Swaps::pool_join(
                Origin::signed(data.origin.into()),
                pool_id,
                data.pool_amount,
                data.max_assets_in,
            );
        }
    });

    let _ = ext.commit_all();
});

#[derive(Debug, arbitrary::Arbitrary)]
struct GeneralPoolJoiningData {
    origin: u8,
    pool_amount: u128,
    max_assets_in: Vec<u128>,
    pool_creation: ValidPoolData,
}
