#![no_main]

use libfuzzer_sys::fuzz_target;
use zrml_swaps::mock::{ExtBuilder, Origin, Swaps};

mod pool_creation;
use pool_creation::{asset, get_valid_pool_id, PoolCreation};

fuzz_target!(|data: ExactAmountPoolJoining| {
    let mut ext = ExtBuilder::default().build();
    let _ = ext.execute_with(|| {
        let pool_id_opt = get_valid_pool_id(data.pool_creation);
        if let Some(pool_id) = pool_id_opt {
            let _ = Swaps::pool_join_with_exact_pool_amount(
                Origin::signed(data.origin.into()),
                pool_id,
                asset(data.asset),
                data.pool_amount,
                data.max_asset_amount,
            );
        }
    });
    let _ = ext.commit_all();
});

#[derive(Debug, arbitrary::Arbitrary)]
struct ExactAmountPoolJoining {
    origin: u8,
    asset: (u128, u16),
    pool_amount: u128,
    max_asset_amount: u128,
    pool_creation: PoolCreation,
}
