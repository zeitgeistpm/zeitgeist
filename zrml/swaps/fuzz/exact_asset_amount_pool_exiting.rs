#![no_main]

use libfuzzer_sys::fuzz_target;
use zrml_swaps::mock::{ExtBuilder, Origin, Swaps};
mod pool_creation;
use pool_creation::get_sample_pool_id;
use pool_creation::asset;

fuzz_target!(|data: ExactAssetAmountPoolExiting| {
    let mut ext = ExtBuilder::default().build();
    let _ = ext.execute_with(|| {
        let _ = Swaps::pool_exit_with_exact_asset_amount(
            Origin::signed(data.origin.into()),
            get_sample_pool_id(),
            asset(data.asset),
            data.asset_amount,
            data.max_pool_amount,
        );
    });
    let _ = ext.commit_all();
});

#[derive(Debug, arbitrary::Arbitrary)]
struct ExactAssetAmountPoolExiting {
    origin: u8,
    pool_id: u8,
    asset: (u128, u16),
    asset_amount: u128,
    max_pool_amount: u128,
}