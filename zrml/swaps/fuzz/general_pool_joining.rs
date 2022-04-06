#![no_main]

use libfuzzer_sys::fuzz_target;

use zeitgeist_primitives::{traits::Swaps as SwapsTrait, types::ScoringRule};
use zrml_swaps::mock::{ExtBuilder, Origin, Swaps};
mod pool_creation;
use pool_creation::asset;

fuzz_target!(|data: GeneralPoolJoining| {
    let mut ext = ExtBuilder::default().build();
    let _ = ext.execute_with(|| {
        let weights = data.weights;
        let assets = data.assets;
        if weights.clone().is_none() {
            return;
        }
        if &assets.len() != &weights.clone().unwrap().len() {
            return;
        }
        let pool_id_result = Swaps::create_pool(
            data.origin2.into(),
            assets.into_iter().map(asset).collect(),
            data.base_asset.map(asset),
            data.market_id.into(),
            ScoringRule::CPMM,
            data.swap_fee.into(),
            weights.into(),
        );

        if pool_id_result.is_err() {
            return;
        }

        // join a pool with a valid pool id

        let _ = Swaps::pool_join(
            Origin::signed(data.origin.into()),
            pool_id_result.unwrap(),
            data.pool_amount,
            data.max_assets_in,
        );
    });

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
    swap_fee: Option<u128>,
    weights: Option<Vec<u128>>,
}
