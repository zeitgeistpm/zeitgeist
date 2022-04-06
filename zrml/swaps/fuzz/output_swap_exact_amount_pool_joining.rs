#![no_main]

use libfuzzer_sys::fuzz_target;
use zrml_swaps::mock::{ExtBuilder, Origin, Swaps};
mod pool_creation;
use pool_creation::{asset, get_sample_pool_id};

fuzz_target!(|data: OutputSwapExactAmountPoolJoining| {
    let mut ext = ExtBuilder::default().build();
    let _ = ext.execute_with(|| {
        let _ = Swaps::swap_exact_amount_out(
            Origin::signed(data.origin.into()),
            get_sample_pool_id(),
            asset(data.asset_in),
            data.max_amount_asset_in,
            asset(data.asset_out),
            data.asset_amount_out,
            data.max_price,
        );
    });
    let _ = ext.commit_all();
});

#[derive(Debug, arbitrary::Arbitrary)]
struct OutputSwapExactAmountPoolJoining {
    origin: u8,
    pool_id: u8,
    asset_in: (u128, u16),
    max_amount_asset_in: u128,
    asset_out: (u128, u16),
    asset_amount_out: u128,
    max_price: u128,
}
