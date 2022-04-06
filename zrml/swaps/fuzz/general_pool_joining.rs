#![no_main]

use libfuzzer_sys::fuzz_target;

use zrml_swaps::mock::{ExtBuilder, Origin, Swaps};
use zeitgeist_primitives::traits::Swaps as SwapsTrait;
mod pool_creation;
use pool_creation::scoring_rule;
use pool_creation::asset;

fuzz_target!(|data: GeneralPoolJoining| {
    let mut ext = ExtBuilder::default().build();
    let pool_id_result = Swaps::create_pool(
        data.origin2.into(),
        data.assets.into_iter().map(asset).collect(),
        data.base_asset.map(asset),
        data.market_id.into(),
        scoring_rule(data.scoring_rule),
        data.swap_fee.into(),
        data.weights.into(),
    );
    match pool_id_result {
        Ok(pool_id) => {
            let _ = ext.execute_with(|| {
                let _ = Swaps::pool_join(
                    Origin::signed(data.origin.into()),
                    pool_id,
                    data.pool_amount,
                    data.max_assets_in,
                );
            });
        },
        Err(_) => (),
    };

    let _ = ext.commit_all();
});

#[derive(Debug, arbitrary::Arbitrary)]
struct GeneralPoolJoining {
    origin: u8,
    pool_amount: u128,
    max_assets_in: Vec<u128>,
    // pool creation
    origin2: u8,
    assets: Vec<(u128, u16)>,
    base_asset: Option<(u128, u16)>,
    market_id: u128,
    scoring_rule: u128,
    swap_fee: Option<u128>,
    weights: Option<Vec<u128>>,
}