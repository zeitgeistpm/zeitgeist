#![no_main]

use libfuzzer_sys::fuzz_target;
use zrml_swaps::mock::{ExtBuilder, Origin, Swaps};
mod pool_creation;
use pool_creation::get_sample_pool_id;
use pool_creation::asset;

fuzz_target!(|data: ExactAssetAmountPoolJoining| {
    let mut ext = ExtBuilder::default().build();
    let _ = ext.execute_with(|| {
        let _ = Swaps::pool_join_with_exact_asset_amount(
            Origin::signed(data.origin.into()),
            get_sample_pool_id(),
            asset(data.asset_in),
            data.asset_amount,
            data.min_pool_amount,
        );
    });
    let _ = ext.commit_all();
});

#[derive(Debug, arbitrary::Arbitrary)]
struct ExactAssetAmountPoolJoining {
    origin: u8,
    pool_id: u8,
    asset_in: (u128, u16),
    asset_amount: u128,
    min_pool_amount: u128,
}