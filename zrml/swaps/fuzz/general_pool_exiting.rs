#![no_main]

use libfuzzer_sys::fuzz_target;
use zrml_swaps::mock::{ExtBuilder, Origin, Swaps};
mod pool_creation;
use pool_creation::get_sample_pool_id;

fuzz_target!(|data: GeneralPoolExiting| {
    let mut ext = ExtBuilder::default().build();
    let _ = ext.execute_with(|| {
        let _ = Swaps::pool_exit(
            Origin::signed(data.origin.into()),
            get_sample_pool_id(),
            data.pool_amount,
            data.min_assets_out,
        );
    });
    let _ = ext.commit_all();
});

#[derive(Debug, arbitrary::Arbitrary)]
struct GeneralPoolExiting {
    origin: u8,
    pool_id: u8,
    pool_amount: u128,
    min_assets_out: Vec<u128>,
}
