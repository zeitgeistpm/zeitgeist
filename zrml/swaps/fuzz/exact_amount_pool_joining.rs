#![no_main]

use libfuzzer_sys::fuzz_target;
use zrml_swaps::mock::{ExtBuilder, Origin, Swaps};
mod pool_creation;
use pool_creation::get_sample_pool_id;
use pool_creation::asset;

fuzz_target!(|data: ExactAmountPoolJoining| {
    let mut ext = ExtBuilder::default().build();
    let _ = ext.execute_with(|| {
        let _ = Swaps::pool_join_with_exact_pool_amount(
            Origin::signed(data.origin.into()),
            get_sample_pool_id(),
            asset(data.asset),
            data.pool_amount,
            data.max_asset_amount,
        );
    });
    let _ = ext.commit_all();
});

#[derive(Debug, arbitrary::Arbitrary)]
struct ExactAmountPoolJoining {
    origin: u8,
    pool_id: u8,
    asset: (u128, u16),
    pool_amount: u128,
    max_asset_amount: u128,
}