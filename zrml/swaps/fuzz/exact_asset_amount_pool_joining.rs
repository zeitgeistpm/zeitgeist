#![no_main]

use libfuzzer_sys::fuzz_target;
use zrml_swaps::mock::{ExtBuilder, Origin, Swaps};

mod data_structs;
use data_structs::ExactAssetAmountData;
mod helper_functions;
use helper_functions::asset;

use zeitgeist_primitives::{traits::Swaps as SwapsTrait, types::ScoringRule};

fuzz_target!(|data: ExactAssetAmountData| {
    let mut ext = ExtBuilder::default().build();
    let _ = ext.execute_with(|| {
        if let Ok(pool_id) = Swaps::create_pool(
            data.pool_creation.origin.into(),
            data.pool_creation.assets.into_iter().map(asset).collect(),
            Some(data.pool_creation.base_asset).map(asset),
            data.pool_creation.market_id,
            ScoringRule::CPMM,
            Some(data.pool_creation.swap_fee),
            Some(data.pool_creation.weights),
        ) {
            let _ = Swaps::pool_join_with_exact_asset_amount(
                Origin::signed(data.origin.into()),
                pool_id,
                asset(data.asset),
                data.asset_amount,
                data.pool_amount,
            );
        } else {
            panic!(
                "There needs to be a valid pool creation! This Swaps::create_pool call returns an \
                 error, but should be ok."
            );
        }
    });
    let _ = ext.commit_all();
});
