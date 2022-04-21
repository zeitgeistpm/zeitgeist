#![no_main]

use libfuzzer_sys::fuzz_target;
use zeitgeist_primitives::{traits::Swaps as SwapsTrait, types::ScoringRule};

use zrml_swaps::mock::{ExtBuilder, Swaps};

mod utils;
use utils::{construct_asset, PoolCreationData};

fuzz_target!(|data: PoolCreationData| {
    let mut ext = ExtBuilder::default().build();
    let _ = ext.execute_with(|| {
        let _ = Swaps::create_pool(
            data.origin,
            data.assets.into_iter().map(construct_asset).collect(),
            construct_asset(data.base_asset),
            data.market_id,
            ScoringRule::CPMM,
            data.swap_fee,
            data.weights,
        );
    });
    let _ = ext.commit_all();
});
