#![no_main]

use libfuzzer_sys::fuzz_target;
use zeitgeist_primitives::{traits::Swaps as SwapsTrait, types::ScoringRule};

use zrml_swaps::mock::{ExtBuilder, Swaps};

mod data_structs;
use data_structs::PoolCreationData;

mod helper_functions;
use helper_functions::asset;

fuzz_target!(|data: PoolCreationData| {
    let mut ext = ExtBuilder::default().build();
    let _ = ext.execute_with(|| {
        let _ = Swaps::create_pool(
            data.origin.into(),
            data.assets.into_iter().map(asset).collect(),
            data.base_asset.map(asset),
            data.market_id.into(),
            ScoringRule::CPMM,
            data.swap_fee.into(),
            data.weights.into(),
        );
    });
    let _ = ext.commit_all();
});
